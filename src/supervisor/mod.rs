pub mod buffer;
pub mod lock;
pub mod process;
pub mod state;
pub mod tools;

use crate::readiness::{HealthProbe, ReadinessContext};
use crate::supervisor::state::{State, SupervisorStatus};
use buffer::LogBuffer;
use process::kill_gracefully;
use regex::Regex;
use serde::Serialize;
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

#[derive(Serialize)]
struct EncoreInfo {
    endpoint: String,
    pid: u32,
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

        let re = Regex::new(r"Running on (http://[^\s]+)").expect("regex validity");

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
                .stderr(Stdio::piped());

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

            let stdout = child.stdout.take().expect("stdout piped");
            let stderr = child.stderr.take().expect("stderr piped");

            let (tx, mut rx) = tokio::sync::watch::channel::<Option<String>>(None);

            // stdout pump
            let log_buffer = self.log_buffer.clone();
            let state_ref = self.state.clone();
            let re_clone = re.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log::info!("stdout: {}", line);
                    log_buffer.push(line.clone());
                    if let Some(url) = re_clone.captures(&line).and_then(|c| c.get(1)) {
                        let u = url.as_str().to_string();
                        log::info!("Detected endpoint: {}", u);

                        {
                            let mut s = state_ref.write().unwrap();
                            s.endpoint = Some(u.clone());
                        }

                        let info = EncoreInfo {
                            endpoint: u.clone(),
                            pid,
                        };

                        if let Err(e) = std::fs::create_dir_all(".axiomregent/run") {
                            log::error!("Failed to create run dir: {}", e);
                        } else if let Ok(json) = serde_json::to_string(&info) {
                            #[allow(clippy::collapsible_if)]
                            if let Err(e) = std::fs::write(".axiomregent/run/encore.json", json) {
                                log::error!("Failed to write encore.json: {}", e);
                            }
                        }

                        let _ = tx.send(Some(u));
                    }
                }
            });

            // stderr pump
            let log_buffer_err = self.log_buffer.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    log::info!("stderr: {}", line);
                    log_buffer_err.push(line);
                }
            });

            let probe_token = CancellationToken::new();
            let probe_token_clone = probe_token.clone();

            if let Some(probe) = &self.health_probe {
                let probe = probe.clone();
                let state_ref = self.state.clone();

                tokio::spawn(async move {
                    let endpoint_url = loop {
                        if probe_token_clone.is_cancelled() {
                            return;
                        }
                        let val = rx.borrow().clone();
                        if let Some(parsed) = val.and_then(|u| url::Url::parse(&u).ok()) {
                            break parsed;
                        }
                        if rx.changed().await.is_err() {
                            return;
                        }
                    };

                    let ctx = ReadinessContext {
                        endpoint: Some(endpoint_url),
                    };

                    loop {
                        if probe_token_clone.is_cancelled() {
                            break;
                        }
                        match probe.check(&ctx) {
                            Ok(_) => {
                                let mut s = state_ref.write().unwrap();
                                if s.state != State::Healthy {
                                    log::info!("Transition to Healthy");
                                    s.state = State::Healthy;
                                }
                            }
                            Err(e) => {
                                let mut s = state_ref.write().unwrap();
                                if s.state == State::Healthy {
                                    log::warn!("Transition to Unhealthy: {}", e);
                                    s.state = State::Unhealthy;
                                }
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                });
            }

            tokio::select! {
                 _ = token.cancelled() => {
                     probe_token.cancel();
                     kill_gracefully(&mut child).await?;
                     break;
                 }
                 msg = self.command_rx.recv() => {
                     match msg {
                         Some(SupervisorCommand::Restart) => {
                             log::info!("Restart requested");
                             probe_token.cancel();
                             kill_gracefully(&mut child).await?;
                             // Loop continues to restart
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
                             // Loop continues to backoff/restart
                         }
                         Err(e) => {
                             log::error!("Wait failed: {}", e);
                             // If wait fails, we probably should restart logic or backoff.
                             // Breaking here means restarting loop.
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
