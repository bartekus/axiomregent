use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::File;
use std::path::PathBuf;

pub struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    pub fn try_acquire(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(&path)
            .with_context(|| format!("Failed to create lock file at {:?}", path))?;
        file.try_lock_exclusive().with_context(|| {
            format!(
                "Failed to lock file at {:?}. Is another instance running?",
                path
            )
        })?;
        Ok(Self { file, path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        // optionally delete file
        // let _ = std::fs::remove_file(&self.path);
    }
}
