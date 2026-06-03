//! Health analyzer for ClusterOperator resources

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct OperatorAnalyzer;

impl OperatorAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn calculate_health_score(&self, resource: &dyn ResourceV2) -> u8 {
        let mut score = 100u8;

        match resource.health_status() {
            HealthStatus::Healthy => {}
            HealthStatus::Warning => score = score.saturating_sub(30),
            HealthStatus::Error => score = score.saturating_sub(60),
            HealthStatus::Unknown => score = score.saturating_sub(15),
        }

        score
    }
}

impl HealthAnalyzer for OperatorAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = if health_score >= 90 {
            "Operator is healthy and functioning normally".to_string()
        } else if health_score >= 70 {
            "Operator has warnings that should be investigated".to_string()
        } else {
            "Operator is degraded or unavailable".to_string()
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Check operator conditions
        for condition in resource.conditions() {
            match condition.type_.as_str() {
                "Available" if condition.status != "True" => {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Critical,
                        IssueCategory::Availability,
                        "Operator Not Available",
                        format!(
                            "Operator is not available: {}",
                            condition.reason.as_deref().unwrap_or("Unknown")
                        ),
                    ));

                    analysis.add_recommendation(
                        Recommendation::new(
                            "Check Operator Pods",
                            "Verify that operator pods are running and healthy",
                        )
                        .with_action("oc get pods -n openshift-<operator-namespace>"),
                    );
                }
                "Degraded" if condition.status == "True" => {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Error,
                        IssueCategory::Health,
                        "Operator Degraded",
                        format!(
                            "Operator is in degraded state: {}",
                            condition.reason.as_deref().unwrap_or("Unknown")
                        ),
                    ));

                    analysis.add_recommendation(
                        Recommendation::new(
                            "Review Operator Logs",
                            "Check operator logs for error messages",
                        )
                        .with_action("oc logs -n openshift-<operator-namespace> <pod-name>"),
                    );
                }
                "Progressing" if condition.status == "True" => {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Info,
                        IssueCategory::Health,
                        "Operator Progressing",
                        "Operator is currently progressing (updating or reconciling)",
                    ));
                }
                _ => {}
            }
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec!["ClusterOperator"]
    }
}

impl Default for OperatorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
