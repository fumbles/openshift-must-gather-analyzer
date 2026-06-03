//! Health analyzer for Node resources

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct NodeAnalyzer;

impl NodeAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_conditions(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        for condition in resource.conditions() {
            match condition.type_.as_str() {
                "Ready" => {
                    if condition.status != "True" {
                        analysis.add_issue(
                            Issue::new(
                                IssueSeverity::Critical,
                                IssueCategory::Availability,
                                "Node Not Ready",
                                format!(
                                    "Node is not in Ready state: {}",
                                    condition.reason.as_deref().unwrap_or("Unknown")
                                ),
                            )
                            .with_component("Ready condition"),
                        );

                        analysis.add_recommendation(
                            Recommendation::new(
                                "Investigate Node Status",
                                "Check node logs and kubelet status to determine why the node is not ready",
                            )
                            .with_action("oc describe node <node-name>")
                            .with_docs("https://docs.openshift.com/container-platform/latest/nodes/nodes/nodes-nodes-viewing.html")
                        );
                    }
                }
                "MemoryPressure" => {
                    if condition.status == "True" {
                        analysis.add_issue(
                            Issue::new(
                                IssueSeverity::Warning,
                                IssueCategory::Capacity,
                                "Memory Pressure Detected",
                                "Node is experiencing memory pressure, which may affect pod scheduling",
                            )
                            .with_component("MemoryPressure condition")
                        );

                        analysis.add_recommendation(
                            Recommendation::new(
                                "Free Up Memory",
                                "Consider evicting pods or adding more nodes to the cluster",
                            )
                            .with_action("oc adm top node <node-name>"),
                        );
                    }
                }
                "DiskPressure" => {
                    if condition.status == "True" {
                        analysis.add_issue(
                            Issue::new(
                                IssueSeverity::Warning,
                                IssueCategory::Storage,
                                "Disk Pressure Detected",
                                "Node is experiencing disk pressure, which may affect pod scheduling",
                            )
                            .with_component("DiskPressure condition")
                        );

                        analysis.add_recommendation(
                            Recommendation::new(
                                "Free Up Disk Space",
                                "Clean up unused images and volumes, or add more storage capacity",
                            )
                            .with_action("oc adm prune images"),
                        );
                    }
                }
                "PIDPressure" => {
                    if condition.status == "True" {
                        analysis.add_issue(
                            Issue::new(
                                IssueSeverity::Warning,
                                IssueCategory::Capacity,
                                "PID Pressure Detected",
                                "Node is running too many processes",
                            )
                            .with_component("PIDPressure condition"),
                        );
                    }
                }
                "NetworkUnavailable" => {
                    if condition.status == "True" {
                        analysis.add_issue(
                            Issue::new(
                                IssueSeverity::Error,
                                IssueCategory::Network,
                                "Network Unavailable",
                                "Node network is not properly configured",
                            )
                            .with_component("NetworkUnavailable condition"),
                        );

                        analysis.add_recommendation(
                            Recommendation::new(
                                "Check Network Configuration",
                                "Verify CNI plugin and network configuration on the node",
                            )
                            .with_docs("https://docs.openshift.com/container-platform/latest/networking/understanding-networking.html")
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn calculate_health_score(&self, resource: &dyn ResourceV2) -> u8 {
        let mut score = 100u8;

        // Check health status
        match resource.health_status() {
            HealthStatus::Healthy => {}
            HealthStatus::Warning => score = score.saturating_sub(20),
            HealthStatus::Error => score = score.saturating_sub(50),
            HealthStatus::Unknown => score = score.saturating_sub(10),
        }

        // Check conditions
        for condition in resource.conditions() {
            match condition.type_.as_str() {
                "Ready" if condition.status != "True" => score = score.saturating_sub(50),
                "MemoryPressure" | "DiskPressure" | "PIDPressure" if condition.status == "True" => {
                    score = score.saturating_sub(15)
                }
                "NetworkUnavailable" if condition.status == "True" => {
                    score = score.saturating_sub(30)
                }
                _ => {}
            }
        }

        score
    }
}

impl HealthAnalyzer for NodeAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = if health_score >= 90 {
            "Node is healthy and operating normally".to_string()
        } else if health_score >= 70 {
            "Node has minor issues that should be monitored".to_string()
        } else if health_score >= 50 {
            "Node has significant issues requiring attention".to_string()
        } else {
            "Node is in critical condition and needs immediate attention".to_string()
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Analyze conditions
        self.analyze_conditions(resource, &mut analysis);

        // Add general recommendations for unhealthy nodes
        if health_score < 90 {
            analysis.add_recommendation(
                Recommendation::new(
                    "Monitor Node Metrics",
                    "Keep an eye on CPU, memory, and disk usage",
                )
                .with_action("oc adm top node <node-name>"),
            );
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec!["Node"]
    }
}

impl Default for NodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
