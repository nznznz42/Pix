use std::thread;

pub fn available_threads() -> usize {
    return thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}