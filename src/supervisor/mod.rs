pub mod lock;
pub mod process;
pub mod state;

use crate::readiness::{HealthProbe, ReadinessContext};
use process::kill_gracefully;
use regex::Regex;
use serde::Serialize;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tokio_util::sync::CancellationToken;

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
}

impl Supervisor {
    pub async fn run(self, token: CancellationToken) -> std::io::Result<()> {
        let mut cmd = Command::new(&self.cmd);
        cmd.args(&self.args)
            .envs(self.env)
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // We could implement FileLock here if we had a lock path

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");
        let pid = child.id().expect("has pid");

        let (tx, mut rx) = tokio::sync::watch::channel::<Option<String>>(None);

        // stdout pump
        tokio::spawn(async move {
            let re = Regex::new(r"Running on (http://[^\s]+)").expect("regex validity");
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                log::info!("stdout: {}", line);
                if let Some(caps) = re.captures(&line) {
                    if let Some(url) = caps.get(1) {
                        let u = url.as_str().to_string();
                        log::info!("Detected endpoint: {}", u);

                        // Write info file
                        let info = EncoreInfo {
                            endpoint: u.clone(),
                            pid,
                        };

                        if let Err(e) = std::fs::create_dir_all(".axiomregent/run") {
                            log::error!("Failed to create run dir: {}", e);
                        } else {
                            if let Ok(json) = serde_json::to_string(&info) {
                                if let Err(e) = std::fs::write(".axiomregent/run/encore.json", json)
                                {
                                    log::error!("Failed to write encore.json: {}", e);
                                }
                            }
                        }

                        let _ = tx.send(Some(u));
                    }
                }
            }
        });

        // stderr pump
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                log::info!("stderr: {}", line);
            }
        });

        // Readiness probe
        if let Some(probe) = self.health_probe {
            let probe_token = token.clone();
            tokio::spawn(async move {
                // Wait for endpoint
                let endpoint_url = loop {
                    if probe_token.is_cancelled() {
                        return;
                    }
                    let val = rx.borrow().clone();
                    if let Some(u) = val {
                        if let Ok(parsed) = url::Url::parse(&u) {
                            break parsed;
                        }
                    }
                    if rx.changed().await.is_err() {
                        return;
                    }
                };

                let ctx = ReadinessContext {
                    endpoint: Some(endpoint_url),
                };

                loop {
                    if probe_token.is_cancelled() {
                        break;
                    }
                    match probe.check(&ctx) {
                        Ok(_) => {} // healthy
                        Err(e) => log::warn!("Readiness check failed: {}", e),
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        }

        tokio::select! {
             status = child.wait() => {
                 match status {
                     Ok(_) => Ok(()),
                     Err(e) => Err(e),
                 }
             },
             _ = token.cancelled() => {
                 kill_gracefully(&mut child).await
             }
        }
    }
}
