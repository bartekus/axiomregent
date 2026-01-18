use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::Path;

#[derive(Debug)]
pub struct RepoLock {
    file: File,
}

impl RepoLock {
    /// Attempts to acquire an exclusive lock on the given path.
    /// Returns Ok(RepoLock) if successful.
    /// Returns Err if the lock cannot be acquired (already locked by another process)
    /// or if there is an IO error.
    pub fn try_acquire(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .context("Failed to open lock file")?;

        match file.try_lock_exclusive() {
            Ok(_) => Ok(Self { file }),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Err(anyhow::anyhow!("Lock is held by another process"))
            }
            Err(e) => Err(anyhow::Error::new(e).context("Failed to acquire lock")),
        }
    }
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        // OS releases lock automatically on close, but we can be explicit.
        let _ = self.file.unlock();
    }
}

include!("lock_tests.rs");
