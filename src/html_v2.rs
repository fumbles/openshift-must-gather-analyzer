// Copyright 2024 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::analyzers::{AnalyzerRegistry, HealthAnalysis};
use crate::mustgather::MustGather;
use crate::resources::{HealthStatus, RelationshipType, Resource, ResourceV2};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Embed the frontend bundles at compile time
const REACT_JS: &str = include_str!("../frontend/dist/assets/index.js");
const REACT_CSS: &str = include_str!("../frontend/dist/assets/index.css");

/// Serializable representation of a resource for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLogData {
    pub container: String,
    pub content: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOwnerReferenceData {
    pub kind: String,
    pub name: String,
    pub uid: Option<String>,
    pub controller: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRelationshipData {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub relationship: String,
}

/// Serializable representation of a resource for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub uid: String,
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_timestamp: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_path: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub logs: Vec<ResourceLogData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_path: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub labels: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub annotations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub owner_references: Vec<ResourceOwnerReferenceData>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub relationships: Vec<ResourceRelationshipData>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub key_fields: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_analysis: Option<HealthAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_path: Option<String>,
}

/// Dashboard data for overview section
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub total_pods: usize,
    pub healthy_pods: usize,
    pub total_operators: usize,
    pub degraded_operators: usize,
    pub cluster_version: Option<String>,
    pub platform_type: Option<String>,
    pub collection_timestamp: Option<String>,
}

/// Cluster health data
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterHealthData {
    pub operators: Vec<ResourceData>,
    pub installed_operators: ResourceCollection,
    pub degraded_count: usize,
    pub unavailable_count: usize,
}

/// Resource collection with issue counts
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceCollection {
    pub items: Vec<ResourceData>,
    #[serde(flatten)]
    pub counts: HashMap<String, usize>,
}

/// Workloads data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkloadsData {
    pub pods: ResourceCollection,
    pub deployments: ResourceCollection,
    pub statefulsets: ResourceCollection,
    pub configmaps: ResourceCollection,
    pub secrets: ResourceCollection,
    pub daemonsets: ResourceCollection,
    pub jobs: ResourceCollection,
    pub cronjobs: ResourceCollection,
    pub replicasets: ResourceCollection,
}

/// Networking data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkingData {
    pub routes: ResourceCollection,
    pub services: ResourceCollection,
    pub endpoints: ResourceCollection,
    pub network_policies: ResourceCollection,
    pub ingress_controllers: ResourceCollection,
}

/// Storage data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageData {
    pub pvcs: ResourceCollection,
    pub pvs: ResourceCollection,
    pub storage_classes: ResourceCollection,
    pub volume_attachments: ResourceCollection,
}

/// Compute data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeData {
    pub machines: ResourceCollection,
    pub machine_health_checks: ResourceCollection,
    pub machine_autoscalers: ResourceCollection,
    pub control_plane_machine_sets: ResourceCollection,
    pub machine_sets: ResourceCollection,
    pub machine_configs: ResourceCollection,
    pub machine_config_pools: ResourceCollection,
    pub machine_configurations: ResourceCollection,
}

/// Security data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityData {
    pub cluster_roles: ResourceCollection,
    pub cluster_role_bindings: ResourceCollection,
    pub security_context_constraints: ResourceCollection,
}

/// Administration data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct AdministrationData {
    pub cluster_settings: ResourceCollection,
    pub namespaces: ResourceCollection,
    pub resource_quotas: ResourceCollection,
    pub limit_ranges: ResourceCollection,
    pub custom_resource_definitions: ResourceCollection,
    pub dynamic_plugins: ResourceCollection,
}

/// OpenShift Virtualization resources
#[derive(Debug, Serialize, Deserialize)]
pub struct VirtualizationData {
    pub hyperconvergeds: ResourceCollection,
    pub kubevirts: ResourceCollection,
    pub virtual_machines: ResourceCollection,
    pub virtual_machine_instances: ResourceCollection,
    pub virtual_machine_pools: ResourceCollection,
    pub virtual_machine_exports: ResourceCollection,
    pub virtual_machine_clones: ResourceCollection,
    pub virtual_machine_snapshots: ResourceCollection,
    pub virtual_machine_snapshot_contents: ResourceCollection,
    pub virtual_machine_restores: ResourceCollection,
    pub data_volumes: ResourceCollection,
    pub data_sources: ResourceCollection,
    pub data_import_crons: ResourceCollection,
    pub instance_types: ResourceCollection,
    pub preferences: ResourceCollection,
}

/// Platform detection and platform-specific resource data
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformData {
    pub fusion_detected: bool,
    pub odf_detected: bool,
    pub service_mesh_detected: bool,
    pub acm_detected: bool,
    pub virtualization_detected: bool,
    pub cpd_detected: bool,
    pub virtualization: VirtualizationData,
}

/// Overview section
#[derive(Debug, Serialize, Deserialize)]
pub struct OverviewSection {
    pub dashboard: DashboardData,
    pub cluster_health: ClusterHealthData,
}

/// Core section
#[derive(Debug, Serialize, Deserialize)]
pub struct CoreSection {
    pub nodes: ResourceCollection,
    pub workloads: WorkloadsData,
    pub namespaces: ResourceCollection,
    pub events: ResourceCollection,
}

/// Infrastructure section
#[derive(Debug, Serialize, Deserialize)]
pub struct InfrastructureSection {
    pub networking: NetworkingData,
    pub storage: StorageData,
}

/// Complete triage-based must-gather data
#[derive(Debug, Serialize, Deserialize)]
pub struct TriageMustGatherData {
    pub overview: OverviewSection,
    pub core: CoreSection,
    pub compute: ComputeData,
    pub security: SecurityData,
    pub administration: AdministrationData,
    pub infrastructure: InfrastructureSection,
    pub platform: PlatformData,
}

// Helper functions for counting issues

fn count_unhealthy_resources<T: ResourceV2>(resources: &[T]) -> usize {
    resources
        .iter()
        .filter(|r| {
            matches!(
                r.health_status(),
                HealthStatus::Error | HealthStatus::Warning
            )
        })
        .count()
}

fn count_error_resources<T: ResourceV2>(resources: &[T]) -> usize {
    resources
        .iter()
        .filter(|r| r.health_status() == HealthStatus::Error)
        .count()
}

fn count_degraded_operators(operators: &[crate::resources::ClusterOperator]) -> usize {
    operators
        .iter()
        .filter(|op| {
            ResourceV2::conditions(*op)
                .iter()
                .any(|c| c.type_ == "Degraded" && c.status == "True")
        })
        .count()
}

fn count_unavailable_operators(operators: &[crate::resources::ClusterOperator]) -> usize {
    operators
        .iter()
        .filter(|op| {
            ResourceV2::conditions(*op)
                .iter()
                .any(|c| c.type_ == "Available" && c.status == "False")
        })
        .count()
}

fn count_not_ready_nodes(nodes: &[crate::resources::Node]) -> usize {
    nodes
        .iter()
        .filter(|n| {
            !ResourceV2::conditions(*n)
                .iter()
                .any(|c| c.type_ == "Ready" && c.status == "True")
        })
        .count()
}

fn count_pressure_nodes(nodes: &[crate::resources::Node]) -> usize {
    nodes
        .iter()
        .filter(|n| {
            ResourceV2::conditions(*n).iter().any(|c| {
                (c.type_.contains("Pressure")
                    || c.type_ == "DiskPressure"
                    || c.type_ == "MemoryPressure"
                    || c.type_ == "PIDPressure")
                    && c.status == "True"
            })
        })
        .count()
}

fn count_unavailable_deployments(deployments: &[crate::resources::Deployment]) -> usize {
    deployments
        .iter()
        .filter(|d| {
            let key_fields = d.key_fields();
            if let (Some(replicas), Some(available)) = (
                key_fields
                    .get("replicas")
                    .and_then(|s| s.parse::<i64>().ok()),
                key_fields
                    .get("available_replicas")
                    .and_then(|s| s.parse::<i64>().ok()),
            ) {
                available < replicas
            } else {
                false
            }
        })
        .count()
}

fn count_unavailable_statefulsets(statefulsets: &[crate::resources::StatefulSet]) -> usize {
    statefulsets
        .iter()
        .filter(|s| {
            let key_fields = s.key_fields();
            if let (Some(replicas), Some(ready)) = (
                key_fields
                    .get("replicas")
                    .and_then(|s| s.parse::<i64>().ok()),
                key_fields
                    .get("ready_replicas")
                    .and_then(|s| s.parse::<i64>().ok()),
            ) {
                ready < replicas
            } else {
                false
            }
        })
        .count()
}

fn count_misscheduled_daemonsets(daemonsets: &[crate::resources::DaemonSet]) -> usize {
    daemonsets
        .iter()
        .filter(|d| {
            let key_fields = d.key_fields();
            key_fields
                .get("number_misscheduled")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|n| n > 0)
                .unwrap_or(false)
        })
        .count()
}

fn count_failed_jobs(jobs: &[crate::resources::Job]) -> usize {
    jobs.iter()
        .filter(|j| {
            let key_fields = j.key_fields();
            key_fields
                .get("failed")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|n| n > 0)
                .unwrap_or(false)
        })
        .count()
}

fn count_suspended_cronjobs(cronjobs: &[crate::resources::CronJob]) -> usize {
    cronjobs
        .iter()
        .filter(|c| {
            let key_fields = c.key_fields();
            key_fields
                .get("suspend")
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(false)
        })
        .count()
}

fn count_unavailable_replicasets(replicasets: &[crate::resources::ReplicaSet]) -> usize {
    replicasets
        .iter()
        .filter(|r| {
            let key_fields = r.key_fields();
            if let (Some(replicas), Some(ready)) = (
                key_fields
                    .get("replicas")
                    .and_then(|s| s.parse::<i64>().ok()),
                key_fields
                    .get("ready_replicas")
                    .and_then(|s| s.parse::<i64>().ok()),
            ) {
                replicas > 0 && ready < replicas
            } else {
                false
            }
        })
        .count()
}

fn count_crashloop_pods(pods: &[crate::resources::Pod]) -> usize {
    pods.iter()
        .filter(|p| {
            // Check container statuses in the manifest for CrashLoopBackOff
            let raw = ResourceV2::raw(*p);
            raw.contains("CrashLoopBackOff")
        })
        .count()
}

fn count_pending_pods(pods: &[crate::resources::Pod]) -> usize {
    pods.iter()
        .filter(|p| {
            // Check pod phase in the manifest
            let raw = ResourceV2::raw(*p);
            raw.contains("phase: Pending")
        })
        .count()
}

fn count_oomkilled_pods(pods: &[crate::resources::Pod]) -> usize {
    pods.iter()
        .filter(|p| {
            // Check container statuses in the manifest for OOMKilled
            let raw = ResourceV2::raw(*p);
            raw.contains("OOMKilled")
        })
        .count()
}

fn count_unadmitted_routes(routes: &[crate::resources::Route]) -> usize {
    routes
        .iter()
        .filter(|r| {
            let key_fields = r.key_fields();
            key_fields
                .get("admitted")
                .and_then(|s| s.parse::<bool>().ok())
                .map(|admitted| !admitted)
                .unwrap_or(false)
        })
        .count()
}

fn count_no_endpoints_services(_services: &[crate::resources::Service]) -> usize {
    // Note: We can't directly check if a service has endpoints without cross-referencing
    // This is a placeholder that returns 0 for now
    // In a full implementation, we'd need to correlate with Endpoints resources
    0
}

fn count_not_ready_endpoints(endpoints: &[crate::resources::Endpoints]) -> usize {
    endpoints
        .iter()
        .filter(|e| {
            let key_fields = e.key_fields();
            let ready_count = key_fields
                .get("ready_addresses")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            ready_count == 0
        })
        .count()
}

fn count_degraded_ingress_controllers(
    controllers: &[crate::resources::IngressController],
) -> usize {
    controllers
        .iter()
        .filter(|c| {
            ResourceV2::conditions(*c)
                .iter()
                .any(|cond| cond.type_ == "Degraded" && cond.status == "True")
        })
        .count()
}

fn count_pending_pvcs(pvcs: &[crate::resources::PersistentVolumeClaim]) -> usize {
    pvcs.iter()
        .filter(|p| {
            let key_fields = p.key_fields();
            key_fields
                .get("phase")
                .map(|phase| phase == "Pending")
                .unwrap_or(false)
        })
        .count()
}

fn count_unbound_pvs(pvs: &[crate::resources::PersistentVolume]) -> usize {
    pvs.iter()
        .filter(|p| {
            let key_fields = p.key_fields();
            key_fields
                .get("phase")
                .map(|phase| phase == "Available" || phase == "Released")
                .unwrap_or(false)
        })
        .count()
}

fn count_failed_volume_attachments(attachments: &[crate::resources::VolumeAttachment]) -> usize {
    attachments
        .iter()
        .filter(|a| {
            let key_fields = a.key_fields();
            key_fields
                .get("attached")
                .and_then(|s| s.parse::<bool>().ok())
                .map(|attached| !attached && !a.errors().is_empty())
                .unwrap_or(false)
        })
        .count()
}

// Convert resources to ResourceData
fn serialize_resources<T: Resource + ResourceV2>(
    resources: &[T],
    analyzer_registry: &AnalyzerRegistry,
    summary_only: bool,
) -> Vec<ResourceData> {
    resources
        .iter()
        .map(|r| {
            let metadata = r.metadata();
            let resource_uid = if metadata.uid.is_empty() {
                match &metadata.namespace {
                    Some(namespace) => {
                        format!(
                            "{}__{}__{}",
                            ResourceV2::kind(r).to_lowercase(),
                            namespace,
                            ResourceV2::name(r)
                        )
                    }
                    None => format!(
                        "{}__{}",
                        ResourceV2::kind(r).to_lowercase(),
                        ResourceV2::name(r)
                    ),
                }
            } else {
                metadata.uid.clone()
            };
            let mut meta_map = HashMap::new();
            if !summary_only {
                meta_map.insert("uid".to_string(), metadata.uid.clone());
                if let Some(ns) = &metadata.namespace {
                    meta_map.insert("namespace".to_string(), ns.clone());
                }
                for (key, value) in &metadata.labels {
                    meta_map.insert(format!("label:{}", key), value.clone());
                }
                for (key, value) in &metadata.annotations {
                    meta_map.insert(format!("annotation:{}", key), value.clone());
                }
                if let Some(timestamp) = &metadata.creation_timestamp {
                    meta_map.insert("creation_timestamp".to_string(), timestamp.clone());
                }
            }

            // Perform health analysis
            let health_analysis = analyzer_registry.analyze(r as &dyn ResourceV2).ok();
            let labels = if summary_only {
                match ResourceV2::kind(r) {
                    "Pod" | "Job" => metadata.labels.clone(),
                    _ => HashMap::new(),
                }
            } else {
                metadata.labels.clone()
            };
            let annotations = if summary_only {
                metadata
                    .annotations
                    .iter()
                    .filter(|(key, _)| key.as_str() == "cronjob.kubernetes.io/instantiate")
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            } else {
                metadata.annotations.clone()
            };
            let key_fields = if summary_only {
                let allowed = match ResourceV2::kind(r) {
                    "Pod" => &["phase", "restart_count"][..],
                    "Deployment" | "ReplicaSet" | "StatefulSet" => {
                        &["ready_replicas", "replicas", "available_replicas"][..]
                    }
                    "DaemonSet" => &[
                        "desired_number_scheduled",
                        "current_number_scheduled",
                        "number_ready",
                    ][..],
                    "Job" => &["completions", "succeeded", "failed"][..],
                    "CronJob" => &["schedule", "suspend"][..],
                    "ConfigMap" => &["data_count", "binary_data_count", "immutable"][..],
                    "Secret" => &["type", "data_count", "immutable"][..],
                    "Service" => &["selector", "type", "cluster_ip", "ports"][..],
                    "Route" => &["host", "service_name", "tls_termination"][..],
                    "IngressController" => &["domain", "available"][..],
                    "Machine" => &["phase", "provider_id"][..],
                    "MachineAutoscaler" => &["min_replicas", "max_replicas"][..],
                    "MachineHealthCheck" => &[
                        "current_healthy",
                        "expected_machines",
                        "remediations_allowed",
                        "max_unhealthy",
                        "remediation_allowed",
                    ][..],
                    "ControlPlaneMachineSet" => &["ready"][..],
                    "MachineSet" => &["replicas", "autoscaling"][..],
                    "MachineConfig" => &["os_image_url"][..],
                    "MachineConfigPool" => &[
                        "degraded",
                        "updating",
                        "machine_count",
                        "ready_machine_count",
                    ][..],
                    "MachineConfiguration" => &[
                        "boot_image_update_degraded",
                        "boot_image_update_progressing",
                    ][..],
                    "Event" => &[
                        "type",
                        "reason",
                        "count",
                        "message",
                        "involved_kind",
                        "involved_name",
                        "timestamp",
                        "reporting_component",
                    ][..],
                    "CustomResourceDefinition" => {
                        &["crd_group", "crd_plural", "crd_kind", "crd_scope"][..]
                    }
                    _ => &[][..],
                };

                r.key_fields()
                    .into_iter()
                    .filter(|(key, _)| allowed.contains(&key.as_str()))
                    .collect()
            } else {
                r.key_fields()
            };
            let owner_references = if summary_only {
                match ResourceV2::kind(r) {
                    "Pod" | "Job" | "ReplicaSet" | "Deployment" | "StatefulSet" | "DaemonSet"
                    | "Machine" | "MachineSet" | "MachineAutoscaler" => r
                        .owner_references()
                        .into_iter()
                        .map(|(kind, name, uid, controller)| ResourceOwnerReferenceData {
                            kind,
                            name,
                            uid,
                            controller,
                        })
                        .collect(),
                    _ => Vec::new(),
                }
            } else {
                r.owner_references()
                    .into_iter()
                    .map(|(kind, name, uid, controller)| ResourceOwnerReferenceData {
                        kind,
                        name,
                        uid,
                        controller,
                    })
                    .collect()
            };
            let relationships = r
                .relationships()
                .into_iter()
                .map(|link| ResourceRelationshipData {
                    kind: link.kind,
                    name: link.name,
                    namespace: link.namespace,
                    relationship: match link.relationship {
                        RelationshipType::Owns => "Owns",
                        RelationshipType::OwnedBy => "OwnedBy",
                        RelationshipType::References => "References",
                        RelationshipType::ReferencedBy => "ReferencedBy",
                        RelationshipType::Controls => "Controls",
                        RelationshipType::ControlledBy => "ControlledBy",
                    }
                    .to_string(),
                })
                .collect();
            ResourceData {
                uid: resource_uid.clone(),
                name: ResourceV2::name(r).to_string(),
                kind: ResourceV2::kind(r).to_string(),
                namespace: ResourceV2::namespace(r).map(|s| s.to_string()),
                creation_timestamp: metadata.creation_timestamp.clone(),
                status: format!("{:?}", ResourceV2::health_status(r)),
                errors: r.errors(),
                warnings: r.warnings(),
                raw: if summary_only {
                    String::new()
                } else {
                    ResourceV2::raw(r).to_string()
                },
                raw_path: if summary_only {
                    None
                } else {
                    Some(format!("data/raw/{}.js", resource_uid))
                },
                logs: if summary_only {
                    Vec::new()
                } else {
                    r.logs()
                        .into_iter()
                        .map(|(container, content, path)| ResourceLogData {
                            container,
                            content,
                            path,
                        })
                        .collect()
                },
                logs_path: if summary_only || ResourceV2::kind(r) != "Pod" {
                    None
                } else {
                    Some(format!("data/logs/{}.js", resource_uid))
                },
                labels,
                annotations,
                owner_references,
                relationships,
                key_fields,
                metadata: meta_map,
                health_analysis: if summary_only { None } else { health_analysis },
                detail_path: None,
            }
        })
        .collect()
}

fn strip_logs(resource: &ResourceData) -> ResourceData {
    let mut resource = resource.clone();
    resource.logs.clear();
    resource
}

fn strip_raw_and_logs(resource: &ResourceData) -> ResourceData {
    let mut resource = strip_logs(resource);
    resource.raw.clear();
    resource
}

impl TriageMustGatherData {
    /// Convert a MustGather into triage-based serializable data
    pub fn from_must_gather(mg: &MustGather) -> Self {
        Self::from_must_gather_with_mode(mg, false)
    }

    pub fn from_must_gather_summary(mg: &MustGather) -> Self {
        Self::from_must_gather_with_mode(mg, true)
    }

    fn from_must_gather_with_mode(mg: &MustGather, summary_only: bool) -> Self {
        let analyzer_registry = AnalyzerRegistry::new();

        // Use all pods from all namespaces
        let all_pods = &mg.pods;

        // Calculate dashboard metrics
        let total_nodes = mg.nodes.len();
        let healthy_nodes = total_nodes - count_unhealthy_resources(&mg.nodes);
        let total_pods = all_pods.len();
        let healthy_pods = total_pods - count_unhealthy_resources(all_pods);
        let total_operators = mg.clusteroperators.len();
        let degraded_operators = count_degraded_operators(&mg.clusteroperators);

        let dashboard = DashboardData {
            total_nodes,
            healthy_nodes,
            total_pods,
            healthy_pods,
            total_operators,
            degraded_operators,
            cluster_version: if mg.version != "Unknown" {
                Some(mg.version.clone())
            } else {
                None
            },
            platform_type: if mg.platformtype != "Unknown" {
                Some(mg.platformtype.clone())
            } else {
                None
            },
            collection_timestamp: mg.collection_timestamp.clone(),
        };

        // Cluster health
        let mut installed_operators_counts = HashMap::new();
        let installed_operators_errors = count_error_resources(&mg.operators);
        installed_operators_counts.insert("error_count".to_string(), installed_operators_errors);
        installed_operators_counts.insert(
            "warning_count".to_string(),
            count_unhealthy_resources(&mg.operators).saturating_sub(installed_operators_errors),
        );

        let cluster_health = ClusterHealthData {
            operators: serialize_resources(&mg.clusteroperators, &analyzer_registry, summary_only),
            installed_operators: ResourceCollection {
                items: serialize_resources(&mg.operators, &analyzer_registry, summary_only),
                counts: installed_operators_counts,
            },
            degraded_count: degraded_operators,
            unavailable_count: count_unavailable_operators(&mg.clusteroperators),
        };

        // Nodes
        let mut nodes_counts = HashMap::new();
        nodes_counts.insert(
            "not_ready_count".to_string(),
            count_not_ready_nodes(&mg.nodes),
        );
        nodes_counts.insert(
            "pressure_count".to_string(),
            count_pressure_nodes(&mg.nodes),
        );

        let nodes = ResourceCollection {
            items: serialize_resources(&mg.nodes, &analyzer_registry, summary_only),
            counts: nodes_counts,
        };

        // Workloads
        let mut deployments_counts = HashMap::new();
        deployments_counts.insert(
            "unavailable_count".to_string(),
            count_unavailable_deployments(&mg.deployments),
        );

        let mut statefulsets_counts = HashMap::new();
        statefulsets_counts.insert(
            "unavailable_count".to_string(),
            count_unavailable_statefulsets(&mg.statefulsets),
        );

        let mut daemonsets_counts = HashMap::new();
        daemonsets_counts.insert(
            "misscheduled_count".to_string(),
            count_misscheduled_daemonsets(&mg.daemonsets),
        );

        let mut jobs_counts = HashMap::new();
        jobs_counts.insert("failed_count".to_string(), count_failed_jobs(&mg.jobs));

        let mut cronjobs_counts = HashMap::new();
        cronjobs_counts.insert(
            "suspended_count".to_string(),
            count_suspended_cronjobs(&mg.cronjobs),
        );

        let mut replicasets_counts = HashMap::new();
        replicasets_counts.insert(
            "unavailable_count".to_string(),
            count_unavailable_replicasets(&mg.replicasets),
        );

        let mut pods_counts = HashMap::new();
        pods_counts.insert(
            "crashloop_count".to_string(),
            count_crashloop_pods(&all_pods),
        );
        pods_counts.insert("pending_count".to_string(), count_pending_pods(&all_pods));
        pods_counts.insert(
            "oomkilled_count".to_string(),
            count_oomkilled_pods(&all_pods),
        );

        let workloads = WorkloadsData {
            pods: ResourceCollection {
                items: serialize_resources(all_pods, &analyzer_registry, summary_only),
                counts: pods_counts,
            },
            deployments: ResourceCollection {
                items: serialize_resources(&mg.deployments, &analyzer_registry, summary_only),
                counts: deployments_counts,
            },
            statefulsets: ResourceCollection {
                items: serialize_resources(&mg.statefulsets, &analyzer_registry, summary_only),
                counts: statefulsets_counts,
            },
            configmaps: ResourceCollection {
                items: serialize_resources(&mg.configmaps, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            secrets: ResourceCollection {
                items: serialize_resources(&mg.secrets, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            daemonsets: ResourceCollection {
                items: serialize_resources(&mg.daemonsets, &analyzer_registry, summary_only),
                counts: daemonsets_counts,
            },
            jobs: ResourceCollection {
                items: serialize_resources(&mg.jobs, &analyzer_registry, summary_only),
                counts: jobs_counts,
            },
            cronjobs: ResourceCollection {
                items: serialize_resources(&mg.cronjobs, &analyzer_registry, summary_only),
                counts: cronjobs_counts,
            },
            replicasets: ResourceCollection {
                items: serialize_resources(&mg.replicasets, &analyzer_registry, summary_only),
                counts: replicasets_counts,
            },
        };

        // Namespaces
        let namespaces = ResourceCollection {
            items: serialize_resources(&mg.namespaces, &analyzer_registry, summary_only),
            counts: HashMap::new(),
        };

        // Events
        let mut events_counts = HashMap::new();
        events_counts.insert(
            "warning_count".to_string(),
            mg.events
                .iter()
                .filter(|event| ResourceV2::health_status(*event) == HealthStatus::Warning)
                .count(),
        );

        let events = ResourceCollection {
            items: serialize_resources(&mg.events, &analyzer_registry, summary_only),
            counts: events_counts,
        };

        // Compute
        let mut machines_counts = HashMap::new();
        machines_counts.insert(
            "not_running_count".to_string(),
            count_error_resources(&mg.machines),
        );

        let mut machine_health_checks_counts = HashMap::new();
        machine_health_checks_counts.insert(
            "unhealthy_count".to_string(),
            count_error_resources(&mg.machinehealthchecks),
        );

        let mut machine_config_pools_counts = HashMap::new();
        let machine_config_pool_errors = count_error_resources(&mg.machineconfigpools);
        machine_config_pools_counts
            .insert("degraded_count".to_string(), machine_config_pool_errors);
        machine_config_pools_counts.insert(
            "updating_count".to_string(),
            count_unhealthy_resources(&mg.machineconfigpools)
                .saturating_sub(machine_config_pool_errors),
        );

        let mut machine_configurations_counts = HashMap::new();
        let machine_configuration_errors = count_error_resources(&mg.machineconfigurations);
        machine_configurations_counts
            .insert("degraded_count".to_string(), machine_configuration_errors);
        machine_configurations_counts.insert(
            "progressing_count".to_string(),
            count_unhealthy_resources(&mg.machineconfigurations)
                .saturating_sub(machine_configuration_errors),
        );

        let compute = ComputeData {
            machines: ResourceCollection {
                items: serialize_resources(&mg.machines, &analyzer_registry, summary_only),
                counts: machines_counts,
            },
            machine_health_checks: ResourceCollection {
                items: serialize_resources(
                    &mg.machinehealthchecks,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: machine_health_checks_counts,
            },
            machine_autoscalers: ResourceCollection {
                items: serialize_resources(
                    &mg.machineautoscalers,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            control_plane_machine_sets: ResourceCollection {
                items: serialize_resources(
                    &mg.controlplanemachinesets,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            machine_sets: ResourceCollection {
                items: serialize_resources(&mg.machinesets, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            machine_configs: ResourceCollection {
                items: serialize_resources(&mg.machineconfigs, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            machine_config_pools: ResourceCollection {
                items: serialize_resources(
                    &mg.machineconfigpools,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: machine_config_pools_counts,
            },
            machine_configurations: ResourceCollection {
                items: serialize_resources(
                    &mg.machineconfigurations,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: machine_configurations_counts,
            },
        };

        // Networking
        let mut routes_counts = HashMap::new();
        routes_counts.insert(
            "unadmitted_count".to_string(),
            count_unadmitted_routes(&mg.routes),
        );

        let mut services_counts = HashMap::new();
        services_counts.insert(
            "no_endpoints_count".to_string(),
            count_no_endpoints_services(&mg.services),
        );

        let mut endpoints_counts = HashMap::new();
        endpoints_counts.insert(
            "not_ready_count".to_string(),
            count_not_ready_endpoints(&mg.endpoints),
        );

        let mut ingress_controllers_counts = HashMap::new();
        ingress_controllers_counts.insert(
            "degraded_count".to_string(),
            count_degraded_ingress_controllers(&mg.ingress_controllers),
        );

        let mut network_policies_counts = HashMap::new();
        network_policies_counts.insert(
            "warning_count".to_string(),
            count_unhealthy_resources(&mg.networkpolicies),
        );

        let networking = NetworkingData {
            routes: ResourceCollection {
                items: serialize_resources(&mg.routes, &analyzer_registry, summary_only),
                counts: routes_counts,
            },
            services: ResourceCollection {
                items: serialize_resources(&mg.services, &analyzer_registry, summary_only),
                counts: services_counts,
            },
            endpoints: ResourceCollection {
                items: serialize_resources(&mg.endpoints, &analyzer_registry, summary_only),
                counts: endpoints_counts,
            },
            network_policies: ResourceCollection {
                items: serialize_resources(&mg.networkpolicies, &analyzer_registry, summary_only),
                counts: network_policies_counts,
            },
            ingress_controllers: ResourceCollection {
                items: serialize_resources(
                    &mg.ingress_controllers,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: ingress_controllers_counts,
            },
        };

        let mut security_context_constraints_counts = HashMap::new();
        security_context_constraints_counts.insert(
            "warning_count".to_string(),
            count_unhealthy_resources(&mg.securitycontextconstraints),
        );

        let security = SecurityData {
            cluster_roles: ResourceCollection {
                items: serialize_resources(&mg.clusterroles, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            cluster_role_bindings: ResourceCollection {
                items: serialize_resources(
                    &mg.clusterrolebindings,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            security_context_constraints: ResourceCollection {
                items: serialize_resources(
                    &mg.securitycontextconstraints,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: security_context_constraints_counts,
            },
        };

        let administration = AdministrationData {
            cluster_settings: ResourceCollection {
                items: serialize_resources(&mg.cluster_settings, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            namespaces: ResourceCollection {
                items: serialize_resources(&mg.namespaces, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            resource_quotas: ResourceCollection {
                items: serialize_resources(&mg.resourcequotas, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            limit_ranges: ResourceCollection {
                items: serialize_resources(&mg.limitranges, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            custom_resource_definitions: ResourceCollection {
                items: serialize_resources(
                    &mg.customresourcedefinitions,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            dynamic_plugins: ResourceCollection {
                items: serialize_resources(&mg.dynamicplugins, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
        };

        // Storage
        let mut pvcs_counts = HashMap::new();
        pvcs_counts.insert("pending_count".to_string(), count_pending_pvcs(&mg.pvcs));

        let mut pvs_counts = HashMap::new();
        pvs_counts.insert("unbound_count".to_string(), count_unbound_pvs(&mg.pvs));

        let mut volume_attachments_counts = HashMap::new();
        volume_attachments_counts.insert(
            "failed_count".to_string(),
            count_failed_volume_attachments(&mg.volume_attachments),
        );

        let storage = StorageData {
            pvcs: ResourceCollection {
                items: serialize_resources(&mg.pvcs, &analyzer_registry, summary_only),
                counts: pvcs_counts,
            },
            pvs: ResourceCollection {
                items: serialize_resources(&mg.pvs, &analyzer_registry, summary_only),
                counts: pvs_counts,
            },
            storage_classes: ResourceCollection {
                items: serialize_resources(&mg.storage_classes, &analyzer_registry, summary_only),
                counts: HashMap::new(),
            },
            volume_attachments: ResourceCollection {
                items: serialize_resources(
                    &mg.volume_attachments,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: volume_attachments_counts,
            },
        };

        let virtualization = VirtualizationData {
            hyperconvergeds: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.hyperconvergeds,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            kubevirts: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.kubevirts,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machines: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machines,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_instances: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_instances,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_pools: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_pools,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_exports: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_exports,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_clones: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_clones,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_snapshots: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_snapshots,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_snapshot_contents: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_snapshot_contents,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            virtual_machine_restores: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.virtual_machine_restores,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            data_volumes: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.data_volumes,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            data_sources: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.data_sources,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            data_import_crons: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.data_import_crons,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            instance_types: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.instance_types,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
            preferences: ResourceCollection {
                items: serialize_resources(
                    &mg.virtualization.preferences,
                    &analyzer_registry,
                    summary_only,
                ),
                counts: HashMap::new(),
            },
        };

        let platform = PlatformData {
            fusion_detected: mg.platform_info.fusion_detected,
            odf_detected: mg.platform_info.odf_detected,
            service_mesh_detected: mg.platform_info.service_mesh_detected,
            acm_detected: mg.platform_info.acm_detected,
            virtualization_detected: mg.platform_info.virtualization_detected,
            cpd_detected: mg.platform_info.cpd_detected,
            virtualization,
        };

        TriageMustGatherData {
            overview: OverviewSection {
                dashboard,
                cluster_health,
            },
            core: CoreSection {
                nodes,
                workloads,
                namespaces,
                events,
            },
            compute,
            security,
            administration,
            infrastructure: InfrastructureSection {
                networking,
                storage,
            },
            platform,
        }
    }
}

/// Generate the HTML page with embedded React app and triage-based data
pub fn generate_html(mg: &MustGather) -> Result<String> {
    let data = TriageMustGatherData::from_must_gather(mg);
    let data_json = serde_json::to_string(&data)?;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Must-Gather Explorer</title>
    <style>{}</style>
</head>
<body>
    <div id="root"></div>
    <script id="must-gather-data" type="application/json">{}</script>
    <script type="module">{}</script>
</body>
</html>"#,
        REACT_CSS, data_json, REACT_JS
    );

    Ok(html)
}

fn site_index_html() -> String {
    let asset_version = REACT_JS.len().wrapping_add(REACT_CSS.len()).to_string();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Must-Gather Explorer</title>
    <link rel="stylesheet" href="assets/index.css?v={}">
</head>
<body>
    <div id="root"></div>
    <script src="data/summary.js"></script>
    <script src="assets/index.js?v={}"></script>
</body>
</html>"#,
        asset_version, asset_version
    )
}

fn write_summary_script(output_path: &Path, data: &TriageMustGatherData) -> Result<()> {
    let data_json = serde_json::to_string(data)?;
    fs::write(
        output_path,
        format!("window.__MGA_DATA__ = {};\n", data_json),
    )?;
    Ok(())
}

fn write_detail_script(resources_dir: &Path, resource: &ResourceData) -> Result<()> {
    let resource_json = serde_json::to_string(&strip_raw_and_logs(resource))?;
    fs::write(
        resources_dir.join(format!("{}.js", resource.uid)),
        format!(
            "window.__MGA_RESOURCE_DETAILS__ = window.__MGA_RESOURCE_DETAILS__ || {{}};\nwindow.__MGA_RESOURCE_DETAILS__[\"{}\"] = {};\n",
            resource.uid, resource_json
        ),
    )?;
    Ok(())
}

fn write_raw_script(raw_dir: &Path, resource: &ResourceData) -> Result<()> {
    let raw_json = serde_json::to_string(&resource.raw)?;
    fs::write(
        raw_dir.join(format!("{}.js", resource.uid)),
        format!(
            "window.__MGA_RESOURCE_RAW__ = window.__MGA_RESOURCE_RAW__ || {{}};\nwindow.__MGA_RESOURCE_RAW__[\"{}\"] = {};\n",
            resource.uid, raw_json
        ),
    )?;
    Ok(())
}

fn write_logs_script(logs_dir: &Path, resource: &ResourceData) -> Result<()> {
    if resource.logs_path.is_none() {
        return Ok(());
    }

    let logs_json = serde_json::to_string(&resource.logs)?;
    fs::write(
        logs_dir.join(format!("{}.js", resource.uid)),
        format!(
            "window.__MGA_RESOURCE_LOGS__ = window.__MGA_RESOURCE_LOGS__ || {{}};\nwindow.__MGA_RESOURCE_LOGS__[\"{}\"] = {};\n",
            resource.uid, logs_json
        ),
    )?;
    Ok(())
}

fn write_resource_collection_details(
    resources_dir: &Path,
    collection: &ResourceCollection,
) -> Result<()> {
    for resource in &collection.items {
        write_detail_script(resources_dir, resource)?;
    }
    Ok(())
}

fn write_resource_details(resources_dir: &Path, resources: &[ResourceData]) -> Result<()> {
    for resource in resources {
        write_detail_script(resources_dir, resource)?;
    }
    Ok(())
}

fn write_resource_collection_logs(logs_dir: &Path, collection: &ResourceCollection) -> Result<()> {
    for resource in &collection.items {
        write_logs_script(logs_dir, resource)?;
    }
    Ok(())
}

fn write_resource_collection_raw(raw_dir: &Path, collection: &ResourceCollection) -> Result<()> {
    for resource in &collection.items {
        write_raw_script(raw_dir, resource)?;
    }
    Ok(())
}

fn write_resource_raw(raw_dir: &Path, resources: &[ResourceData]) -> Result<()> {
    for resource in resources {
        write_raw_script(raw_dir, resource)?;
    }
    Ok(())
}

fn write_site_details(output_dir: &Path, data: &TriageMustGatherData) -> Result<()> {
    let resources_dir = output_dir.join("data/resources");
    let raw_dir = output_dir.join("data/raw");
    let logs_dir = output_dir.join("data/logs");
    fs::create_dir_all(&resources_dir)?;
    fs::create_dir_all(&raw_dir)?;
    fs::create_dir_all(&logs_dir)?;

    write_resource_details(&resources_dir, &data.overview.cluster_health.operators)?;
    write_resource_collection_details(
        &resources_dir,
        &data.overview.cluster_health.installed_operators,
    )?;
    write_resource_collection_details(&resources_dir, &data.core.nodes)?;
    write_resource_collection_details(&resources_dir, &data.core.namespaces)?;
    write_resource_collection_details(&resources_dir, &data.core.events)?;
    write_resource_collection_details(&resources_dir, &data.compute.machines)?;
    write_resource_collection_details(&resources_dir, &data.compute.machine_health_checks)?;
    write_resource_collection_details(&resources_dir, &data.compute.machine_autoscalers)?;
    write_resource_collection_details(&resources_dir, &data.compute.control_plane_machine_sets)?;
    write_resource_collection_details(&resources_dir, &data.compute.machine_sets)?;
    write_resource_collection_details(&resources_dir, &data.compute.machine_configs)?;
    write_resource_collection_details(&resources_dir, &data.compute.machine_config_pools)?;
    write_resource_collection_details(&resources_dir, &data.compute.machine_configurations)?;
    write_resource_collection_details(&resources_dir, &data.security.cluster_roles)?;
    write_resource_collection_details(&resources_dir, &data.security.cluster_role_bindings)?;
    write_resource_collection_details(&resources_dir, &data.security.security_context_constraints)?;
    write_resource_collection_details(&resources_dir, &data.administration.cluster_settings)?;
    write_resource_collection_details(&resources_dir, &data.administration.namespaces)?;
    write_resource_collection_details(&resources_dir, &data.administration.resource_quotas)?;
    write_resource_collection_details(&resources_dir, &data.administration.limit_ranges)?;
    write_resource_collection_details(
        &resources_dir,
        &data.administration.custom_resource_definitions,
    )?;
    write_resource_collection_details(&resources_dir, &data.administration.dynamic_plugins)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.pods)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.deployments)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.statefulsets)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.configmaps)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.secrets)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.daemonsets)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.jobs)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.cronjobs)?;
    write_resource_collection_details(&resources_dir, &data.core.workloads.replicasets)?;
    write_resource_collection_details(&resources_dir, &data.infrastructure.networking.routes)?;
    write_resource_collection_details(&resources_dir, &data.infrastructure.networking.services)?;
    write_resource_collection_details(&resources_dir, &data.infrastructure.networking.endpoints)?;
    write_resource_collection_details(
        &resources_dir,
        &data.infrastructure.networking.network_policies,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.infrastructure.networking.ingress_controllers,
    )?;
    write_resource_collection_details(&resources_dir, &data.infrastructure.storage.pvcs)?;
    write_resource_collection_details(&resources_dir, &data.infrastructure.storage.pvs)?;
    write_resource_collection_details(
        &resources_dir,
        &data.infrastructure.storage.storage_classes,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.infrastructure.storage.volume_attachments,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.hyperconvergeds,
    )?;
    write_resource_collection_details(&resources_dir, &data.platform.virtualization.kubevirts)?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machines,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machine_instances,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machine_pools,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machine_exports,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machine_clones,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machine_snapshots,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data
            .platform
            .virtualization
            .virtual_machine_snapshot_contents,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.virtual_machine_restores,
    )?;
    write_resource_collection_details(&resources_dir, &data.platform.virtualization.data_volumes)?;
    write_resource_collection_details(&resources_dir, &data.platform.virtualization.data_sources)?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.data_import_crons,
    )?;
    write_resource_collection_details(
        &resources_dir,
        &data.platform.virtualization.instance_types,
    )?;
    write_resource_collection_details(&resources_dir, &data.platform.virtualization.preferences)?;

    write_resource_raw(&raw_dir, &data.overview.cluster_health.operators)?;
    write_resource_collection_raw(&raw_dir, &data.overview.cluster_health.installed_operators)?;
    write_resource_collection_raw(&raw_dir, &data.core.nodes)?;
    write_resource_collection_raw(&raw_dir, &data.core.namespaces)?;
    write_resource_collection_raw(&raw_dir, &data.core.events)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machines)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machine_health_checks)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machine_autoscalers)?;
    write_resource_collection_raw(&raw_dir, &data.compute.control_plane_machine_sets)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machine_sets)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machine_configs)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machine_config_pools)?;
    write_resource_collection_raw(&raw_dir, &data.compute.machine_configurations)?;
    write_resource_collection_raw(&raw_dir, &data.security.cluster_roles)?;
    write_resource_collection_raw(&raw_dir, &data.security.cluster_role_bindings)?;
    write_resource_collection_raw(&raw_dir, &data.security.security_context_constraints)?;
    write_resource_collection_raw(&raw_dir, &data.administration.cluster_settings)?;
    write_resource_collection_raw(&raw_dir, &data.administration.namespaces)?;
    write_resource_collection_raw(&raw_dir, &data.administration.resource_quotas)?;
    write_resource_collection_raw(&raw_dir, &data.administration.limit_ranges)?;
    write_resource_collection_raw(&raw_dir, &data.administration.custom_resource_definitions)?;
    write_resource_collection_raw(&raw_dir, &data.administration.dynamic_plugins)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.pods)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.deployments)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.statefulsets)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.configmaps)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.secrets)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.daemonsets)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.jobs)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.cronjobs)?;
    write_resource_collection_raw(&raw_dir, &data.core.workloads.replicasets)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.networking.routes)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.networking.services)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.networking.endpoints)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.networking.network_policies)?;
    write_resource_collection_raw(
        &raw_dir,
        &data.infrastructure.networking.ingress_controllers,
    )?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.storage.pvcs)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.storage.pvs)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.storage.storage_classes)?;
    write_resource_collection_raw(&raw_dir, &data.infrastructure.storage.volume_attachments)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.hyperconvergeds)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.kubevirts)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.virtual_machines)?;
    write_resource_collection_raw(
        &raw_dir,
        &data.platform.virtualization.virtual_machine_instances,
    )?;
    write_resource_collection_raw(
        &raw_dir,
        &data.platform.virtualization.virtual_machine_pools,
    )?;
    write_resource_collection_raw(
        &raw_dir,
        &data.platform.virtualization.virtual_machine_exports,
    )?;
    write_resource_collection_raw(
        &raw_dir,
        &data.platform.virtualization.virtual_machine_clones,
    )?;
    write_resource_collection_raw(
        &raw_dir,
        &data.platform.virtualization.virtual_machine_snapshots,
    )?;
    write_resource_collection_raw(
        &raw_dir,
        &data
            .platform
            .virtualization
            .virtual_machine_snapshot_contents,
    )?;
    write_resource_collection_raw(
        &raw_dir,
        &data.platform.virtualization.virtual_machine_restores,
    )?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.data_volumes)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.data_sources)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.data_import_crons)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.instance_types)?;
    write_resource_collection_raw(&raw_dir, &data.platform.virtualization.preferences)?;
    write_resource_collection_logs(&logs_dir, &data.core.workloads.pods)?;

    Ok(())
}

pub fn generate_site(output_dir: &Path, mg: &MustGather) -> Result<()> {
    let summary_data = TriageMustGatherData::from_must_gather_summary(mg);
    let full_data = TriageMustGatherData::from_must_gather(mg);

    fs::create_dir_all(output_dir)?;
    let assets_dir = output_dir.join("assets");
    let data_dir = output_dir.join("data");
    if assets_dir.exists() {
        fs::remove_dir_all(&assets_dir)?;
    }
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir)?;
    }
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&data_dir)?;

    fs::write(output_dir.join("index.html"), site_index_html())?;
    fs::write(assets_dir.join("index.js"), REACT_JS)?;
    fs::write(assets_dir.join("index.css"), REACT_CSS)?;
    write_summary_script(&data_dir.join("summary.js"), &summary_data)?;
    write_site_details(output_dir, &full_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generate_html_structure() {
        // This will be tested with actual must-gather data
        // Just verify the function signature for now
    }
}
