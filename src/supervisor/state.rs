use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Stopped,
    Starting,
    Healthy,
    Unhealthy,
    Backoff,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorStatus {
    pub state: State,
    pub endpoint: Option<String>,
    pub pid: Option<u32>,
}
