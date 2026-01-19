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

        let supervisor = Supervisor {
            cmd: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'hello world'; echo 'error log' >&2; sleep 1".to_string(),
            ],
            cwd,
            env: vec![],
            health_probe: None,
            log_buffer: log_buffer.clone(),
        };

        let token = CancellationToken::new();
        // Run in background
        let handle = tokio::spawn(async move { supervisor.run(token).await });

        // Wait for logs to appear
        tokio::time::sleep(Duration::from_secs(2)).await;

        let logs = log_buffer.read();

        // Verify stdout
        assert!(logs.iter().any(|l| l.contains("hello world")));
        // Verify stderr
        assert!(logs.iter().any(|l| l.contains("error log")));

        // Ensure handle is finished
        let _ = handle.await;
    }
}
