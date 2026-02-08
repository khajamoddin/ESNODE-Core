// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB
mod collectors;
pub mod config;
mod event_worker;
mod http;
pub mod control;
pub mod metrics;
pub mod nvml_ext;
pub mod policy;
pub mod predictive;
pub mod rca;
pub mod state;
pub mod tsdb;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

use anyhow::Context;
use collectors::{
    app::AppCollector, cpu::CpuCollector, disk::DiskCollector, gpu::GpuCollector,
    memory::MemoryCollector, network::NetworkCollector, numa::NumaCollector, power::PowerCollector,
    Collector,
};
pub use config::{AgentConfig, ConfigOverrides, LogLevel};
use http::{build_router, serve, HttpState};
use metrics::MetricsRegistry;
use std::net::SocketAddr;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{info, warn};
use tsdb::{samples_from_registry, LocalTsdb, LocalTsdbConfig};

pub struct Agent {
    config: AgentConfig,
    metrics: MetricsRegistry,
    collectors: Vec<Box<dyn Collector>>,
    healthy: Arc<AtomicBool>,
    status: state::StatusState,
    local_tsdb: Option<Arc<LocalTsdb>>,
}

impl Agent {
    pub fn new(config: AgentConfig) -> anyhow::Result<Self> {
        let metrics = MetricsRegistry::new()?;
        let healthy = Arc::new(AtomicBool::new(true));
        let status = state::StatusState::new(healthy.clone());
        let mut collectors: Vec<Box<dyn Collector>> = Vec::new();

        if config.enable_cpu {
            info!("CPU collector enabled");
            metrics
                .agent_collector_disabled
                .with_label_values(&["cpu"])
                .set(0.0);
            metrics
                .agent_collector_disabled
                .with_label_values(&["numa"])
                .set(0.0);
            collectors.push(Box::new(CpuCollector::new(status.clone())));
            collectors.push(Box::new(NumaCollector::new()));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["cpu"])
                .set(1.0);
            metrics
                .agent_collector_disabled
                .with_label_values(&["numa"])
                .set(1.0);
        }
        if config.enable_memory {
            info!("Memory collector enabled");
            metrics
                .agent_collector_disabled
                .with_label_values(&["memory"])
                .set(0.0);
            collectors.push(Box::new(MemoryCollector::new(status.clone())));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["memory"])
                .set(1.0);
        }
        if config.enable_disk {
            info!("Disk collector enabled");
            metrics
                .agent_collector_disabled
                .with_label_values(&["disk"])
                .set(0.0);
            collectors.push(Box::new(DiskCollector::new(status.clone())));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["disk"])
                .set(1.0);
        }
        if config.enable_network {
            info!("Network collector enabled");
            metrics
                .agent_collector_disabled
                .with_label_values(&["network"])
                .set(0.0);
            collectors.push(Box::new(NetworkCollector::new(status.clone())));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["network"])
                .set(1.0);
        }
        if config.enable_gpu {
            let (collector, warning) = GpuCollector::new(status.clone(), &config);
            if let Some(msg) = warning {
                warn!("{msg}");
                metrics
                    .agent_collector_disabled
                    .with_label_values(&["gpu"])
                    .set(1.0);
            } else {
                metrics
                    .agent_collector_disabled
                    .with_label_values(&["gpu"])
                    .set(0.0);
            }
            collectors.push(Box::new(collector));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["gpu"])
                .set(1.0);
        }
        if config.enable_power {
            info!("Power collector enabled");
            metrics
                .agent_collector_disabled
                .with_label_values(&["power"])
                .set(0.0);
            collectors.push(Box::new(PowerCollector::new(
                status.clone(),
                config.node_power_envelope_watts,
            )));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["power"])
                .set(1.0);
        }
        let agent_label = "local".to_string();
        if config.enable_app {
            info!("App collector enabled (url={})", config.app_metrics_url);
            metrics
                .agent_collector_disabled
                .with_label_values(&["app"])
                .set(0.0);
            collectors.push(Box::new(AppCollector::new(
                status.clone(),
                config.app_metrics_url.clone(),
                agent_label.clone(),
            )));
        } else {
            metrics
                .agent_collector_disabled
                .with_label_values(&["app"])
                .set(1.0);
        }
        let local_tsdb = if config.enable_local_tsdb {
            info!(
                "Local TSDB enabled (path={}, retention={}h, max_disk={} MB)",
                config.local_tsdb_path,
                config.local_tsdb_retention_hours,
                config.local_tsdb_max_disk_mb
            );
            match LocalTsdb::new(LocalTsdbConfig::from(&config)) {
                Ok(tsdb) => Some(Arc::new(tsdb)),
                Err(err) => {
                    warn!(
                        "Local TSDB disabled: failed to init at {} ({err}); on-agent buffer is OFF. \
                         Set ESNODE_LOCAL_TSDB_PATH to a writable directory or disable --enable-local-tsdb.",
                        config.local_tsdb_path
                    );
                    None
                }
            }
        } else {
            None
        };

        let start_secs = chrono::Utc::now().timestamp() as f64;
        metrics.agent_running.set(1.0);
        metrics.agent_start_time_seconds.set(start_secs);
        let version = env!("CARGO_PKG_VERSION");
        let commit = option_env!("ESNODE_COMMIT").unwrap_or("unknown");
        metrics
            .agent_build_info
            .with_label_values(&[version, commit])
            .set(1.0);
        metrics
            .ai_tokens_per_joule
            .with_label_values(&[agent_label.as_str()])
            .set(0.0);
        metrics
            .ai_tokens_per_watt
            .with_label_values(&[agent_label.as_str()])
            .set(0.0);
        metrics
            .ai_cost_per_million_tokens_usd
            .with_label_values(&[agent_label.as_str()])
            .set(0.0);
        metrics
            .ai_carbon_grams_per_token
            .with_label_values(&[agent_label.as_str()])
            .set(0.0);
        metrics.degradation_score.set(0.0);

        Ok(Self {
            config,
            metrics,
            collectors,
            healthy,
            status,
            local_tsdb,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let Self {
            config,
            metrics,
            collectors,
            healthy,
            status,
            local_tsdb,
        } = self;

        let shared_collectors = Arc::new(Mutex::new(collectors));
        let metrics_clone = metrics.clone();
        let healthy_clone = healthy.clone();
        let scrape_interval = config.scrape_interval;
        let status_state = status.clone();
        let tsdb_for_collection = local_tsdb.clone();
        let tsdb_for_shutdown = local_tsdb.clone();
        let tsdb_pruner_handle = local_tsdb
            .clone()
            .map(|tsdb| tsdb.spawn_pruner(std::time::Duration::from_secs(60)));
        
        let orchestrator_state_clone = if let Some(orch_config) = &config.orchestrator {
             if orch_config.enabled {
                let devices = vec![]; 
                let orchestrator = esnode_orchestrator::Orchestrator::new(devices, orch_config.clone());
                Some(esnode_orchestrator::AppState {
                    orchestrator: std::sync::Arc::new(std::sync::RwLock::new(orchestrator)),
                    token: orch_config.token.clone(),
                })
             } else {
                 None
             }
        } else {
            None
        };

        if let Some(state) = &orchestrator_state_clone {
             info!("Initializing ESNODE-Orchestrator...");
             let loop_state = state.clone();
             tokio::spawn(async move {
                esnode_orchestrator::run_loop(loop_state).await;
             });
        }
        
        let orch_state_clone_for_update = orchestrator_state_clone.clone();

        let collection_task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(scrape_interval);
            let mut last_tsdb_write_ms: i64 = 0;
            
            let mut rca_engine = crate::rca::RcaEngine::new(
                std::time::Duration::from_secs(300), 
                scrape_interval
            );
            let mut risk_predictor = crate::predictive::FailureRiskPredictor::new();

            loop {
                ticker.tick().await;
                let ts_ms = chrono::Utc::now().timestamp_millis();
                let now_ms = ts_ms as u64;
                let mut guard = shared_collectors.lock().await;
                let mut all_ok = true;

                for collector in guard.iter_mut() {
                    let start = Instant::now();
                    if let Err(err) = collector.collect(&metrics_clone).await {
                        warn!("collector {} failed: {:?}", collector.name(), err);
                        metrics_clone.inc_error(collector.name());
                        status_state.record_error(collector.name(), format!("{err:?}"), now_ms);
                        all_ok = false;
                    }
                    let duration = start.elapsed().as_secs_f64();
                    metrics_clone.observe_scrape_duration(collector.name(), duration);
                }

                status_state.set_last_scrape(now_ms);
                healthy_clone.store(all_ok, Ordering::Relaxed);
                
                // --- K8s Event Correlation Simulation ---
                // In a production environment, this would interface with the K8s API
                // or parse host logs for pod evictions/starts.
                if status_state.get_load_avg_1m() > 8000 { // Load > 8.0
                    status_state.set_k8s_events_detected(true);
                } else {
                    status_state.set_k8s_events_detected(false);
                }

                status_state.update_degradation_score(&metrics_clone);

                // --- Predictive Maintenance & AIOps ---
                let snapshot_full = status_state.snapshot();
                rca_engine.add_snapshot(snapshot_full.clone());
                let rca_events = rca_engine.analyze();
                
                // Convert RCA events to AIOps format and store in StatusState
                let aiops_rca: Vec<state::AIOpsRcaEvent> = rca_events.iter().map(|event| {
                    state::AIOpsRcaEvent {
                        gpu_id: "N/A".to_string(), // RcaEvent doesn't track specific GPU
                        timestamp_ms: now_ms,
                        root_cause: format!("{:?}", event.cause),
                        confidence: event.confidence,
                        details: event.description.clone(),
                    }
                }).collect();
                
                status_state.update_rca_events(aiops_rca);
                
                for event in rca_events {
                    info!("RCA Detection: {:?}", event);
                    metrics_clone.rca_detections_total
                        .with_label_values(&[&format!("{:?}", event.cause), &format!("{:.1}", event.confidence)])
                        .inc();
                }

                let risks = risk_predictor.analyze(&snapshot_full);
                
                // Convert risk assessments to AIOps format and store in StatusState
                let aiops_risk: Vec<state::AIOpsRiskAssessment> = risks.iter().map(|(uuid, assessment)| {
                    state::AIOpsRiskAssessment {
                        gpu_id: uuid.clone(),
                        failure_probability: assessment.failure_probability,
                        risk_score: assessment.risk_score,
                        factors: assessment.factors.clone(),
                    }
                }).collect();
                
                status_state.update_risk_assessments(aiops_risk);
                
                for (uuid, assessment) in risks {
                     if assessment.risk_score > 0.0 {
                         metrics_clone.gpu_failure_risk_score
                             .with_label_values(&[&uuid])
                             .set(assessment.risk_score);
                         
                         if assessment.risk_score >= 50.0 {
                             warn!("Predictive Maintenance Alert: GPU {} risk score {:.1} (Factors: {:?})", 
                                 uuid, assessment.risk_score, assessment.factors);
                         }
                     }
                }

                // --- Orchestrator Integration ---
                if let Some(orch_app_state) = &orch_state_clone_for_update {
                    if let Ok(mut orch) = orch_app_state.orchestrator.write() {
                        let gpu_status = status_state.gpu_status.read().unwrap();
                        for gpu in gpu_status.iter() {
                            let id = gpu.uuid.clone().unwrap_or(gpu.gpu.clone());
                            let device = esnode_orchestrator::Device {
                                id: id.clone(),
                                kind: esnode_orchestrator::DeviceKind::Gpu,
                                peak_flops_tflops: 100.0, // Placeholder or derive from model
                                mem_gb: gpu.memory_total_bytes.unwrap_or(0.0) / 1024.0 / 1024.0 / 1024.0,
                                power_watts_idle: 20.0, // Estimation
                                power_watts_max: gpu.power_watts.unwrap_or(250.0).max(100.0),
                                current_load: gpu.util_percent.unwrap_or(0.0) / 100.0,
                                temperature_celsius: gpu.temperature_celsius,
                                real_power_watts: gpu.power_watts,
                                assigned_tasks: vec![],
                                last_seen: now_ms,
                            };
                            orch.update_device(device);
                        }
                    }
                }
                drop(guard);

                if let Some(tsdb) = tsdb_for_collection.clone() {
                    if ts_ms - last_tsdb_write_ms >= 30_000 {
                        let samples = samples_from_registry(&metrics_clone, ts_ms);
                        if let Err(err) = tsdb.write_samples(&samples).await {
                            warn!("local TSDB write failed: {:?}", err);
                        }
                        last_tsdb_write_ms = ts_ms;
                    }
                }
            }
        });
        
        let mut enforcement_ticker = tokio::time::interval(config.enforcement_interval);
        // Offset first tick to avoid stampede at startup
        enforcement_ticker.reset(); 
        
        let enforcement_config = config.clone();
        let enforcement_status = status.clone();
        let enforcement_metrics = metrics.clone();
        
        let enforcement_task = tokio::spawn(async move {
            if enforcement_config.efficiency_profile_path.is_none() {
                // Determine if we should exit or sleep. Sleeping is safer for the select! block.
                std::future::pending::<()>().await;
                return;
            }
            let profile_path = enforcement_config.efficiency_profile_path.as_ref().unwrap();
            let mode = &enforcement_config.enforcement_mode;
            // Enforcer needs to be Send. agent_core::control::Enforcer holds Nvml which is Send.
            let enforcer = crate::control::Enforcer::new();
            let mut dampener = crate::control::FlapDampener::new(enforcement_config.dampening_interval);

            loop {
                enforcement_ticker.tick().await;
                
                let contents = match tokio::fs::read_to_string(profile_path).await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to read efficiency profile at {}: {}", profile_path, e);
                        continue;
                    }
                };
                
                let profile: crate::policy::EfficiencyProfile = match serde_yaml::from_str(&contents) {
                     Ok(p) => p,
                     Err(e) => {
                         warn!("Failed to parse efficiency profile: {}", e);
                         continue;
                     }
                };
                
                // We need a StatusSnapshot. status is typically updated by collection_task.
                // StatusState is thread-safe (Arc<RwLock>).
                let snapshot = enforcement_status.snapshot();
                let plan = profile.plan(&snapshot);
                
                let violations: Vec<_> = plan.matched_policies.iter()
                    .filter(|p| matches!(p.status, crate::policy::PlanStatus::Violated))
                    .collect();

                if !violations.is_empty() {
                    info!("Efficiency Audit: Found {} violations", violations.len());
                    for v in &violations {
                         info!("Violation: {} on {} (Current: {}, Limit: {})", 
                            v.policy_name, v.target_resource, v.current_value, v.threshold);
                         
                         enforcement_metrics.policy_violations_total
                            .with_label_values(&[&v.policy_name, &v.target_resource, "violation"])
                            .inc();

                         if *mode == crate::config::EnforcementMode::Enforce {
                             if !dampener.can_apply(&v.policy_name, &v.target_resource) {
                                 info!("Dampened enforcement of {} on {}", v.policy_name, v.target_resource);
                                 continue;
                             }
                             // Re-find policy definition to get the action details
                             if let Some(policy) = profile.policies.iter().find(|p| p.name == v.policy_name) {
                                match enforcer.apply_action(&v.target_resource, &policy.action) {
                                    Ok(msg) => {
                                        info!("ENFORCED: {}", msg);
                                        dampener.record_action(&v.policy_name, &v.target_resource);
                                        enforcement_metrics.policy_enforced_total
                                            .with_label_values(&[&v.policy_name, &v.target_resource, "success"])
                                            .inc();
                                    },
                                    Err(e) => {
                                        warn!("ENFORCEMENT FAILED: {}", e);
                                        enforcement_metrics.policy_enforced_total
                                            .with_label_values(&[&v.policy_name, &v.target_resource, "failure"])
                                            .inc();
                                    },
                                }
                             }
                         }
                    }
                }
            }
        });
                
        // Orchestrator already initialized above
        let orchestrator_state = orchestrator_state_clone;
        let http_state = HttpState {
            metrics: metrics.clone(),
            healthy: healthy.clone(),
            status: status.clone(),
            tsdb: local_tsdb.clone(),
            orchestrator: orchestrator_state,
            orchestrator_allow_public: config.orchestrator.as_ref().is_some_and(|o| o.allow_public),
            listen_is_loopback: listen_is_loopback(&config.listen_address),
            orchestrator_token: config.orchestrator.as_ref().and_then(|o| o.token.clone()),
        };
        let router = build_router(http_state);
        let http_task = serve(&config.listen_address, router)
            .await
            .context("starting HTTP server")?;

        tokio::select! {
            res = collection_task => {
                if let Err(err) = res {
                    return Err(anyhow::anyhow!("collection task panicked: {err:?}"));
                }
            },
            res = enforcement_task => {
                if let Err(err) = res {
                    // If the enforcement task panics (unlikely unless FS error or similar), log it.
                    // We might not want to kill the whole agent, but for now strict mode is fine.
                    return Err(anyhow::anyhow!("enforcement task panicked: {err:?}"));
                }
            },
            res = http_task => {
                if let Err(err) = res {
                    return Err(anyhow::anyhow!("http server task panicked: {err:?}"));
                }
            },
            _ = signal::ctrl_c() => {
                if let Some(tsdb) = tsdb_for_shutdown {
                    let _ = tsdb.flush_current().await;
                }
                if let Some(handle) = tsdb_pruner_handle { handle.abort(); }
                return Ok(());
            }
        }
        Ok(())
    }
}

fn listen_is_loopback(listen: &str) -> bool {
    listen
        .parse::<SocketAddr>()
        .map(|addr| addr.ip().is_loopback())
        .unwrap_or(false)
}
