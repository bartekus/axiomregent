use anyhow::{Context, Result};
use std::net::TcpStream;
use std::time::Duration;
use url::Url;

/// Context for readiness checks (e.g. valid endpoint).
pub struct ReadinessContext {
    pub endpoint: Option<Url>,
}

/// Defines a readiness check that can be performed.
pub trait ReadinessCheck: Send + Sync {
    /// Runs the check.
    /// Returns Ok(()) if healthy, Err if not.
    fn check(&self, ctx: &ReadinessContext) -> Result<()>;
    /// Returns a descriptive name for the check.
    fn name(&self) -> &str;
}

pub struct TcpCheck;

impl ReadinessCheck for TcpCheck {
    fn check(&self, ctx: &ReadinessContext) -> Result<()> {
        let endpoint = ctx.endpoint.as_ref().context("No endpoint detected yet")?;
        let addr = *endpoint
            .socket_addrs(|| None)?
            .first()
            .ok_or_else(|| anyhow::anyhow!("No socket addr"))?;

        TcpStream::connect_timeout(&addr, Duration::from_secs(1))
            .map(|_| ())
            .with_context(|| format!("TCP connect to {} failed", addr))
    }

    fn name(&self) -> &str {
        "tcp"
    }
}

pub struct HttpCheck {
    pub path: String,
}

impl ReadinessCheck for HttpCheck {
    fn check(&self, ctx: &ReadinessContext) -> Result<()> {
        let endpoint = ctx.endpoint.as_ref().context("No endpoint detected yet")?;
        let url = endpoint.join(&self.path)?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        let _ = client.get(url).send()?;
        Ok(())
    }

    fn name(&self) -> &str {
        "http"
    }
}

pub struct AppCheck {
    pub path: String,
}

impl ReadinessCheck for AppCheck {
    fn check(&self, ctx: &ReadinessContext) -> Result<()> {
        let endpoint = ctx.endpoint.as_ref().context("No endpoint detected yet")?;
        let url = endpoint.join(&self.path)?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        let resp = client.get(url).send()?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("HTTP {}", resp.status()))
        }
    }

    fn name(&self) -> &str {
        "app"
    }
}

pub struct HealthProbe {
    pub checks: Vec<std::sync::Arc<dyn ReadinessCheck>>,
}

impl Clone for HealthProbe {
    fn clone(&self) -> Self {
        Self {
            checks: self.checks.clone(),
        }
    }
}

impl HealthProbe {
    pub fn new(checks: Vec<std::sync::Arc<dyn ReadinessCheck>>) -> Self {
        Self { checks }
    }

    pub fn check(&self, ctx: &ReadinessContext) -> Result<()> {
        for check in &self.checks {
            check
                .check(ctx)
                .with_context(|| format!("Check '{}' failed", check.name()))?;
        }
        Ok(())
    }
}
