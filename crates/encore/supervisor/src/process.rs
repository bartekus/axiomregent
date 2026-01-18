use anyhow::{Context, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

#[derive(Debug)]
pub struct ChildGuard {
    pub child: Child,
}

impl ChildGuard {
    pub fn new(child: Child) -> Self {
        Self { child }
    }

    /// Asynchronously kill the child process.
    /// This is safer than Drop for controlled shutdown.
    pub async fn kill(&mut self) -> Result<()> {
        // Send SIGTERM using libc on Unix
        #[cfg(unix)]
        {
            if let Some(pid) = self.child.id() {
                unsafe {
                    // Send SIGTERM to the process group (negative PID)
                    // We assume setsid was called on spawn.
                    libc::kill(-(pid as i32), libc::SIGTERM);
                }
            }
        }
        #[cfg(not(unix))]
        {
            // Windows fallback (no process groups easily accessed via std/tokio)
            let _ = self.child.start_kill();
        }

        // Wait for a bit
        match tokio::time::timeout(Duration::from_secs(5), self.child.wait()).await {
            Ok(_) => return Ok(()),
            Err(_) => {
                log::warn!("Process did not exit after SIGTERM, sending SIGKILL");
            }
        }

        // Force kill
        #[cfg(unix)]
        {
            if let Some(pid) = self.child.id() {
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
            }
        }
        #[cfg(not(unix))]
        {
            let _ = self.child.start_kill();
        }

        self.child.wait().await?;
        Ok(())
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // Synchronous kill attempt on drop.
        // We cannot await here, so we do a best-effort start_kill.
        // If the runtime is shutting down, this might be all we get.
        let _ = self.child.start_kill();
    }
}

pub struct ProcessConfig {
    pub binary: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
}

pub fn spawn(config: ProcessConfig) -> Result<ChildGuard> {
    let mut cmd = Command::new(&config.binary);
    cmd.args(&config.args);
    cmd.envs(config.env);

    if let Some(cwd) = config.cwd {
        cmd.current_dir(cwd);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Unix: Create new process group
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        cmd.pre_exec(|| {
            // setsid creates a new session and process group.
            // This allows us to kill the whole tree by killing -(pid).
            libc::setsid();
            Ok(())
        });
    }

    let child = cmd.spawn().context("Failed to spawn process")?;
    Ok(ChildGuard::new(child))
}

include!("process_tests.rs");
