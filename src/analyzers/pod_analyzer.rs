//! Health analyzer for Pod resources

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct PodAnalyzer;

impl PodAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn calculate_health_score(&self, resource: &dyn ResourceV2) -> u8 {
        let mut score = 100u8;

        match resource.health_status() {
            HealthStatus::Healthy => {}
            HealthStatus::Warning => score = score.saturating_sub(20),
            HealthStatus::Error => score = score.saturating_sub(50),
            HealthStatus::Unknown => score = score.saturating_sub(10),
        }

        if resource
            .logs()
            .iter()
            .any(|(_, log, _)| contains_glibc_runtime_failure(log))
        {
            score = score.saturating_sub(25);
        }

        score
    }
}

fn contains_glibc_runtime_failure(log: &str) -> bool {
    log.contains("libc.so.6: version `GLIBC_")
        && log.contains("not found")
        && log.contains("required by")
}

fn first_matching_log_line<'a>(log: &'a str, pattern: &str) -> Option<&'a str> {
    log.lines()
        .find(|line| line.contains(pattern))
        .map(str::trim)
}

impl HealthAnalyzer for PodAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = if health_score >= 90 {
            "Pod is healthy and running normally".to_string()
        } else if health_score >= 70 {
            "Pod has minor issues".to_string()
        } else {
            "Pod has significant issues".to_string()
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Check for common pod issues
        for condition in resource.conditions() {
            if condition.type_ == "Ready" && condition.status != "True" {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "Pod Not Ready",
                    format!(
                        "Pod is not ready: {}",
                        condition.reason.as_deref().unwrap_or("Unknown")
                    ),
                ));

                analysis.add_recommendation(
                    Recommendation::new("Check Pod Status", "Investigate pod logs and events")
                        .with_action(format!(
                            "oc logs pod/{} -n {}",
                            resource.name(),
                            resource.namespace().unwrap_or("default")
                        )),
                );
            }
        }

        for (container_name, log, _) in resource.logs() {
            if contains_glibc_runtime_failure(&log) {
                let detail = first_matching_log_line(&log, "GLIBC_").unwrap_or(
                    "Container binary requires a newer glibc than the runtime image provides",
                );

                analysis.add_issue(
                    Issue::new(
                        IssueSeverity::Critical,
                        IssueCategory::Configuration,
                        "Runtime Library Mismatch",
                        format!(
                            "Container '{}' failed to start because the runtime image is missing required glibc symbols. {}",
                            container_name, detail
                        ),
                    )
                    .with_component(container_name.clone()),
                );

                analysis.add_recommendation(
                    Recommendation::new(
                        "Rebuild or Replace Container Image",
                        "The container binary was built against a newer glibc than the image/runtime provides. Align the binary, base image, and cluster-supported runtime.",
                    )
                    .with_action(format!(
                        "oc logs pod/{} -n {}",
                        resource.name(),
                        resource.namespace().unwrap_or("default")
                    )),
                );
            }
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec!["Pod"]
    }
}

impl Default for PodAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
