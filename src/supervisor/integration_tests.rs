#[cfg(test)]
mod tests {
    use crate::supervisor::Supervisor;
    use crate::supervisor::buffer::LogBuffer;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_supervisor_log_capture() {
        let log_buffer = Arc::new(LogBuffer::new(10));
        let cwd = std::env::current_dir().unwrap();

        let (s_handle, command_rx) = crate::supervisor::SupervisorHandle::new(log_buffer.clone());

        let supervisor = Supervisor {
            cmd: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'ignored stdout'; echo 'error log' >&2; sleep 1".to_string(),
            ],
            cwd,
            env: vec![],
            health_probe: None,
            log_buffer: log_buffer.clone(),
            state: s_handle.state.clone(),
            command_rx,
        };

        let token = CancellationToken::new();
        let ct = token.clone();
        // Run in background
        let handle = tokio::spawn(async move { supervisor.run(ct).await });

        // Wait for logs to appear
        tokio::time::sleep(Duration::from_secs(2)).await;

        let logs = log_buffer.read();

        // Verify stderr (stdout is consumed by RPC)
        assert!(logs.iter().any(|l| l.contains("error log")));

        token.cancel();
        // Ensure handle is finished
        let _ = handle.await;
    }
}
