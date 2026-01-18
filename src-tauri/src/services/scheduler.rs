use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tokio::sync::RwLock;
use crate::services::{Aggregator, Notifier, Cache};
use tracing::{info, error};

pub struct Scheduler {
    aggregator: Arc<Aggregator>,
    notifier: Arc<Notifier>,
    cache: Arc<Cache>,
    interval_seconds: Arc<RwLock<u64>>,
    running: Arc<RwLock<bool>>,
}

impl Scheduler {
    pub fn new(
        aggregator: Arc<Aggregator>,
        notifier: Arc<Notifier>,
        cache: Arc<Cache>,
        interval_seconds: u64,
    ) -> Self {
        Self {
            aggregator,
            notifier,
            cache,
            interval_seconds: Arc::new(RwLock::new(interval_seconds)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) {
        {
            let mut running = self.running.write().await;
            if *running {
                return; // Already running
            }
            *running = true;
        }

        info!("Starting scheduler");

        // Run immediately on start
        self.run_fetch_cycle().await;

        // Then run on interval
        let aggregator = self.aggregator.clone();
        let notifier = self.notifier.clone();
        let cache = self.cache.clone();
        let interval = self.interval_seconds.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                let interval_secs = *interval.read().await;
                time::sleep(Duration::from_secs(interval_secs)).await;

                if !*running.read().await {
                    break;
                }

                // Fetch quotas
                let quotas = aggregator.fetch_all_quotas().await;

                // Update cache
                for quota in quotas {
                    // Check notifications
                    if let Err(e) = notifier.check_and_notify(&quota).await {
                        error!("Notification check failed: {}", e);
                    }

                    cache.set(quota.account_id.clone(), quota).await;
                }

                info!("Completed scheduled fetch cycle");
            }
        });
    }

    #[allow(dead_code)]
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopped scheduler");
    }

    pub async fn set_interval(&self, seconds: u64) {
        let mut interval = self.interval_seconds.write().await;
        *interval = seconds;
        info!("Updated scheduler interval to {} seconds", seconds);
    }

    pub async fn run_fetch_cycle(&self) {
        info!("Running manual fetch cycle");

        let quotas = self.aggregator.fetch_all_quotas().await;

        for quota in quotas {
            if let Err(e) = self.notifier.check_and_notify(&quota).await {
                error!("Notification check failed: {}", e);
            }

            self.cache.set(quota.account_id.clone(), quota).await;
        }

        info!("Completed manual fetch cycle");
    }
}
