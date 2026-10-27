# Task Queue & Concurrency

Packwire includes a background task execution subsystem (`packwire.core.task_queue`) designed to process asynchronous package operations without blocking the command line or freezing the graphical interface.

---

## 1. Architecture

```text
[GUI / Client Request]
         │
         ▼
[task_queue.submit_install()]
         │
         ├─── Enqueue job (TaskJob)
         │
         ▼
[Threaded Worker Loop] (PackwireWorker-0)
         │
         ├─── Update status ("running")
         ├─── Capture real-time log lines (thread-safe lock)
         ├─── Stream progress percentage (0 - 100%)
         │
         ▼
[Installer Execution]
         │
         ▼
[Completion / Failure]
         │
         └─── Persist result dict & status ("completed" | "failed")
```

---

## 2. Key Components

### `TaskJob`
Encapsulates all metadata and runtime state of a background job:
- `id`: Deterministic unique identifier (e.g., `job-a1b2c3d4`).
- `package_id` & `package_name`: Target package identifiers.
- `status`: Current state (`queued`, `running`, `completed`, `failed`).
- `status_text`: Latest status message for UI progress bars.
- `progress`: Numerical percentage (0 to 100, or -1 for indeterminate).
- `logs`: Thread-safe chronologically ordered log entries.
- `result`: Outcome dictionary containing `success` boolean and summary message.

### `TaskQueue`
- Manages an internal `queue.Queue` fed to dedicated daemon threads.
- Exposes `get_job(job_id)` and `list_jobs()` for polling from GUI or IPC endpoints.
- Thread-safe synchronization via `threading.Lock`.
