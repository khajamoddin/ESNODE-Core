// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB
mod client;
mod console;

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use agent_core::{Agent, AgentConfig, ConfigOverrides, LogLevel};
use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use client::AgentClient;
use console::{run_console};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "esnode-core", version, about = "GPU-aware host metrics exporter")]
struct Cli {
    /// Optional path to configuration file (TOML). Also read from `ESNODE_CONFIG`.
    #[arg(long, env = "ESNODE_CONFIG")]
    config: Option<PathBuf>,

    /// Disable ANSI colors (applies to TUI + non-interactive output).
    #[arg(long)]
    no_color: bool,

    /// Address for HTTP listener, e.g. 0.0.0.0:9100
    #[arg(long, env = "ESNODE_LISTEN_ADDRESS")]
    listen_address: Option<String>,

    /// Scrape interval (e.g. 5s, 1m)
    #[arg(long, env = "ESNODE_SCRAPE_INTERVAL")]
    scrape_interval: Option<String>,

    /// Enable or disable CPU collector
    #[arg(long, env = "ESNODE_ENABLE_CPU")]
    enable_cpu: Option<bool>,

    /// Enable or disable memory collector
    #[arg(long, env = "ESNODE_ENABLE_MEMORY")]
    enable_memory: Option<bool>,

    /// Enable or disable disk collector
    #[arg(long, env = "ESNODE_ENABLE_DISK")]
    enable_disk: Option<bool>,

    /// Enable or disable network collector
    #[arg(long, env = "ESNODE_ENABLE_NETWORK")]
    enable_network: Option<bool>,
    
    /// Enable or disable eBPF high-frequency performance collector
    #[arg(long, env = "ESNODE_ENABLE_EBPF")]
    enable_ebpf: Option<bool>,

    /// Enable or disable GPU collector
    #[arg(long, env = "ESNODE_ENABLE_GPU")]
    enable_gpu: Option<bool>,

    /// Enable or disable AMD GPU collector (`ROCm`)
    #[arg(long, env = "ESNODE_ENABLE_GPU_AMD")]
    enable_gpu_amd: Option<bool>,

    /// Enable or disable MIG telemetry (requires GPU feature; guarded).
    #[arg(long, env = "ESNODE_ENABLE_GPU_MIG")]
    enable_gpu_mig: Option<bool>,

    /// Enable or disable GPU event polling (XID/ECC/retire/throttle).
    #[arg(long, env = "ESNODE_ENABLE_GPU_EVENTS")]
    enable_gpu_events: Option<bool>,

    /// Optional filter for visible GPUs (UUIDs or indices, comma separated).
    #[arg(long, env = "ESNODE_GPU_VISIBLE_DEVICES")]
    gpu_visible_devices: Option<String>,

    /// Optional filter for GPUs where MIG can be managed (UUIDs or indices).
    #[arg(long, env = "ESNODE_MIG_CONFIG_DEVICES")]
    mig_config_devices: Option<String>,

    /// Enable Kubernetes-compatible resource/label naming.
    #[arg(long, env = "ESNODE_K8S_MODE")]
    k8s_mode: Option<bool>,

    /// Enable or disable power collector (CPU/package/hwmon/BMC if available)
    #[arg(long, env = "ESNODE_ENABLE_POWER")]
    enable_power: Option<bool>,

    /// Optional node power envelope in watts (for breach flag)
    #[arg(long, env = "ESNODE_NODE_POWER_ENVELOPE_WATTS")]
    node_power_envelope_watts: Option<f64>,

    /// Enable lightweight on-agent TSDB buffer (JSONL-backed).
    #[arg(long, env = "ESNODE_ENABLE_LOCAL_TSDB")]
    enable_local_tsdb: Option<bool>,

    /// Filesystem path for the agent TSDB (when enabled).
    #[arg(long, env = "ESNODE_LOCAL_TSDB_PATH")]
    local_tsdb_path: Option<String>,

    /// Retention window for the on-agent TSDB (hours).
    #[arg(long, env = "ESNODE_LOCAL_TSDB_RETENTION_HOURS")]
    local_tsdb_retention_hours: Option<u64>,

    /// Maximum disk budget for the on-agent TSDB (MB).
    #[arg(long, env = "ESNODE_LOCAL_TSDB_MAX_DISK_MB")]
    local_tsdb_max_disk_mb: Option<u64>,

    /// Enable ESNODE-Orchestrator (Autonomous features)
    #[arg(long, env = "ESNODE_ENABLE_ORCHESTRATOR")]
    pub enable_orchestrator: Option<bool>,



    /// Enable App/Model Awareness collector
    #[arg(long, env = "ESNODE_ENABLE_APP")]
    pub enable_app: Option<bool>,

    /// URL for application metrics (e.g. http://localhost:8000/metrics)
    #[arg(long, env = "ESNODE_APP_METRICS_URL")]
    pub app_metrics_url: Option<String>,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, env = "ESNODE_LOG_LEVEL")]
    log_level: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Default daemon mode: run the agent and expose /metrics.
    Daemon,
    /// One-shot node summary (CPU/mem/GPU/power).
    Status,
    /// Dump metrics snapshot in human-readable form.
    Metrics {
        #[arg(value_enum, default_value_t = MetricsProfile::Basic)]
        profile: MetricsProfile,
    },
    /// Enable a metric set and persist it to config.
    EnableMetricSet {
        #[arg(value_enum)]
        set: MetricSet,
    },
    /// Disable a metric set and persist it to config.
    DisableMetricSet {
        #[arg(value_enum)]
        set: MetricSet,
    },
    /// List metric profiles and what they include.
    Profiles,
    /// Run quick self-check for GPU API, permissions, filesystem, etc.
    Diagnostics,
    /// Launch the AS/400-inspired console UI.
    Cli,
    /// View or modify agent config.
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },
    /// Plan an efficiency profile against the current node status.
    Plan {
        /// Path to the efficiency profile (YAML).
        file: PathBuf,
    },
    /// Enforce an efficiency profile (Apply actions).
    Apply {
        /// Path to the efficiency profile (YAML).
        file: PathBuf,
        /// Skip interactive confirmation.
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show,
    Set {
        /// Key-value pair (key=value) to persist into esnode.toml.
        key_value: String,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum MetricsProfile {
    Basic,
    Full,
    GpuOnly,
    PowerOnly,
}



#[derive(Copy, Clone, Debug, ValueEnum)]
enum MetricSet {
    Host,
    Gpu,
    Power,
    Mcp,
    App,
    All,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = resolve_config_path(&cli);
    
    // Load base config (Defaults + Env)
    let mut config = agent_core::config::load_config(Some(config_path.clone()))
        .context("failed to load base configuration")?;

    // Apply CLI overrides which take precedence
    let cli_overrides = cli_to_overrides(&cli)?;
    config.apply_overrides(cli_overrides);

    match cli.command.as_ref().unwrap_or(&Command::Daemon) {
        Command::Daemon => {
            init_tracing(&config);
            tracing::info!("Starting ESNODE-Core with config: {:?}", config);
            
            // Instantiate drivers from config
            let drivers = instantiate_drivers(&config)?;
            
            let agent = Agent::new(config, drivers)?;
            agent.run().await
        }
        Command::Status => {
            let client = AgentClient::new(&config.listen_address);
            command_status(&client, cli.no_color)
        }
        Command::Metrics { profile } => {
            let client = AgentClient::new(&config.listen_address);
            command_metrics(&client, *profile)
        }
        Command::EnableMetricSet { set } => command_toggle_metric_set(&config_path, *set, true),
        Command::DisableMetricSet { set } => command_toggle_metric_set(&config_path, *set, false),
        Command::Profiles => {
            command_profiles();
            Ok(())
        }
        Command::Diagnostics => {
            let client = AgentClient::new(&config.listen_address);
            command_diagnostics(&client)
        }
        Command::Cli => {
            let client = AgentClient::new(&config.listen_address);

            run_console(
                &client,
                cli.no_color,
                config_path.clone(),
                config.clone(),
            )
        }

        Command::Config { action } => match action {
            ConfigCommand::Show => command_config_show(&config_path, &config),
            ConfigCommand::Set { key_value } => command_config_set(&config_path, key_value),
        },
        Command::Plan { file } => {
            let client = AgentClient::new(&config.listen_address);
            command_plan(&client, file)
        },
        Command::Apply { file, yes } => {
            let client = AgentClient::new(&config.listen_address);
            command_apply(&client, file, *yes)
        },
    }
}

fn resolve_config_path(cli: &Cli) -> PathBuf {
    if let Some(path) = &cli.config {
        path.clone()
    } else {
        PathBuf::from("esnode.toml")
    }
}

fn load_config_file(path: &Path) -> Result<ConfigOverrides> {
    let contents = fs::read_to_string(path)?;
    let overrides: ConfigOverrides = toml::from_str(&contents)?;
    Ok(overrides)
}



fn cli_to_overrides(cli: &Cli) -> Result<ConfigOverrides> {
    let orchestrator = if cli.enable_orchestrator.unwrap_or(false) {
        Some(agent_core::config::OrchestratorConfig {
            enabled: true,
            ..Default::default()
        })
    } else {
        None
    };

    Ok(ConfigOverrides {
        listen_address: cli.listen_address.clone(),
        scrape_interval: parse_duration(cli.scrape_interval.as_deref())?,
        enable_cpu: cli.enable_cpu,
        enable_memory: cli.enable_memory,
        enable_disk: cli.enable_disk,
        enable_network: cli.enable_network,
        enable_ebpf: cli.enable_ebpf,
        enable_gpu: cli.enable_gpu,
        enable_gpu_amd: cli.enable_gpu_amd,
        enable_gpu_mig: cli.enable_gpu_mig,
        enable_gpu_events: cli.enable_gpu_events,
        gpu_visible_devices: cli.gpu_visible_devices.clone(),
        mig_config_devices: cli.mig_config_devices.clone(),
        k8s_mode: cli.k8s_mode,
        enable_power: cli.enable_power,
        enable_mcp: None,
        enable_app: cli.enable_app,
        app_metrics_url: cli.app_metrics_url.clone(),
        enable_rack_thermals: None,
        node_power_envelope_watts: cli.node_power_envelope_watts,
        enable_local_tsdb: cli.enable_local_tsdb,
        local_tsdb_path: cli.local_tsdb_path.clone(),
        local_tsdb_retention_hours: cli.local_tsdb_retention_hours,
        local_tsdb_max_disk_mb: cli.local_tsdb_max_disk_mb,
        log_level: parse_log_level(cli.log_level.as_deref())?,
        orchestrator,
        efficiency_profile_path: None,
        enforcement_mode: None,
        enforcement_interval: None,
        dampening_interval: None,
    })
}

fn parse_duration(input: Option<&str>) -> Result<Option<Duration>> {
    if let Some(value) = input {
        let duration = humantime::parse_duration(value)?;
        Ok(Some(duration))
    } else {
        Ok(None)
    }
}

fn parse_log_level(input: Option<&str>) -> Result<Option<LogLevel>> {
    if let Some(level) = input {
        let parsed = match level.to_ascii_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" | "warning" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            other => bail!("unknown log level {other}"),
        };
        Ok(Some(parsed))
    } else {
        Ok(None)
    }
}

fn instantiate_drivers(config: &AgentConfig) -> Result<Vec<Box<dyn agent_core::drivers::Driver>>> {
    use agent_core::drivers::{Driver, SensorType};
    use std::net::SocketAddr;
    
    let mut drivers: Vec<Box<dyn Driver>> = Vec::new();
    
    for driver_cfg in &config.drivers {
        match driver_cfg.protocol.as_str() {
            "modbus" => {
                let addr: SocketAddr = driver_cfg.target.parse()
                    .with_context(|| format!("Invalid Modbus target address: {}", driver_cfg.target))?;
                
                let slave_id = driver_cfg.params.get("slave_id")
                    .and_then(|s| s.parse::<u8>().ok())
                    .unwrap_or(1);
                
                let mut register_mappings = Vec::new();
                
                // Parse registers from params (format: "40001:temperature:2:0.1")
                if let Some(register_str) = driver_cfg.params.get("registers") {
                    for reg_def in register_str.split(',') {
                        let parts: Vec<&str> = reg_def.split(':').collect();
                        if parts.len() >= 4 {
                            let address = parts[0].parse::<u16>()?;
                            let sensor_type = match parts[1].to_lowercase().as_str() {
                                "temperature" => SensorType::Temperature,
                                "pressure" => SensorType::Pressure,
                                "voltage" => SensorType::Voltage,
                                "current" => SensorType::Current,
                                "power" => SensorType::Power,
                                _ => SensorType::Other,
                            };
                            let count = parts[2].parse::<u16>()?;
                            let scale = parts[3].parse::<f64>()?;
                            
                            register_mappings.push(esnode_modbus::RegisterMapping {
                                address,
                                count,
                                sensor_type,
                                unit: "raw".to_string(),
                                scale,
                            });
                        }
                    }
                }
                
                // Default mapping if none specified
                if register_mappings.is_empty() {
                    register_mappings.push(esnode_modbus::RegisterMapping {
                        address: 40001,
                        count: 2,
                        sensor_type: SensorType::Temperature,
                        unit: "celsius".to_string(),
                        scale: 0.1,
                    });
                }
                
                drivers.push(Box::new(esnode_modbus::ModbusDriver::new(
                    driver_cfg.id.clone(),
                    addr,
                    slave_id,
                    register_mappings,
                )));
            }
            "dnp3" => {
                let addr: SocketAddr = driver_cfg.target.parse()
                    .with_context(|| format!("Invalid DNP3 target address: {}", driver_cfg.target))?;
                
                let local_addr = driver_cfg.params.get("local_addr")
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(1);
                    
                let remote_addr = driver_cfg.params.get("remote_addr")
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(1024);
                
                let dnp3_config = esnode_dnp3::Dnp3Config {
                    local_addr,
                    remote_addr,
                    integrity_interval_ms: 5000,
                };
                
                drivers.push(Box::new(esnode_dnp3::Dnp3Driver::new(
                    driver_cfg.id.clone(),
                    addr,
                    dnp3_config,
                )));
            }
            "snmp" => {
                let addr: SocketAddr = driver_cfg.target.parse()
                    .with_context(|| format!("Invalid SNMP target address: {}", driver_cfg.target))?;
                
                let community = driver_cfg.params.get("community")
                    .unwrap_or(&"public".to_string())
                    .clone();
                    
                let oids = driver_cfg.params.get("oids")
                    .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
                    .unwrap_or_else(|| vec!["1.3.6.1.2.1.1.1.0".to_string()]);
                
                let snmp_config = esnode_snmp::SnmpConfig {
                    target: addr,
                    community,
                    oids,
                };
                
                drivers.push(Box::new(esnode_snmp::SnmpDriver::new(
                    driver_cfg.id.clone(),
                    snmp_config,
                )));
            }
            "mqtt" => {
                let broker = driver_cfg.target.split(':').next()
                    .unwrap_or("localhost")
                    .to_string();
                
                let port = driver_cfg.target.split(':').nth(1)
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(1883);
                
                let client_id = driver_cfg.params.get("client_id")
                    .cloned()
                    .unwrap_or_else(|| format!("esnode-{}", driver_cfg.id));
                
                let username = driver_cfg.params.get("username").cloned();
                let password = driver_cfg.params.get("password").cloned();
                
                let topics = driver_cfg.params.get("topics")
                    .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                    .unwrap_or_else(|| vec!["sensors/#".to_string()]);
                
                let qos = driver_cfg.params.get("qos")
                    .and_then(|s| s.parse::<u8>().ok())
                    .unwrap_or(1);
                
                // Parse topic mappings from params
                let mut topic_mappings = Vec::new();
                
                // Format: "mappings=topic:sensor_type:unit:value_path:scale,..."
                if let Some(mappings_str) = driver_cfg.params.get("mappings") {
                    for mapping_def in mappings_str.split(',') {
                        let parts: Vec<&str> = mapping_def.split(':').collect();
                        if parts.len() >= 4 {
                            let sensor_type = match parts[1].to_lowercase().as_str() {
                                "temperature" => SensorType::Temperature,
                                "pressure" => SensorType::Pressure,
                                "voltage" => SensorType::Voltage,
                                "current" => SensorType::Current,
                                "power" => SensorType::Power,
                                "energy" => SensorType::Energy,
                                "frequency" => SensorType::Frequency,
                                _ => SensorType::Other,
                            };
                            
                            let scale = parts.get(4)
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(1.0);
                            
                            topic_mappings.push(esnode_mqtt::TopicMapping::new(
                                parts[0].to_string(),
                                sensor_type,
                                parts[2].to_string(),
                                parts[3].to_string(),
                                scale,
                            ));
                        }
                    }
                }
                
                // Default mapping if none specified
                if topic_mappings.is_empty() {
                    topic_mappings.push(esnode_mqtt::TopicMapping::new(
                        "sensors/#".to_string(),
                        SensorType::Temperature,
                        "celsius".to_string(),
                        "value".to_string(),
                        1.0,
                    ));
                }
                
                let mqtt_config = esnode_mqtt::MqttConfig {
                    broker,
                    port,
                    client_id,
                    username,
                    password,
                    topics,
                    qos,
                    use_tls: driver_cfg.params.get("use_tls")
                        .and_then(|s| s.parse::<bool>().ok())
                        .unwrap_or(false),
                    ca_cert_path: driver_cfg.params.get("ca_cert_path").cloned(),
                    client_cert_path: driver_cfg.params.get("client_cert_path").cloned(),
                    client_key_path: driver_cfg.params.get("client_key_path").cloned(),
                    topic_mappings,
                };
                
                drivers.push(Box::new(esnode_mqtt::MqttDriver::new(
                    driver_cfg.id.clone(),
                    mqtt_config,
                )));
            }
            unknown => {
                bail!("Unknown driver protocol: {}", unknown);
            }
        }
    }
    
    Ok(drivers)
}

fn init_tracing(config: &AgentConfig) {
    let env_filter =
        EnvFilter::from_default_env().add_directive(config.log_level.as_tracing().into());
    let subscriber = fmt().with_env_filter(env_filter).finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn command_status(client: &AgentClient, no_color: bool) -> Result<()> {
    use std::fmt::Write;
    let _ = no_color;
    let snapshot = client.fetch_status()?;
    let mut out = String::new();
    out.push_str("Node status\n");
    out.push_str("ESNODE status (basic profile)\n");
    writeln!(
        &mut out,
        "  Healthy: {}",
        if snapshot.healthy { "yes" } else { "no" }
    )?;
    writeln!(&mut out, "  Load 1m: {:.2}", snapshot.load_avg_1m)?;
    writeln!(
        &mut out,
        "  Node power: {}",
        snapshot
            .node_power_watts
            .map_or_else(|| "n/a".to_string(), |v| format!("{v:.1} W"))
    )?;
    writeln!(&mut out, "  GPUs: {} detected", snapshot.gpus.len())?;
    let avg_util = average(
        &snapshot
            .gpus
            .iter()
            .filter_map(|g| g.util_percent)
            .collect::<Vec<_>>(),
    );
    let avg_power = average(
        &snapshot
            .gpus
            .iter()
            .filter_map(|g| g.power_watts)
            .collect::<Vec<_>>(),
    );
    writeln!(&mut out, "    Avg util: {:.1}%", avg_util.unwrap_or(0.0))?;
    writeln!(&mut out, "    Avg power: {:.1} W", avg_power.unwrap_or(0.0))?;
    if !snapshot.last_errors.is_empty() {
        out.push_str("  Recent errors:\n");
        for err in &snapshot.last_errors {
            writeln!(
                &mut out,
                "    [{}] {} ({})",
                err.collector, err.message, err.unix_ms
            )?;
        }
    }
    print!("{out}");
    Ok(())
}

fn average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn command_metrics(client: &AgentClient, profile: MetricsProfile) -> Result<()> {
    println!("ESNODE metrics snapshot (profile: {profile:?})");
    match client.fetch_metrics_text() {
        Ok(body) => {
            println!("{body}");
            Ok(())
        }
        Err(err) => bail!("unable to fetch /metrics: {err}"),
    }
}

fn command_profiles() {
    println!("Available profiles:");
    println!("  basic     -> host + gpu core + power");
    println!("  full      -> everything (host, gpu, power, mcp, app)");
    println!("  gpu-only  -> GPU core + power only");
    println!("  power-only-> host power + gpu power");
}

fn command_toggle_metric_set(path: &Path, set: MetricSet, enable: bool) -> Result<()> {
    let mut config = AgentConfig::default();
    if path.exists() {
        let file_overrides = load_config_file(path)?;
        config.apply_overrides(file_overrides);
    }
    
    match set {
        MetricSet::Host => {
            config.enable_cpu = enable;
            config.enable_memory = enable;
            config.enable_disk = enable;
            config.enable_network = enable;
        }
        MetricSet::Gpu => {
            config.enable_gpu = enable;
        }
        MetricSet::Power => {
            config.enable_power = enable;
        }
        MetricSet::Mcp => {
            config.enable_mcp = enable;
        }
        MetricSet::App => {
            config.enable_app = enable;
        }
        MetricSet::All => {
            config.enable_cpu = enable;
            config.enable_memory = enable;
            config.enable_disk = enable;
            config.enable_network = enable;
            config.enable_gpu = enable;
            config.enable_power = enable;
            config.enable_mcp = enable;
            config.enable_app = enable;
            config.enable_rack_thermals = enable;
        }
    }

    persist_config(path, &config)?;
    println!(
        "{} metric set {:?} in {}",
        if enable { "Enabled" } else { "Disabled" },
        set,
        path.display()
    );
    Ok(())
}

fn persist_config(path: &Path, config: &AgentConfig) -> Result<()> {
    let contents = toml::to_string_pretty(config)?;
    fs::write(path, contents)?;
    Ok(())
}

fn command_diagnostics(client: &AgentClient) -> Result<()> {
    println!("Running ESNODE diagnostics...");
    match client.fetch_status() {
        Ok(status) => {
            println!("  Agent reachable at {}", client.base_url());
            println!(
                "  GPU status entries: {} ({} recent errors)",
                status.gpus.len(),
                status.last_errors.len()
            );
            println!(
                "  Node power: {}",
                status
                    .node_power_watts
                    .map_or_else(|| "n/a".to_string(), |v| format!("{v:.1} W"))
            );
            Ok(())
        }
        Err(err) => bail!("agent not reachable: {err}"),
    }
}

fn command_config_show(path: &Path, effective: &AgentConfig) -> Result<()> {
    println!("Config path: {}", path.display());
    println!("{}", toml::to_string_pretty(effective)?);
    Ok(())
}

fn command_config_set(path: &Path, pair: &str) -> Result<()> {
    let (key, value) = pair
        .split_once('=')
        .ok_or_else(|| anyhow!("use key=value syntax"))?;
    let mut config = AgentConfig::default();
    if path.exists() {
        let file_overrides = load_config_file(path)?;
        config.apply_overrides(file_overrides);
    }
    apply_config_kv(&mut config, key, value)?;
    persist_config(path, &config)?;
    println!("Updated {} in {}", key, path.display());
    Ok(())
}

fn apply_config_kv(config: &mut AgentConfig, key: &str, val: &str) -> Result<()> {
    match key {
        "listen_address" => config.listen_address = val.to_string(),
        "scrape_interval" => config.scrape_interval = parse_duration(Some(val))?.unwrap(),
        "enable_cpu" => config.enable_cpu = val.parse()?,
        "enable_memory" => config.enable_memory = val.parse()?,
        "enable_disk" => config.enable_disk = val.parse()?,
        "enable_network" => config.enable_network = val.parse()?,
        "enable_gpu" => config.enable_gpu = val.parse()?,
        "enable_power" => config.enable_power = val.parse()?,
        "enable_mcp" => config.enable_mcp = val.parse()?,
        "enable_app" => config.enable_app = val.parse()?,
        "enable_rack_thermals" => config.enable_rack_thermals = val.parse()?,
        "node_power_envelope_watts" => config.node_power_envelope_watts = Some(val.parse()?),
        "log_level" => config.log_level = parse_log_level(Some(val))?.unwrap(),
        other => bail!("unknown config key {other}"),
    }
    Ok(())
}

fn command_plan(client: &AgentClient, profile_path: &Path) -> Result<()> {
    let contents = fs::read_to_string(profile_path)
        .with_context(|| format!("failed to read profile {}", profile_path.display()))?;
    
    let profile: agent_core::policy::EfficiencyProfile = serde_yaml::from_str(&contents)
        .with_context(|| "failed to parse efficiency profile YAML")?;

    println!("Refreshing state from agent at {}...", client.base_url());
    let status = client.fetch_status()
        .with_context(|| "failed to fetch current status from agent")?;

    println!("Analyzed {} GPUs.", status.gpus.len());
    
    let result = profile.plan(&status);
    
    println!("\nPlan: {} policies to check for profile '{}'.\n", result.matched_policies.len(), result.profile_name);
    
    let mut violations = 0;
    
    for plan in result.matched_policies {
        let symbol = match plan.status {
            agent_core::policy::PlanStatus::Satisfied => "✅",
            agent_core::policy::PlanStatus::Violated => "❌",
            agent_core::policy::PlanStatus::Skipped => "⏭️",
        };
        
        println!("{} Policy \"{}\" on {}:", symbol, plan.policy_name, plan.target_resource);
        println!("    Current: {} | Limit: {}", plan.current_value, plan.threshold);
        
        if let Some(action) = plan.computed_action.clone() {
            println!("    -> PLAN ACTION: {}", action);
            violations += 1;
        }
        println!();
    }
    
    if violations > 0 {
        println!("⚠️  Plan found {} violations that would be corrected.", violations);
    } else {
        println!("✨ No violations found. Cluster is efficient.");
    }

    Ok(())
}

fn command_apply(client: &AgentClient, profile_path: &Path, yes: bool) -> Result<()> {
    let contents = fs::read_to_string(profile_path)
        .with_context(|| format!("failed to read profile {}", profile_path.display()))?;
    
    // We need to import the EfficiencyProfile struct. Since agent-core exposes it in policy
    // but agent-bin depends on agent-core, we can access it.
    let profile: agent_core::policy::EfficiencyProfile = serde_yaml::from_str(&contents)
        .with_context(|| "failed to parse efficiency profile YAML")?;

    println!("Refreshing state from agent at {}...", client.base_url());
    let status = client.fetch_status()
        .with_context(|| "failed to fetch current status from agent")?;

    println!("Analyzed {} GPUs.", status.gpus.len());
    
    let result = profile.plan(&status);
    
    // Filter for violations. Note: plan.status is an Enum so we need to match carefully.
    let violations: Vec<_> = result.matched_policies.iter().filter(|p| {
        matches!(p.status, agent_core::policy::PlanStatus::Violated)
    }).collect();

    if violations.is_empty() {
        println!("✨ No violations found in profile '{}'. Nothing to apply.", profile.metadata.name);
        return Ok(());
    }

    println!("\n⚠️  Found {} violations that require action:", violations.len());
    for plan in &violations {
        println!("❌ Policy \"{}\" on {}:", plan.policy_name, plan.target_resource);
        println!("    Current: {} | Limit: {}", plan.current_value, plan.threshold);
        if let Some(action) = &plan.computed_action {
             println!("    -> PROPOSED ACTION: {}", action);
        }
        println!();
    }
    
    if !yes {
        use std::io::{self, Write};
        print!("\nDo you want to enforce these actions? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("Applying efficiency profile '{}'...", profile.metadata.name);
    
    // Instantiate Enforcer
    let enforcer = agent_core::control::Enforcer::new();
    let mut applied_count = 0;
    
    for plan in violations {
        // Find defining policy
        if let Some(policy) = profile.policies.iter().find(|p| p.name == plan.policy_name) {
             match enforcer.apply_action(&plan.target_resource, &policy.action) {
                Ok(msg) => {
                    println!("✅ Applied on {}: {}", plan.target_resource, msg);
                    applied_count += 1;
                },
                Err(e) => {
                    println!("❌ Failed to apply policy '{}' on {}: {}", plan.policy_name, plan.target_resource, e);
                }
             }
        } else {
            println!("❌ Error: Could not find policy definition for '{}'", plan.policy_name);
        }
    }

    println!("\nSummary: {} actions applied successfully.", applied_count);
    Ok(())
}




#[cfg(test)]
mod tests {
    use super::{cli_to_overrides, Cli, Command, MetricSet, MetricsProfile};
    use clap::Parser;

    #[test]
    fn cli_parses_status_command() {
        let cli = Cli::parse_from(["esnode-core", "status"]);
        assert!(matches!(cli.command, Some(Command::Status)));
    }

    #[test]
    fn cli_parses_metrics_command_with_profile() {
        let cli = Cli::parse_from(["esnode-core", "metrics", "full"]);
        match cli.command {
            Some(Command::Metrics { profile }) => assert!(matches!(profile, MetricsProfile::Full)),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn cli_overrides_enable_flags() {
        let cli = Cli::parse_from([
            "esnode-core",
            "--enable-cpu",
            "false",
            "--enable-network",
            "false",
            "enable-metric-set",
            "gpu",
        ]);
        let overrides = cli_to_overrides(&cli).unwrap();
        assert_eq!(overrides.enable_cpu, Some(false));
        assert_eq!(overrides.enable_network, Some(false));
        match cli.command {
            Some(Command::EnableMetricSet { set }) => assert!(matches!(set, MetricSet::Gpu)),
            _ => panic!("expected enable-metric-set"),
        }
    }
}
