#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_lock_acquisition_success() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");

        // First acquisition should succeed
        let lock1 = RepoLock::try_acquire(&lock_path);
        assert!(lock1.is_ok(), "First lock acquisition failed");
    }

    #[test]
    fn test_lock_exclusion() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");

        // Hold the lock
        let _lock1 = RepoLock::try_acquire(&lock_path).unwrap();

        // Second attempt should fail
        let lock2 = RepoLock::try_acquire(&lock_path);
        assert!(lock2.is_err(), "Second lock acquisition should fail");

        let err = lock2.unwrap_err();
        assert_eq!(err.to_string(), "Lock is held by another process");
    }

    #[test]
    fn test_lock_release_on_drop() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");

        {
            let _lock1 = RepoLock::try_acquire(&lock_path).unwrap();
        } // _lock1 dropped here

        // Should be able to re-acquire
        let lock2 = RepoLock::try_acquire(&lock_path);
        assert!(lock2.is_ok(), "Failed to re-acquire lock after drop");
    }
}
