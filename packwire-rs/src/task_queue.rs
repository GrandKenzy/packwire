//! Cola de trabajos para instalaciones en segundo plano. Los workers reciben
//! mensajes por un canal, comparten únicamente estado protegido y se cierran
//! de forma determinista cuando se destruye la cola.

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use time::OffsetDateTime;

use crate::config::Paths;
use crate::installer::Installer;
use crate::manifest::ManifestLoader;
use crate::models::Manifest;
use crate::resolver::{resolve_manifest, Resolver};

pub type QueueResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
pub type InstallHandler =
    dyn Fn(InstallRequest, Arc<TaskJob>) -> QueueResult<(bool, String)> + Send + Sync + 'static;

static JOB_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    pub package_id: String,
    pub package_name: String,
    pub mode: String,
    pub version: Option<String>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskJobSnapshot {
    pub id: String,
    pub package_id: String,
    pub package_name: String,
    pub mode: String,
    pub version: Option<String>,
    pub status: String,
    pub status_text: String,
    pub progress: i32,
    pub logs: Vec<String>,
    pub result: Option<TaskResult>,
    pub created_at: f64,
    pub completed_at: Option<f64>,
}

#[derive(Debug)]
struct TaskJobState {
    status: String,
    status_text: String,
    progress: i32,
    logs: Vec<String>,
    result: Option<TaskResult>,
    completed_at: Option<f64>,
}

/// Estado de una instalación individual, seguro para consultar desde la GUI.
pub struct TaskJob {
    pub id: String,
    pub package_id: String,
    pub package_name: String,
    pub mode: String,
    pub version: Option<String>,
    pub force: bool,
    pub created_at: f64,
    state: Mutex<TaskJobState>,
}

impl std::fmt::Debug for TaskJob {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskJob")
            .field("id", &self.id)
            .field("package_id", &self.package_id)
            .field("package_name", &self.package_name)
            .field("mode", &self.mode)
            .field("version", &self.version)
            .field("force", &self.force)
            .field("created_at", &self.created_at)
            .finish_non_exhaustive()
    }
}

impl TaskJob {
    fn new(request: InstallRequest) -> Self {
        Self {
            id: next_job_id(),
            package_id: request.package_id,
            package_name: request.package_name,
            mode: request.mode,
            version: request.version,
            force: request.force,
            created_at: unix_timestamp(),
            state: Mutex::new(TaskJobState {
                status: "queued".into(),
                status_text: "En cola de espera...".into(),
                progress: 0,
                logs: Vec::new(),
                result: None,
                completed_at: None,
            }),
        }
    }

    pub fn request(&self) -> InstallRequest {
        InstallRequest {
            package_id: self.package_id.clone(),
            package_name: self.package_name.clone(),
            mode: self.mode.clone(),
            version: self.version.clone(),
            force: self.force,
        }
    }

    pub fn add_log(&self, message: impl AsRef<str>) {
        if let Ok(mut state) = self.state.lock() {
            let line = format!("[{}] {}", clock_string(), message.as_ref());
            state.logs.push(line);
            state.status_text = message.as_ref().to_owned();
        }
    }

    pub fn set_status(&self, status: impl Into<String>, status_text: Option<&str>) {
        if let Ok(mut state) = self.state.lock() {
            state.status = status.into();
            if let Some(status_text) = status_text.filter(|text| !text.is_empty()) {
                state.status_text = status_text.to_owned();
            }
            if matches!(state.status.as_str(), "completed" | "failed") {
                state.completed_at = Some(unix_timestamp());
            }
        }
    }

    pub fn set_progress(&self, progress: i32) {
        if let Ok(mut state) = self.state.lock() {
            state.progress = progress;
        }
    }

    pub fn set_result(&self, success: bool, message: impl Into<String>) {
        let message = message.into();
        if let Ok(mut state) = self.state.lock() {
            state.result = Some(TaskResult {
                success,
                message: message.clone(),
            });
            state.status = if success { "completed" } else { "failed" }.into();
            state.status_text = message;
            state.completed_at = Some(unix_timestamp());
        }
    }

    pub fn snapshot(&self) -> TaskJobSnapshot {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        TaskJobSnapshot {
            id: self.id.clone(),
            package_id: self.package_id.clone(),
            package_name: self.package_name.clone(),
            mode: self.mode.clone(),
            version: self.version.clone(),
            status: state.status.clone(),
            status_text: state.status_text.clone(),
            progress: state.progress,
            logs: state.logs.clone(),
            result: state.result.clone(),
            created_at: self.created_at,
            completed_at: state.completed_at,
        }
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self.snapshot()).expect("TaskJobSnapshot siempre es serializable")
    }

    /// Representación tipo diccionario para la interfaz JSON de la aplicación.
    pub fn to_dict(&self) -> HashMap<String, Value> {
        self.to_value()
            .as_object()
            .map(|object| {
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

enum WorkMessage {
    Install(Arc<TaskJob>),
    Shutdown,
}

/// Pool de workers para operaciones de instalación.
pub struct TaskQueue {
    sender: mpsc::Sender<WorkMessage>,
    jobs: Arc<Mutex<HashMap<String, Arc<TaskJob>>>>,
    workers: Vec<JoinHandle<()>>,
}

impl std::fmt::Debug for TaskQueue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskQueue")
            .field(
                "job_count",
                &self.jobs.lock().map(|jobs| jobs.len()).unwrap_or(0),
            )
            .field("worker_count", &self.workers.len())
            .finish_non_exhaustive()
    }
}

impl TaskQueue {
    /// Crea una cola con un executor de respaldo que marca el trabajo como error.
    /// Use with_executor o with_packwire para ejecutar instalaciones reales.
    pub fn new(num_workers: usize) -> Self {
        Self::with_executor(num_workers, |_request, _job| {
            Err(io::Error::other("TaskQueue no tiene un executor configurado").into())
        })
    }

    pub fn with_executor<F>(num_workers: usize, executor: F) -> Self
    where
        F: Fn(InstallRequest, Arc<TaskJob>) -> QueueResult<(bool, String)> + Send + Sync + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let jobs = Arc::new(Mutex::new(HashMap::new()));
        let executor: Arc<InstallHandler> = Arc::new(executor);
        let worker_count = num_workers.max(1);
        let mut workers = Vec::with_capacity(worker_count);

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let executor = Arc::clone(&executor);
            let name = format!("PackwireWorker-{index}");
            workers.push(
                thread::Builder::new()
                    .name(name)
                    .spawn(move || worker_loop(receiver, executor))
                    .expect("debe poder iniciarse un worker de Packwire"),
            );
        }

        Self {
            sender,
            jobs,
            workers,
        }
    }

    /// Crea una cola conectada al instalador y al descubridor de manifests.
    pub fn with_packwire(paths: Paths, num_workers: usize) -> QueueResult<Self> {
        let installer = Arc::new(Mutex::new(Installer::new(paths.clone())?));
        let loader = ManifestLoader::new(paths.manifests_dir.clone());
        let resolver = Resolver::new()?;

        Ok(Self::with_executor(num_workers, move |request, job| {
            let mut manifest = find_manifest(&loader, &request)?;
            prepare_manifest(&resolver, &mut manifest, &request)?;

            let installer = installer
                .lock()
                .map_err(|_| io::Error::other("el instalador está bloqueado por otro worker"))?;
            let logger = |message: &str| job.add_log(message);
            installer.install(
                &mut manifest,
                Some(&request.mode),
                request.force,
                Some(&logger),
            )
        }))
    }

    pub fn submit_install(
        &self,
        package_id: impl Into<String>,
        package_name: Option<String>,
        mode: impl Into<String>,
        version: Option<String>,
        force: bool,
    ) -> QueueResult<String> {
        let package_id = package_id.into();
        let request = InstallRequest {
            package_name: package_name.unwrap_or_else(|| package_id.clone()),
            package_id,
            mode: mode.into(),
            version,
            force,
        };
        let job = Arc::new(TaskJob::new(request));
        let job_id = job.id.clone();
        self.jobs
            .lock()
            .map_err(|_| io::Error::other("el registro de trabajos está bloqueado"))?
            .insert(job_id.clone(), Arc::clone(&job));

        if let Err(error) = self.sender.send(WorkMessage::Install(job)) {
            self.jobs
                .lock()
                .map_err(|_| io::Error::other("el registro de trabajos está bloqueado"))?
                .remove(&job_id);
            return Err(io::Error::other(format!("no se pudo encolar el trabajo: {error}")).into());
        }
        Ok(job_id)
    }

    pub fn get_job(&self, job_id: &str) -> Option<HashMap<String, Value>> {
        self.jobs
            .lock()
            .ok()
            .and_then(|jobs| jobs.get(job_id).cloned())
            .map(|job| job.to_dict())
    }

    pub fn get_job_snapshot(&self, job_id: &str) -> Option<TaskJobSnapshot> {
        self.jobs
            .lock()
            .ok()
            .and_then(|jobs| jobs.get(job_id).cloned())
            .map(|job| job.snapshot())
    }

    pub fn list_jobs(&self) -> Vec<HashMap<String, Value>> {
        let mut jobs: Vec<_> = self
            .jobs
            .lock()
            .map(|jobs| jobs.values().cloned().collect())
            .unwrap_or_default();
        jobs.sort_by(|left, right| right.created_at.total_cmp(&left.created_at));
        jobs.into_iter().map(|job| job.to_dict()).collect()
    }

    pub fn list_job_snapshots(&self) -> Vec<TaskJobSnapshot> {
        let mut jobs: Vec<_> = self
            .jobs
            .lock()
            .map(|jobs| jobs.values().cloned().collect())
            .unwrap_or_default();
        jobs.sort_by(|left, right| right.created_at.total_cmp(&left.created_at));
        jobs.into_iter().map(|job| job.snapshot()).collect()
    }

    pub fn job_count(&self) -> usize {
        self.jobs.lock().map(|jobs| jobs.len()).unwrap_or(0)
    }
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        for _ in &self.workers {
            let _ = self.sender.send(WorkMessage::Shutdown);
        }
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

fn worker_loop(receiver: Arc<Mutex<mpsc::Receiver<WorkMessage>>>, executor: Arc<InstallHandler>) {
    loop {
        let message = match receiver.lock() {
            Ok(receiver) => receiver.recv(),
            Err(_) => return,
        };
        match message {
            Ok(WorkMessage::Shutdown) | Err(_) => return,
            Ok(WorkMessage::Install(job)) => run_job(job, &executor),
        }
    }
}

fn run_job(job: Arc<TaskJob>, executor: &Arc<InstallHandler>) {
    job.set_status(
        "running",
        Some(&format!(
            "Iniciando instalación de '{}'...",
            job.package_name
        )),
    );
    job.add_log(format!(
        "Comenzando tarea para {} ({}) en modo '{}'",
        job.package_name, job.package_id, job.mode
    ));

    let request = job.request();
    let result = catch_unwind(AssertUnwindSafe(|| executor(request, Arc::clone(&job))));
    let (success, message) = match result {
        Ok(Ok((success, message))) => (success, message),
        Ok(Err(error)) => (false, format!("Excepción inesperada: {error}")),
        Err(payload) => (
            false,
            format!("Panic inesperado en worker: {}", panic_message(payload)),
        ),
    };

    job.set_result(success, message.clone());
    job.add_log(format!(
        "Resultado final: {} - {message}",
        if success { "OK" } else { "ERROR" }
    ));
}

fn find_manifest(loader: &ManifestLoader, request: &InstallRequest) -> QueueResult<Manifest> {
    let manifest = if request.version.is_some() && !request.package_id.contains('@') {
        loader
            .find_manifest(&format!(
                "{}@{}",
                request.package_id,
                request.version.as_deref().unwrap_or_default()
            ))
            .or_else(|| loader.find_manifest(&request.package_id))
    } else if !request.package_id.contains('@') {
        loader
            .find_manifest(&format!("{}@stable", request.package_id))
            .or_else(|| loader.find_manifest(&request.package_id))
    } else {
        loader.find_manifest(&request.package_id)
    };
    manifest.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "No se encontró ningún manifest para '{}'.",
                request.package_id
            ),
        )
        .into()
    })
}

fn prepare_manifest(
    resolver: &Resolver,
    manifest: &mut Manifest,
    request: &InstallRequest,
) -> QueueResult<()> {
    if let Some(version) = request.version.as_deref() {
        manifest.version = Some(version.to_owned());
        manifest.package_type = "fixed".into();
        manifest.id = format!(
            "{}@{}",
            request
                .package_id
                .split('@')
                .next()
                .unwrap_or(&request.package_id)
                .to_lowercase(),
            version
        );
        resolve_manifest(resolver, manifest)?;
    } else if !request.package_id.contains('@') {
        manifest.package_type = "stable".into();
        manifest.id = format!("{}@stable", request.package_id.to_lowercase());
        resolve_manifest(resolver, manifest)?;
    }
    Ok(())
}

fn next_job_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or_default();
    let sequence = JOB_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("job-{:08x}", (now ^ sequence) as u32)
}

fn unix_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default()
}

fn clock_string() -> String {
    let time = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format!(
        "{:02}:{:02}:{:02}",
        time.hour(),
        time.minute(),
        time.second()
    )
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "error sin mensaje".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use std::thread;
    use std::time::Duration;

    fn wait_for_terminal(queue: &TaskQueue, job_id: &str) -> TaskJobSnapshot {
        for _ in 0..100 {
            if let Some(job) = queue.get_job_snapshot(job_id) {
                if matches!(job.status.as_str(), "completed" | "failed") {
                    return job;
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("el trabajo no terminó a tiempo");
    }

    #[test]
    fn task_job_protects_logs_and_returns_consistent_snapshots() {
        let job = Arc::new(TaskJob::new(InstallRequest {
            package_id: "tool".into(),
            package_name: "Tool".into(),
            mode: "here".into(),
            version: None,
            force: false,
        }));
        let mut workers = Vec::new();
        for index in 0..8 {
            let job = Arc::clone(&job);
            workers.push(thread::spawn(move || {
                job.add_log(format!("log-{index}"));
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
        let snapshot = job.snapshot();
        assert_eq!(snapshot.logs.len(), 8);
        assert_eq!(snapshot.status, "queued");
    }

    #[test]
    fn queue_runs_jobs_and_orders_newest_first() {
        let queue = TaskQueue::with_executor(2, |request, job| {
            job.set_progress(100);
            job.add_log(format!("ejecutando {}", request.package_id));
            Ok((true, "completado".into()))
        });
        let first = queue
            .submit_install("one", None, "here", None, false)
            .unwrap();
        let second = queue
            .submit_install("two", Some("Dos".into()), "command", None, false)
            .unwrap();
        let first_snapshot = wait_for_terminal(&queue, &first);
        let second_snapshot = wait_for_terminal(&queue, &second);
        assert_eq!(first_snapshot.status, "completed");
        assert_eq!(first_snapshot.progress, 100);
        assert_eq!(second_snapshot.package_name, "Dos");
        assert_eq!(queue.list_job_snapshots().len(), 2);
        assert_eq!(queue.list_jobs()[0]["id"], second);
    }

    #[test]
    fn executor_errors_and_panics_become_failed_jobs() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_executor = Arc::clone(&calls);
        let queue = TaskQueue::with_executor(1, move |_request, _job| {
            let call = calls_for_executor.fetch_add(1, Ordering::Relaxed);
            if call == 0 {
                panic!("fallo de prueba");
            }
            Err(io::Error::other("error de prueba").into())
        });
        let first = queue
            .submit_install("panic", None, "here", None, false)
            .unwrap();
        let second = queue
            .submit_install("error", None, "here", None, false)
            .unwrap();
        assert_eq!(wait_for_terminal(&queue, &first).status, "failed");
        assert_eq!(wait_for_terminal(&queue, &second).status, "failed");
    }

    #[test]
    fn drop_joins_workers_after_queued_work_finishes() {
        let finished = Arc::new(AtomicBool::new(false));
        let finished_for_executor = Arc::clone(&finished);
        let queue = TaskQueue::with_executor(1, move |_request, _job| {
            thread::sleep(Duration::from_millis(10));
            finished_for_executor.store(true, Ordering::Release);
            Ok((true, "ok".into()))
        });
        queue
            .submit_install("tool", None, "here", None, false)
            .unwrap();
        drop(queue);
        assert!(finished.load(Ordering::Acquire));
    }
}
