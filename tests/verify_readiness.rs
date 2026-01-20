use anyhow::Result;
use axiomregent::supervisor::Supervisor;
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[test]
fn test_endpoint_detection_and_file_write() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        // Setup
        let run_dir = PathBuf::from(".axiomregent/run");
        if run_dir.exists() {
            // Clean up previous runs if possible
            let _ = std::fs::remove_dir_all(&run_dir);
        }

        let log_buffer = std::sync::Arc::new(axiomregent::supervisor::buffer::LogBuffer::new(100));
        let (s_handle, command_rx) =
            axiomregent::supervisor::SupervisorHandle::new(log_buffer.clone());

        let sup = Supervisor {
            cmd: "sh".into(),
            args: vec![
                "-c".into(),
                "echo 'Running on http://127.0.0.1:9999'; sleep 10".into(),
            ],
            cwd: std::env::current_dir()?,
            env: vec![],
            health_probe: None,
            log_buffer: log_buffer.clone(),
            state: s_handle.state.clone(),
            command_rx,
        };

        let token = CancellationToken::new();

        // Spawn supervisor
        let ct = token.clone();
        let handle = tokio::spawn(async move {
            sup.run(ct).await.unwrap();
        });

        // Wait for file creation
        let mut found = false;
        let file_path = run_dir.join("encore.json");
        for _ in 0..50 {
            // 10 seconds max
            if file_path.exists() {
                found = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        token.cancel();
        let _ = handle.await;

        assert!(found, "encore.json should be created within timeout");

        let content = std::fs::read_to_string(file_path)?;
        assert!(
            content.contains("http://127.0.0.1:9999"),
            "Content should contain endpoint. Found: {}",
            content
        );

        Ok(())
    })
}
