use std::sync::Arc;
use chrono::{Utc, Timelike};
use notify_rust::Notification;
use crate::db::{Repository, NotificationState};
use crate::providers::QuotaData;
use crate::error::Result;
use tracing::{info, warn};

pub struct Notifier {
    repo: Arc<Repository>,
}

impl Notifier {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub async fn check_and_notify(&self, quota: &QuotaData) -> Result<()> {
        // Check if notifications are enabled
        let enabled = self.repo.get_setting("notifications_enabled").await?
            .unwrap_or_else(|| "true".to_string());

        if enabled != "true" {
            return Ok(());
        }

        // Check if we're in quiet hours
        if self.is_quiet_hours().await? {
            return Ok(());
        }

        // Calculate usage percentage
        let percentage = self.calculate_usage_percentage(quota);
        if percentage.is_none() {
            return Ok(()); // Can't determine percentage
        }

        let percentage = percentage.unwrap();

        // Get or create notification state
        let mut state = self.repo.get_notification_state(&quota.account_id).await?
            .unwrap_or_else(|| NotificationState {
                account_id: quota.account_id.clone(),
                last_75_percent_notified: None,
                last_90_percent_notified: None,
                last_95_percent_notified: None,
            });

        let now = Utc::now().timestamp();
        let one_day_ago = now - 86400;

        // Check 95% threshold
        if percentage >= 95.0 && self.should_notify_threshold(&state.last_95_percent_notified, one_day_ago) {
            self.send_notification(
                "URGENT: Quota Critical",
                &format!("Your {} account is at {:.1}% - approaching limit!",
                    quota.account_id, percentage),
                notify_rust::Urgency::Critical,
            )?;
            state.last_95_percent_notified = Some(now);
            info!("Sent 95% notification for account {}", quota.account_id);
        }
        // Check 90% threshold
        else if percentage >= 90.0 && self.should_notify_threshold(&state.last_90_percent_notified, one_day_ago) {
            self.send_notification(
                "Quota Caution",
                &format!("Your {} account is at {:.1}% usage",
                    quota.account_id, percentage),
                notify_rust::Urgency::Normal,
            )?;
            state.last_90_percent_notified = Some(now);
            info!("Sent 90% notification for account {}", quota.account_id);
        }
        // Check 75% threshold
        else if percentage >= 75.0 && self.should_notify_threshold(&state.last_75_percent_notified, one_day_ago) {
            self.send_notification(
                "Quota Warning",
                &format!("Your {} account is at {:.1}% usage",
                    quota.account_id, percentage),
                notify_rust::Urgency::Low,
            )?;
            state.last_75_percent_notified = Some(now);
            info!("Sent 75% notification for account {}", quota.account_id);
        }

        // Update state
        self.repo.update_notification_state(&state).await?;

        Ok(())
    }

    fn should_notify_threshold(&self, last_notified: &Option<i64>, threshold: i64) -> bool {
        match last_notified {
            Some(time) => *time < threshold,
            None => true,
        }
    }

    fn calculate_usage_percentage(&self, quota: &QuotaData) -> Option<f64> {
        if let (Some(limit), Some(remaining)) = (quota.quota_limit, quota.quota_remaining) {
            if limit > 0 {
                let used = limit - remaining;
                return Some((used as f64 / limit as f64) * 100.0);
            }
        }

        // Fallback: if we have cost data but no explicit limits
        // This is a placeholder - in practice, users would set their own budget limits
        None
    }

    async fn is_quiet_hours(&self) -> Result<bool> {
        let start = self.repo.get_setting("quiet_hours_start").await?;
        let end = self.repo.get_setting("quiet_hours_end").await?;

        if let (Some(start), Some(end)) = (start, end) {
            if !start.is_empty() && !end.is_empty() {
                let now = chrono::Local::now();
                let current_hour = now.hour();

                // Parse hours (format: "HH:MM")
                if let (Some(start_hour), Some(end_hour)) = (
                    start.split(':').next().and_then(|h| h.parse::<u32>().ok()),
                    end.split(':').next().and_then(|h| h.parse::<u32>().ok()),
                ) {
                    if start_hour < end_hour {
                        return Ok(current_hour >= start_hour && current_hour < end_hour);
                    } else {
                        return Ok(current_hour >= start_hour || current_hour < end_hour);
                    }
                }
            }
        }

        Ok(false)
    }

    fn send_notification(&self, summary: &str, body: &str, urgency: notify_rust::Urgency) -> Result<()> {
        match Notification::new()
            .summary(summary)
            .body(body)
            .urgency(urgency)
            .timeout(notify_rust::Timeout::Milliseconds(6000))
            .show()
        {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to send notification: {}", e);
                Ok(()) // Don't fail the whole operation if notification fails
            }
        }
    }
}
