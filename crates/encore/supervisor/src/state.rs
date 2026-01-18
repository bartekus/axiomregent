use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Ownership {
    Managed,
    External,
    Off,
}

impl fmt::Display for Ownership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ownership::Managed => write!(f, "Managed"),
            Ownership::External => write!(f, "External"),
            Ownership::Off => write!(f, "Off"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum State {
    Stopped,
    Starting,
    Healthy,
    Unhealthy,
    Backoff,
    Fatal,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Stopped => write!(f, "Stopped"),
            State::Starting => write!(f, "Starting"),
            State::Healthy => write!(f, "Healthy"),
            State::Unhealthy => write!(f, "Unhealthy"),
            State::Backoff => write!(f, "Backoff"),
            State::Fatal => write!(f, "Fatal"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SupervisorStatus {
    pub ownership: Ownership,
    pub state: State,
    pub pid: Option<u32>,
    pub endpoint: Option<String>,
    pub uptime_seconds: u64,
    pub restart_count: u32,
    pub last_error: Option<String>,
}
