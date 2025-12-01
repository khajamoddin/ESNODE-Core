// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB
use agent_core::state::StatusSnapshot;
use anyhow::{Context, Result};

/// Lightweight HTTP client for talking to the local agent without external deps.
pub struct AgentClient {
    base_url: String,
}

impl AgentClient {
    pub fn new(listen_address: &str) -> Self {
        let normalized =
            if listen_address.starts_with("http://") || listen_address.starts_with("https://") {
                listen_address.to_string()
            } else {
                format!("http://{}", listen_address)
            };
        AgentClient {
            base_url: normalized.trim_end_matches('/').to_string(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn fetch_status(&self) -> Result<StatusSnapshot> {
        let url = format!("{}/status", self.base_url);
        let body = ureq::get(&url)
            .call()
            .context("requesting /status")?
            .into_string()
            .context("reading /status body")?;
        let snapshot: StatusSnapshot =
            serde_json::from_str(&body).context("parsing status JSON")?;
        Ok(snapshot)
    }

    pub fn fetch_metrics_text(&self) -> Result<String> {
        let url = format!("{}/metrics", self.base_url);
        let body = ureq::get(&url)
            .call()
            .context("requesting /metrics")?
            .into_string()
            .context("reading /metrics body")?;
        Ok(body)
    }
}
