# Task Queue and Concurrency

`packwire-rs/src/task_queue.rs` provides a synchronized queue for network, extraction, and installation work. Each job has an identifier, status, progress, and log events that can be queried by the GUI.

The queue keeps work off the render thread and coordinates workers through channels and synchronized state. Errors are returned to the job without leaving partial installation records.
