//! Health analyzer for Machine resources

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct MachineAnalyzer;

impl MachineAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn calculate_health_score(&self, resource: &dyn ResourceV2) -> u8 {
        let mut score = 100u8;

        match resource.health_status() {
            HealthStatus::Healthy => {}
            HealthStatus::Warning => score = score.saturating_sub(25),
            HealthStatus::Error => score = score.saturating_sub(55),
            HealthStatus::Unknown => score = score.saturating_sub(10),
        }

        score
    }
}

impl HealthAnalyzer for MachineAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = if health_score >= 90 {
            "Machine is healthy and provisioned correctly".to_string()
        } else if health_score >= 70 {
            "Machine has minor issues".to_string()
        } else {
            "Machine has significant provisioning or health issues".to_string()
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Check machine phase and conditions
        for condition in resource.conditions() {
            if condition.type_ == "Ready" && condition.status != "True" {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "Machine Not Ready",
                    format!(
                        "Machine is not ready: {}",
                        condition.reason.as_deref().unwrap_or("Unknown")
                    ),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Machine Status",
                        "Investigate machine provisioning and node association",
                    )
                    .with_action("oc describe machine <machine-name> -n openshift-machine-api"),
                );
            }
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec!["Machine"]
    }
}

impl Default for MachineAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
