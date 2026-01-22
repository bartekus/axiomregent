pub mod buffer;
pub mod lock;
pub mod process;
pub mod rpc;
pub mod state;
pub mod tools;

use crate::readiness::HealthProbe;
use crate::supervisor::state::{State, SupervisorStatus};
use buffer::LogBuffer;
use process::kill_gracefully;
use std::sync::Arc;
use std::sync::RwLock;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::sync::mpsc;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tokio_util::sync::CancellationToken;

pub enum SupervisorCommand {
    Restart,
}

#[derive(Clone)]
pub struct SupervisorHandle {
    pub state: Arc<RwLock<SupervisorStatus>>,
    pub command_tx: mpsc::Sender<SupervisorCommand>,
    pub log_buffer: Arc<LogBuffer>,
}

impl SupervisorHandle {
    pub fn new(log_buffer: Arc<LogBuffer>) -> (Self, mpsc::Receiver<SupervisorCommand>) {
        let state = Arc::new(RwLock::new(SupervisorStatus {
            state: State::Stopped,
            endpoint: None,
            pid: None,
        }));
        let (tx, rx) = mpsc::channel(10);
        (
            Self {
                state,
                command_tx: tx,
                log_buffer,
            },
            rx,
        )
    }

    pub fn get_status(&self) -> SupervisorStatus {
        self.state.read().unwrap().clone()
    }

    pub fn restart(&self) -> Result<(), mpsc::error::TrySendError<SupervisorCommand>> {
        self.command_tx.try_send(SupervisorCommand::Restart)
    }
}

pub struct Supervisor {
    pub cmd: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub health_probe: Option<HealthProbe>,
    pub log_buffer: Arc<LogBuffer>,
    pub state: Arc<RwLock<SupervisorStatus>>,
    pub command_rx: mpsc::Receiver<SupervisorCommand>,
}

impl Supervisor {
    pub async fn run(mut self, token: CancellationToken) -> std::io::Result<()> {
        let cmd_name = self.cmd.clone();

        loop {
            if token.is_cancelled() {
                let mut s = self.state.write().unwrap();
                s.state = State::Stopped;
                s.pid = None;
                s.endpoint = None;
                break;
            }

            // Update State: Starting
            {
                let mut s = self.state.write().unwrap();
                s.state = State::Starting;
                s.pid = None;
                s.endpoint = None;
            }
            log::info!("Supervisor starting process: {}", cmd_name);

            let mut cmd = Command::new(&self.cmd);
            cmd.args(&self.args)
                .envs(self.env.clone())
                .current_dir(&self.cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped()); // Enable stdin for RPC

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to spawn {}: {}", cmd_name, e);
                    {
                        let mut s = self.state.write().unwrap();
                        s.state = State::Backoff;
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let pid = child.id().expect("has pid");
            {
                let mut s = self.state.write().unwrap();
                s.pid = Some(pid);
            }

            // stderr pump (Logs)
            let stderr = child.stderr.take().expect("stderr piped");
            let log_buffer_err = self.log_buffer.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    // log::info!("stderr: {}", line);
                    log_buffer_err.push(line);
                }
            });

            // Initializing RPC Client (takes stdin/stdout)
            // We interpret "Protocol Skeleton" as: we TRY to perform handshake.
            // If it fails (e.g. legacy daemon), we might fall back or just error for now (as this is a replacement).
            // For MVP, we assume the binary supports it.
            let mut client_opt = match crate::supervisor::rpc::DaemonClient::new(&mut child) {
                Ok(c) => Some(c),
                Err(e) => {
                    log::error!("Failed to create DaemonClient: {}", e);
                    None
                    // We continue, child might just be running without RPC?
                    // But we consumed stdout/stdin, so maybe just kill it?
                    // For now, let's proceed to wait to see if it exits.
                }
            };

            if let Some(ref mut client) = client_opt {
                log::info!("Performing Daemon Handshake...");
                if let Err(e) = client.hello().await {
                    log::error!("Daemon Handshake failed: {}", e);
                    // If handshake triggers failure, we might want to restart?
                } else {
                    log::info!("Daemon Handshake success.");
                    // Mark healthy? Or wait for probe?
                }
            }

            let probe_token = CancellationToken::new();
            // ... (Probe logic can remain if we have an endpoint, but we don't detect it via regex anymore)
            // ... (Ideally ensure() returns the endpoint. For now, probe is effectively disabled unless we get endpoint from somewhere else)

            tokio::select! {
                 _ = token.cancelled() => {
                     probe_token.cancel();
                     if let Some(mut client) = client_opt {
                         let _ = client.shutdown().await;
                     }
                     kill_gracefully(&mut child).await?;
                     break;
                 }
                 msg = self.command_rx.recv() => {
                     match msg {
                         Some(SupervisorCommand::Restart) => {
                             log::info!("Restart requested");
                             probe_token.cancel();
                             if let Some(mut client) = client_opt {
                                 let _ = client.shutdown().await;
                             }
                             kill_gracefully(&mut child).await?;
                         }
                         None => {
                             probe_token.cancel();
                             kill_gracefully(&mut child).await?;
                             return Ok(());
                         }
                     }
                 }
                 status = child.wait() => {
                     probe_token.cancel();
                     match status {
                         Ok(e) => {
                             log::warn!("Process exited: {}", e);
                             {
                                 let mut s = self.state.write().unwrap();
                                 s.state = State::Backoff;
                                 s.pid = None;
                                 s.endpoint = None;
                             }
                             tokio::time::sleep(Duration::from_secs(2)).await;
                         }
                         Err(e) => {
                             log::error!("Wait failed: {}", e);
                         }
                     }
                 }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests;
