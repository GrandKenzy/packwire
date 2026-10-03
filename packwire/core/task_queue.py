import time
import uuid
import queue
import threading
from typing import Dict, Any, List, Optional, Callable


class TaskJob:
    """Represents a background package installation or update job."""

    def __init__(
        self,
        job_id: str,
        package_id: str,
        package_name: str,
        mode: str = "here",
        version: Optional[str] = None,
        force: bool = False
    ):
        self.id = job_id
        self.package_id = package_id
        self.package_name = package_name
        self.mode = mode
        self.version = version
        self.force = force
        self.status = "queued"  # "queued", "running", "completed", "failed"
        self.status_text = "En cola de espera..."
        self.progress = 0  # 0 to 100, or -1 for indeterminate
        self.logs: List[str] = []
        self.result: Optional[Dict[str, Any]] = None
        self.created_at = time.time()
        self.completed_at: Optional[float] = None
        self._lock = threading.Lock()

    def add_log(self, message: str) -> None:
        """Add a log line thread-safely."""
        timestamp = time.strftime("%H:%M:%S")
        with self._lock:
            self.logs.append(f"[{timestamp}] {message}")
            self.status_text = message

    def set_status(self, status: str, status_text: Optional[str] = None) -> None:
        with self._lock:
            self.status = status
            if status_text:
                self.status_text = status_text
            if status in ("completed", "failed"):
                self.completed_at = time.time()

    def set_result(self, success: bool, message: str) -> None:
        with self._lock:
            self.result = {"success": success, "message": message}
            self.status = "completed" if success else "failed"
            self.status_text = message
            self.completed_at = time.time()

    def to_dict(self) -> Dict[str, Any]:
        with self._lock:
            return {
                "id": self.id,
                "package_id": self.package_id,
                "package_name": self.package_name,
                "mode": self.mode,
                "version": self.version,
                "status": self.status,
                "status_text": self.status_text,
                "progress": self.progress,
                "logs": list(self.logs),
                "result": self.result,
                "created_at": self.created_at,
                "completed_at": self.completed_at,
            }


class TaskQueue:
    """Threaded worker queue executing package operations in background."""

    def __init__(self, num_workers: int = 1):
        self._queue: queue.Queue = queue.Queue()
        self._jobs: Dict[str, TaskJob] = {}
        self._lock = threading.Lock()
        self._stop_event = threading.Event()
        self._workers: List[threading.Thread] = []

        for i in range(num_workers):
            t = threading.Thread(target=self._worker_loop, daemon=True, name=f"PackwireWorker-{i}")
            t.start()
            self._workers.append(t)

    def submit_install(
        self,
        package_id: str,
        package_name: Optional[str] = None,
        mode: str = "here",
        version: Optional[str] = None,
        force: bool = False
    ) -> str:
        """Enqueue an installation task and return unique job ID."""
        job_id = f"job-{uuid.uuid4().hex[:8]}"
        display_name = package_name or package_id
        job = TaskJob(
            job_id=job_id,
            package_id=package_id,
            package_name=display_name,
            mode=mode,
            version=version,
            force=force
        )
        with self._lock:
            self._jobs[job_id] = job

        self._queue.put(job)
        return job_id

    def get_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve job data by ID."""
        with self._lock:
            job = self._jobs.get(job_id)
            return job.to_dict() if job else None

    def list_jobs(self) -> List[Dict[str, Any]]:
        """List all recent jobs ordered by creation time."""
        with self._lock:
            return [j.to_dict() for j in sorted(self._jobs.values(), key=lambda x: x.created_at, reverse=True)]

    def _worker_loop(self) -> None:
        """Background thread executing jobs sequentially."""
        import packwire

        while not self._stop_event.is_set():
            try:
                job: TaskJob = self._queue.get(timeout=1.0)
            except queue.Empty:
                continue

            job.set_status("running", f"Iniciando instalación de '{job.package_name}'...")
            job.add_log(f"Comenzando tarea para {job.package_name} ({job.package_id}) en modo '{job.mode}'")

            try:
                # Execute installation passing the job logger callback
                ok, msg = packwire.install(
                    target=job.package_id,
                    version=job.version,
                    mode=job.mode,
                    force=job.force,
                    logger=job.add_log
                )
                job.set_result(ok, msg)
                job.add_log(f"Resultado final: {'OK' if ok else 'ERROR'} - {msg}")
            except Exception as e:
                err_msg = f"Excepción inesperada: {str(e)}"
                job.set_result(False, err_msg)
                job.add_log(err_msg)
            finally:
                self._queue.task_done()


# Global Singleton TaskQueue
task_queue = TaskQueue()
