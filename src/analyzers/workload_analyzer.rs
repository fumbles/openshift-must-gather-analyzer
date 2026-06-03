//! Health analyzer for Workload resources (Deployments, StatefulSets, DaemonSets, Jobs, CronJobs, ReplicaSets)

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct WorkloadAnalyzer;

impl WorkloadAnalyzer {
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

        // Additional deductions based on key fields
        let key_fields = resource.key_fields();

        // Check replica counts for Deployments, StatefulSets, ReplicaSets
        if let (Some(replicas), Some(ready)) = (
            key_fields
                .get("replicas")
                .and_then(|s| s.parse::<i64>().ok()),
            key_fields
                .get("ready_replicas")
                .and_then(|s| s.parse::<i64>().ok()),
        ) {
            if replicas > 0 {
                let ready_percent = (ready * 100) / replicas;
                if ready_percent < 50 {
                    score = score.saturating_sub(30);
                } else if ready_percent < 100 {
                    score = score.saturating_sub(10);
                }
            }
        }

        // Check for failed jobs
        if let Some(failed) = key_fields.get("failed").and_then(|s| s.parse::<i64>().ok()) {
            if failed > 0 {
                score = score.saturating_sub(40);
            }
        }

        score
    }

    fn analyze_deployment(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let (Some(replicas), Some(available)) = (
            key_fields
                .get("replicas")
                .and_then(|s| s.parse::<i64>().ok()),
            key_fields
                .get("available_replicas")
                .and_then(|s| s.parse::<i64>().ok()),
        ) {
            if available < replicas {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "Deployment Not Fully Available",
                    format!("Only {}/{} replicas are available", available, replicas),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Pod Status",
                        "Investigate why pods are not becoming available",
                    )
                    .with_action(format!(
                        "oc get pods -l app={} -n {}",
                        resource.name(),
                        resource.namespace().unwrap_or("default")
                    )),
                );
            }
        }

        if let Some(unavailable) = key_fields
            .get("unavailable_replicas")
            .and_then(|s| s.parse::<i64>().ok())
        {
            if unavailable > 0 {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "Unavailable Replicas",
                    format!("{} replicas are unavailable", unavailable),
                ));
            }
        }
    }

    fn analyze_statefulset(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let (Some(replicas), Some(ready)) = (
            key_fields
                .get("replicas")
                .and_then(|s| s.parse::<i64>().ok()),
            key_fields
                .get("ready_replicas")
                .and_then(|s| s.parse::<i64>().ok()),
        ) {
            if ready < replicas {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "StatefulSet Not Fully Ready",
                    format!("Only {}/{} replicas are ready", ready, replicas),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check StatefulSet Pods",
                        "StatefulSets require sequential pod startup. Check if pods are stuck",
                    )
                    .with_action(format!(
                        "oc get pods -l app={} -n {} -o wide",
                        resource.name(),
                        resource.namespace().unwrap_or("default")
                    )),
                );
            }
        }

        if let (Some(replicas), Some(updated)) = (
            key_fields
                .get("replicas")
                .and_then(|s| s.parse::<i64>().ok()),
            key_fields
                .get("updated_replicas")
                .and_then(|s| s.parse::<i64>().ok()),
        ) {
            if updated < replicas {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Configuration,
                    "StatefulSet Update In Progress",
                    format!("Only {}/{} replicas are updated", updated, replicas),
                ));
            }
        }
    }

    fn analyze_daemonset(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let (Some(desired), Some(ready)) = (
            key_fields
                .get("desired_number_scheduled")
                .and_then(|s| s.parse::<i64>().ok()),
            key_fields
                .get("number_ready")
                .and_then(|s| s.parse::<i64>().ok()),
        ) {
            if ready < desired {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "DaemonSet Not Fully Ready",
                    format!("Only {}/{} pods are ready", ready, desired),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check DaemonSet Pods",
                        "DaemonSets should run on all matching nodes. Check node selectors and taints",
                    )
                    .with_action(format!("oc get pods -l app={} -n {} -o wide",
                        resource.name(),
                        resource.namespace().unwrap_or("default")))
                );
            }
        }

        if let Some(unavailable) = key_fields
            .get("number_unavailable")
            .and_then(|s| s.parse::<i64>().ok())
        {
            if unavailable > 0 {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "Unavailable DaemonSet Pods",
                    format!("{} pods are unavailable", unavailable),
                ));
            }
        }

        if let Some(misscheduled) = key_fields
            .get("number_misscheduled")
            .and_then(|s| s.parse::<i64>().ok())
        {
            if misscheduled > 0 {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Configuration,
                    "Misscheduled DaemonSet Pods",
                    format!(
                        "{} pods are running on nodes where they shouldn't",
                        misscheduled
                    ),
                ));
            }
        }
    }

    fn analyze_job(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let Some(failed) = key_fields.get("failed").and_then(|s| s.parse::<i64>().ok()) {
            if failed > 0 {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "Job Has Failed Pods",
                    format!("{} pods have failed", failed),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Job Logs",
                        "Review logs of failed pods to identify the issue",
                    )
                    .with_action(format!(
                        "oc logs job/{} -n {}",
                        resource.name(),
                        resource.namespace().unwrap_or("default")
                    )),
                );
            }
        }

        // Check if job is complete
        for condition in resource.conditions() {
            if condition.type_ == "Failed" && condition.status == "True" {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Critical,
                    IssueCategory::Availability,
                    "Job Failed",
                    condition
                        .message
                        .as_deref()
                        .unwrap_or("Job has failed")
                        .to_string(),
                ));
            }
        }
    }

    fn analyze_cronjob(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let Some(suspend) = key_fields
            .get("suspend")
            .and_then(|s| s.parse::<bool>().ok())
        {
            if suspend {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Configuration,
                    "CronJob Suspended",
                    "CronJob is currently suspended and will not create new jobs",
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Resume CronJob",
                        "If the suspension was unintentional, resume the CronJob",
                    )
                    .with_action(format!(
                        "oc patch cronjob {} -n {} -p '{{\"spec\":{{\"suspend\":false}}}}'",
                        resource.name(),
                        resource.namespace().unwrap_or("default")
                    )),
                );
            }
        }
    }

    fn analyze_replicaset(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let (Some(replicas), Some(ready)) = (
            key_fields
                .get("replicas")
                .and_then(|s| s.parse::<i64>().ok()),
            key_fields
                .get("ready_replicas")
                .and_then(|s| s.parse::<i64>().ok()),
        ) {
            if replicas > 0 && ready < replicas {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "ReplicaSet Not Fully Ready",
                    format!("Only {}/{} replicas are ready", ready, replicas),
                ));
            }
        }
    }
}

impl HealthAnalyzer for WorkloadAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = match resource.kind() {
            "Deployment" => format!("Deployment {} health analysis", resource.name()),
            "StatefulSet" => format!("StatefulSet {} health analysis", resource.name()),
            "DaemonSet" => format!("DaemonSet {} health analysis", resource.name()),
            "Job" => format!("Job {} health analysis", resource.name()),
            "CronJob" => format!("CronJob {} health analysis", resource.name()),
            "ReplicaSet" => format!("ReplicaSet {} health analysis", resource.name()),
            _ => format!("Workload {} health analysis", resource.name()),
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Perform kind-specific analysis
        match resource.kind() {
            "Deployment" => self.analyze_deployment(resource, &mut analysis),
            "StatefulSet" => self.analyze_statefulset(resource, &mut analysis),
            "DaemonSet" => self.analyze_daemonset(resource, &mut analysis),
            "Job" => self.analyze_job(resource, &mut analysis),
            "CronJob" => self.analyze_cronjob(resource, &mut analysis),
            "ReplicaSet" => self.analyze_replicaset(resource, &mut analysis),
            _ => {}
        }

        // Add general recommendations if there are issues
        if !analysis.issues.is_empty() {
            analysis.add_recommendation(
                Recommendation::new(
                    "Review Events",
                    "Check cluster events for additional context",
                )
                .with_action(format!(
                    "oc get events -n {} --field-selector involvedObject.name={}",
                    resource.namespace().unwrap_or("default"),
                    resource.name()
                )),
            );
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec![
            "Deployment",
            "StatefulSet",
            "DaemonSet",
            "Job",
            "CronJob",
            "ReplicaSet",
        ]
    }
}

impl Default for WorkloadAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
