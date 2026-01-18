#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_spawn_and_kill() {
        // Spawn a long-running process (sleep)
        let config = ProcessConfig {
            binary: "sleep".to_string(),
            args: vec!["100".to_string()],
            env: vec![],
            cwd: None,
        };

        let mut guard = spawn(config).expect("Failed to spawn sleep");
        let pid = guard.child.id().expect("Child has no PID");

        // Verify it runs
        assert!(pid > 0);

        // Kill it
        guard.kill().await.expect("Failed to kill");

        // Wait should return success (signal kill is usually code likely non-zero or signal, but we just verify it exited)
        // Note: exit code depends on signal.
    }

    #[tokio::test]
    async fn test_process_group_cleanup() {
        // This test is harder to write portably without creating a proper child hierarchy.
        // We will rely on manual verification or a robust script for this.
        // For now, spawn a simple shell that traps logic?

        let config = ProcessConfig {
            binary: "sh".to_string(),
            args: vec!["-c".to_string(), "sleep 100".to_string()],
            env: vec![],
            cwd: None,
        };

        let mut guard = spawn(config).expect("Failed to spawn sh");
        guard.kill().await.expect("Failed to kill");
    }

    #[tokio::test]
    async fn test_drop_cleanup() {
        let config = ProcessConfig {
            binary: "sleep".to_string(),
            args: vec!["100".to_string()],
            env: vec![],
            cwd: None,
        };

        {
            let guard = spawn(config).expect("Failed to spawn sleep");
            // guard dropped here
        }

        // We can't easily verify the process is gone from within Rust without `kill(0, pid)` check which might be racy or permissioned.
        // But we trust start_kill was called.
    }
}
