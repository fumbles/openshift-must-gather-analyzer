//! Health analyzer for Storage resources (PVC, PV, StorageClass, VolumeAttachment)

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct StorageAnalyzer;

impl StorageAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn calculate_health_score(&self, resource: &dyn ResourceV2) -> u8 {
        let mut score = 100u8;

        match resource.health_status() {
            HealthStatus::Healthy => {}
            HealthStatus::Warning => score = score.saturating_sub(30),
            HealthStatus::Error => score = score.saturating_sub(60),
            HealthStatus::Unknown => score = score.saturating_sub(10),
        }

        score
    }

    fn analyze_pvc(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let Some(phase) = key_fields.get("phase") {
            match phase.as_str() {
                "Pending" => {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Warning,
                        IssueCategory::Storage,
                        "PVC Pending",
                        format!(
                            "PersistentVolumeClaim {} is pending - waiting for volume provisioning",
                            resource.name()
                        ),
                    ));

                    analysis.add_recommendation(
                        Recommendation::new(
                            "Check Storage Provisioning",
                            "Verify that a StorageClass exists and can provision volumes",
                        )
                        .with_action(format!(
                            "oc get storageclass && oc describe pvc {} -n {}",
                            resource.name(),
                            resource.namespace().unwrap_or("default")
                        )),
                    );

                    if let Some(sc) = key_fields.get("storage_class") {
                        analysis.add_recommendation(
                            Recommendation::new(
                                "Verify StorageClass",
                                format!("Check if StorageClass '{}' is available and properly configured", sc),
                            )
                            .with_action(format!("oc get storageclass {}", sc))
                        );
                    }
                }
                "Lost" => {
                    analysis.add_issue(
                        Issue::new(
                            IssueSeverity::Critical,
                            IssueCategory::Storage,
                            "PVC Lost",
                            format!("PersistentVolumeClaim {} is in Lost state - bound volume no longer exists", resource.name()),
                        )
                    );

                    analysis.add_recommendation(
                        Recommendation::new(
                            "Recreate PVC",
                            "The bound PersistentVolume was deleted. You may need to recreate the PVC",
                        )
                        .with_action(format!("oc delete pvc {} -n {} && oc apply -f <pvc-definition>",
                            resource.name(),
                            resource.namespace().unwrap_or("default")))
                    );
                }
                _ => {}
            }
        }
    }

    fn analyze_pv(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let Some(phase) = key_fields.get("phase") {
            match phase.as_str() {
                "Available" => {
                    // Available PV with no claim is normal, but worth noting
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Info,
                        IssueCategory::Storage,
                        "PV Available",
                        format!(
                            "PersistentVolume {} is available and ready to be claimed",
                            resource.name()
                        ),
                    ));
                }
                "Released" => {
                    analysis.add_issue(
                        Issue::new(
                            IssueSeverity::Warning,
                            IssueCategory::Storage,
                            "PV Released",
                            format!("PersistentVolume {} is released - claim was deleted but volume not reclaimed", resource.name()),
                        )
                    );

                    analysis.add_recommendation(
                        Recommendation::new(
                            "Reclaim Volume",
                            "Manually reclaim the volume or delete it if no longer needed",
                        )
                        .with_action(format!("oc delete pv {}", resource.name())),
                    );
                }
                "Failed" => {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Critical,
                        IssueCategory::Storage,
                        "PV Failed",
                        format!("PersistentVolume {} is in Failed state", resource.name()),
                    ));

                    analysis.add_recommendation(
                        Recommendation::new(
                            "Investigate Failure",
                            "Check PV events and status for failure details",
                        )
                        .with_action(format!("oc describe pv {}", resource.name())),
                    );
                }
                _ => {}
            }
        }
    }

    fn analyze_volume_attachment(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let Some(attached) = key_fields
            .get("attached")
            .and_then(|s| s.parse::<bool>().ok())
        {
            if !attached {
                // Check if there are errors
                let has_errors = !resource.errors().is_empty();

                if has_errors {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Error,
                        IssueCategory::Storage,
                        "Volume Attachment Failed",
                        format!("VolumeAttachment {} failed to attach", resource.name()),
                    ));

                    // Add specific error messages
                    for error in resource.errors() {
                        analysis.add_issue(Issue::new(
                            IssueSeverity::Error,
                            IssueCategory::Storage,
                            "Attachment Error",
                            error,
                        ));
                    }
                } else {
                    analysis.add_issue(Issue::new(
                        IssueSeverity::Warning,
                        IssueCategory::Storage,
                        "Volume Not Attached",
                        format!("VolumeAttachment {} is not yet attached", resource.name()),
                    ));
                }

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Node and Volume",
                        "Verify that the node is ready and the volume is accessible",
                    )
                    .with_action(format!("oc describe volumeattachment {}", resource.name())),
                );

                if let Some(node) = key_fields.get("node_name") {
                    analysis.add_recommendation(
                        Recommendation::new(
                            "Check Node Status",
                            format!(
                                "Verify that node '{}' is ready and can attach volumes",
                                node
                            ),
                        )
                        .with_action(format!("oc get node {} -o wide", node)),
                    );
                }
            }
        }
    }
}

impl HealthAnalyzer for StorageAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = match resource.kind() {
            "PersistentVolumeClaim" => format!("PVC {} health analysis", resource.name()),
            "PersistentVolume" => format!("PV {} health analysis", resource.name()),
            "StorageClass" => format!("StorageClass {} health analysis", resource.name()),
            "VolumeAttachment" => format!("VolumeAttachment {} health analysis", resource.name()),
            _ => format!("Storage resource {} health analysis", resource.name()),
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Perform kind-specific analysis
        match resource.kind() {
            "PersistentVolumeClaim" => self.analyze_pvc(resource, &mut analysis),
            "PersistentVolume" => self.analyze_pv(resource, &mut analysis),
            "VolumeAttachment" => self.analyze_volume_attachment(resource, &mut analysis),
            "StorageClass" => {
                // StorageClass is a configuration object, always healthy
                analysis.add_issue(Issue::new(
                    IssueSeverity::Info,
                    IssueCategory::Storage,
                    "StorageClass Configuration",
                    format!("StorageClass {} is a configuration object", resource.name()),
                ));
            }
            _ => {}
        }

        // Add general recommendations if there are issues
        if !analysis.issues.is_empty()
            && analysis.issues.iter().any(|i| {
                matches!(
                    i.severity,
                    IssueSeverity::Warning | IssueSeverity::Error | IssueSeverity::Critical
                )
            })
        {
            analysis.add_recommendation(
                Recommendation::new(
                    "Review Storage Events",
                    "Check cluster events for storage-related issues",
                )
                .with_action(format!(
                    "oc get events --all-namespaces --field-selector involvedObject.name={}",
                    resource.name()
                )),
            );
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec![
            "PersistentVolumeClaim",
            "PersistentVolume",
            "StorageClass",
            "VolumeAttachment",
        ]
    }
}

impl Default for StorageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
