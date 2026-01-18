use std::time::Duration;
use tokio::process::Child;

pub async fn kill_gracefully(child: &mut Child) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use libc;
        if let Some(pid) = child.id() {
            // SIGINT
            unsafe { libc::kill(pid as i32, libc::SIGINT) };

            // Wait a bit
            tokio::select! {
                _ = child.wait() => return Ok(()),
                _ = tokio::time::sleep(Duration::from_secs(5)) => {}
            }

            // SIGTERM
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };

            tokio::select! {
                _ = child.wait() => return Ok(()),
                _ = tokio::time::sleep(Duration::from_secs(5)) => {}
            }

            // SIGKILL (fallback)
        }
    }

    child.kill().await
}
