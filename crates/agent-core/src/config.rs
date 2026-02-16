use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrchestratorConfig {
    pub enabled: bool,
    pub token: Option<String>,
    pub allow_public: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DriverConfig {
    pub protocol: String, // "modbus", "dnp3", "snmp"
    pub id: String,
    pub target: String,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token: None,
            allow_public: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum EnforcementMode {
    Monitor,
    Enforce,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_tracing(&self) -> tracing::Level {
        match self {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

/// Global configuration for the ESNODE Agent.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    /// Metadata tags identifying this agent (e.g., env=prod, region=us-east).
    pub tags: HashMap<String, String>,

    /// The interval between metric collection scrapes.
    /// Default: 100ms. High-frequency telemetry (10ms) requires kernel tuning.
    #[serde(with = "humantime_serde")]
    pub scrape_interval: Duration,
    
    // Collectors - Compute
    pub enable_cpu: bool,
    pub enable_memory: bool,
    pub enable_disk: bool,
    pub enable_network: bool,
    pub enable_ebpf: bool,
    
    // Collectors - GPU
    pub enable_gpu: bool,
    pub enable_gpu_amd: bool,
    pub enable_gpu_mig: bool,
    pub enable_gpu_events: bool,
    pub gpu_visible_devices: Option<String>,
    pub mig_config_devices: Option<String>,
    
    // Collectors - Power/Thermal
    pub enable_power: bool,
    pub node_power_envelope_watts: Option<f64>,
    pub enable_rack_thermals: bool,
    
    // Environment
    pub k8s_mode: bool,
    pub enable_mcp: bool, // Mission Control Plane
    
    // App Awareness
    pub enable_app: bool,
    pub app_metrics_url: String,

    // Networking
    pub listen_address: String,

    // Local Storage
    pub enable_local_tsdb: bool,
    pub local_tsdb_path: String,
    pub local_tsdb_retention_hours: u64,
    pub local_tsdb_max_disk_mb: u64,

    // Control Plane
    pub orchestrator: Option<OrchestratorConfig>,
    
    // Policy / Enforcement
    pub efficiency_profile_path: Option<PathBuf>,
    pub enforcement_mode: EnforcementMode,
    #[serde(with = "humantime_serde")]
    pub enforcement_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub dampening_interval: Duration,

    // Drivers
    #[serde(default)]
    pub drivers: Vec<DriverConfig>,

    // Legacy / Other
    pub log_level: LogLevel,
}

// Minimal ConfigOverrides struct for CLI merging
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ConfigOverrides {
    pub listen_address: Option<String>,
    #[serde(default, with = "humantime_serde")]
    pub scrape_interval: Option<Duration>,
    pub enable_cpu: Option<bool>,
    pub enable_memory: Option<bool>,
    pub enable_disk: Option<bool>,
    pub enable_network: Option<bool>,
    pub enable_ebpf: Option<bool>,
    pub enable_gpu: Option<bool>,
    pub enable_gpu_amd: Option<bool>,
    pub enable_gpu_mig: Option<bool>,
    pub enable_gpu_events: Option<bool>,
    pub gpu_visible_devices: Option<String>,
    pub mig_config_devices: Option<String>,
    pub k8s_mode: Option<bool>,
    pub enable_power: Option<bool>,
    pub enable_mcp: Option<bool>,
    pub enable_app: Option<bool>,
    pub app_metrics_url: Option<String>,
    pub enable_rack_thermals: Option<bool>,
    pub node_power_envelope_watts: Option<f64>,
    pub enable_local_tsdb: Option<bool>,
    pub local_tsdb_path: Option<String>,
    pub local_tsdb_retention_hours: Option<u64>,
    pub local_tsdb_max_disk_mb: Option<u64>,
    pub log_level: Option<LogLevel>,
    pub orchestrator: Option<OrchestratorConfig>,
    pub efficiency_profile_path: Option<PathBuf>,
    pub enforcement_mode: Option<EnforcementMode>,
    #[serde(default, with = "humantime_serde")]
    pub enforcement_interval: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    pub dampening_interval: Option<Duration>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let mut tags = HashMap::new();
        tags.insert("env".to_string(), "dev".to_string());
        
        Self {
            tags,
            scrape_interval: Duration::from_millis(100), // Fast 100ms default
            
            enable_cpu: true,
            enable_memory: true,
            enable_disk: true,
            enable_network: true,
            enable_ebpf: false,
            
            enable_gpu: true,
            enable_gpu_amd: false,
            enable_gpu_mig: false,
            enable_gpu_events: true,
            gpu_visible_devices: None,
            mig_config_devices: None,
            
            enable_power: true,
            node_power_envelope_watts: None,
            enable_rack_thermals: false,
            
            k8s_mode: false,
            enable_mcp: false,
            
            enable_app: false,
            app_metrics_url: "http://localhost:8000/metrics".to_string(),
            
            listen_address: "0.0.0.0:9100".to_string(),
            
            enable_local_tsdb: false,
            local_tsdb_path: "/tmp/esnode_tsdb".to_string(),
            local_tsdb_retention_hours: 24,
            local_tsdb_max_disk_mb: 512,
            
            orchestrator: None,
            
            efficiency_profile_path: None,
            enforcement_mode: EnforcementMode::Monitor,
            enforcement_interval: Duration::from_secs(5),
            dampening_interval: Duration::from_secs(60),
            
            drivers: Vec::new(),

            log_level: LogLevel::Info,
        }
    }
}

impl AgentConfig {
    pub fn apply_overrides(&mut self, overrides: ConfigOverrides) {
        if let Some(v) = overrides.listen_address { self.listen_address = v; }
        if let Some(v) = overrides.scrape_interval { self.scrape_interval = v; }
        if let Some(v) = overrides.enable_cpu { self.enable_cpu = v; }
        if let Some(v) = overrides.enable_memory { self.enable_memory = v; }
        if let Some(v) = overrides.enable_disk { self.enable_disk = v; }
        if let Some(v) = overrides.enable_network { self.enable_network = v; }
        if let Some(v) = overrides.enable_ebpf { self.enable_ebpf = v; }
        if let Some(v) = overrides.enable_gpu { self.enable_gpu = v; }
        if let Some(v) = overrides.enable_gpu_amd { self.enable_gpu_amd = v; }
        if let Some(v) = overrides.enable_gpu_mig { self.enable_gpu_mig = v; }
        if let Some(v) = overrides.enable_gpu_events { self.enable_gpu_events = v; }
        if let Some(v) = overrides.gpu_visible_devices { self.gpu_visible_devices = Some(v); }
        if let Some(v) = overrides.mig_config_devices { self.mig_config_devices = Some(v); }
        if let Some(v) = overrides.k8s_mode { self.k8s_mode = v; }
        if let Some(v) = overrides.enable_power { self.enable_power = v; }
        if let Some(v) = overrides.node_power_envelope_watts { self.node_power_envelope_watts = Some(v); }
        if let Some(v) = overrides.enable_rack_thermals { self.enable_rack_thermals = v; }
        if let Some(v) = overrides.enable_mcp { self.enable_mcp = v; }
        if let Some(v) = overrides.enable_app { self.enable_app = v; }
        if let Some(v) = overrides.app_metrics_url { self.app_metrics_url = v; }
        if let Some(v) = overrides.enable_local_tsdb { self.enable_local_tsdb = v; }
        if let Some(v) = overrides.local_tsdb_path { self.local_tsdb_path = v; }
        if let Some(v) = overrides.local_tsdb_retention_hours { self.local_tsdb_retention_hours = v; }
        if let Some(v) = overrides.local_tsdb_max_disk_mb { self.local_tsdb_max_disk_mb = v; }
        if let Some(v) = overrides.log_level { self.log_level = v; }
        if let Some(v) = overrides.orchestrator { self.orchestrator = Some(v); }
        if let Some(v) = overrides.efficiency_profile_path { self.efficiency_profile_path = Some(v); }
        if let Some(v) = overrides.enforcement_mode { self.enforcement_mode = v; }
        if let Some(v) = overrides.enforcement_interval { self.enforcement_interval = v; }
        if let Some(v) = overrides.dampening_interval { self.dampening_interval = v; }
    }
}

pub fn load_config(path: Option<PathBuf>) -> Result<AgentConfig, config::ConfigError> {
    let builder = config::Config::builder()
        .add_source(config::Config::try_from(&AgentConfig::default())?)
        .add_source(config::Environment::with_prefix("ESNODE"));

    if let Some(p) = path {
        // Only add file if it exists, otherwise ignore (optional)
        if p.exists() {
             return builder.add_source(config::File::from(p)).build()?.try_deserialize();
        }
    }
    
    // Fallback if no file provided or file doesn't exist
    builder.add_source(config::File::with_name("esnode").required(false))
           .build()?
           .try_deserialize()
}
