// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// MAX_LOG_LINES defines the maximum number of log lines that will be included in the output
const MAX_LOG_LINES: usize = 99;
const MAX_MUST_GATHER_ROOT_SEARCH_DEPTH: usize = 8;

/// Platform detection information based on namespace analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub fusion_detected: bool,
    pub odf_detected: bool,
    pub service_mesh_detected: bool,
    pub acm_detected: bool,
    pub virtualization_detected: bool,
    pub cpd_detected: bool,
}

impl PlatformInfo {
    /// Detect platforms based on namespace names
    pub fn detect(namespaces: &[Namespace]) -> Self {
        let ns_names: Vec<String> = namespaces
            .iter()
            .map(|ns| Resource::name(ns).clone())
            .collect();

        Self {
            // IBM Spectrum Fusion
            fusion_detected: ns_names
                .iter()
                .any(|n| n.contains("ibm-spectrum-fusion") || n.contains("isf-")),

            // OpenShift Data Foundation (ODF)
            odf_detected: ns_names
                .iter()
                .any(|n| n == "openshift-storage" || n.contains("ocs-")),

            // Service Mesh
            service_mesh_detected: ns_names.iter().any(|n| {
                n == "istio-system" || n == "openshift-service-mesh" || n.contains("servicemesh")
            }),

            // Advanced Cluster Management (ACM)
            acm_detected: ns_names.iter().any(|n| {
                n.contains("open-cluster-management") || n.contains("multicluster-engine")
            }),

            // OpenShift Virtualization
            virtualization_detected: ns_names.iter().any(|n| {
                n == "openshift-cnv"
                    || n == "openshift-virtualization-os-images"
                    || n.contains("kubevirt")
                    || n.contains("virtualization")
            }),

            // Cloud Pak for Data (CPD)
            cpd_detected: ns_names.iter().any(|n| {
                n.contains("cpd-") || n.contains("zen") || n.contains("ibm-common-services")
            }),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VirtualizationResources {
    pub hyperconvergeds: Vec<GenericResource>,
    pub kubevirts: Vec<GenericResource>,
    pub virtual_machines: Vec<GenericResource>,
    pub virtual_machine_instances: Vec<GenericResource>,
    pub virtual_machine_pools: Vec<GenericResource>,
    pub virtual_machine_exports: Vec<GenericResource>,
    pub virtual_machine_clones: Vec<GenericResource>,
    pub virtual_machine_snapshots: Vec<GenericResource>,
    pub virtual_machine_snapshot_contents: Vec<GenericResource>,
    pub virtual_machine_restores: Vec<GenericResource>,
    pub data_volumes: Vec<GenericResource>,
    pub data_sources: Vec<GenericResource>,
    pub data_import_crons: Vec<GenericResource>,
    pub instance_types: Vec<GenericResource>,
    pub preferences: Vec<GenericResource>,
}

impl VirtualizationResources {
    pub fn is_empty(&self) -> bool {
        self.hyperconvergeds.is_empty()
            && self.kubevirts.is_empty()
            && self.virtual_machines.is_empty()
            && self.virtual_machine_instances.is_empty()
            && self.virtual_machine_pools.is_empty()
            && self.virtual_machine_exports.is_empty()
            && self.virtual_machine_clones.is_empty()
            && self.virtual_machine_snapshots.is_empty()
            && self.virtual_machine_snapshot_contents.is_empty()
            && self.virtual_machine_restores.is_empty()
            && self.data_volumes.is_empty()
            && self.data_sources.is_empty()
            && self.data_import_crons.is_empty()
            && self.instance_types.is_empty()
            && self.preferences.is_empty()
    }
}

pub struct MustGather {
    pub title: String,
    pub collection_timestamp: Option<String>,
    pub version: String,
    pub platformtype: String,
    pub clusteroperators: Vec<ClusterOperator>,
    pub operators: Vec<GenericResource>,
    pub clusterroles: Vec<ClusterRole>,
    pub clusterrolebindings: Vec<ClusterRoleBinding>,
    pub securitycontextconstraints: Vec<SecurityContextConstraint>,
    pub machines: Vec<Machine>,
    pub machinehealthchecks: Vec<MachineHealthCheck>,
    pub machinesets: Vec<MachineSet>,
    pub machineconfigurations: Vec<MachineConfiguration>,
    pub machineconfigpools: Vec<MachineConfigPool>,
    pub machineconfigs: Vec<MachineConfig>,
    pub nodes: Vec<Node>,
    pub namespaces: Vec<Namespace>,
    pub cluster_settings: Vec<GenericResource>,
    pub resourcequotas: Vec<GenericResource>,
    pub limitranges: Vec<GenericResource>,
    pub customresourcedefinitions: Vec<GenericResource>,
    pub dynamicplugins: Vec<GenericResource>,
    pub events: Vec<Event>,
    pub csrs: Vec<CertificateSigningRequest>,
    pub clusterautoscalers: Vec<ClusterAutoscaler>,
    pub machineautoscalers: Vec<MachineAutoscaler>,
    pub baremetalhosts: Vec<BareMetalHost>,
    pub controlplanemachinesets: Vec<ControlPlaneMachineSet>,
    pub mapipods: Vec<Pod>,
    pub mcopods: Vec<Pod>,
    pub ccmopods: Vec<Pod>,
    pub ccmpods: Vec<Pod>,
    pub pods: Vec<Pod>, // All pods from all namespaces

    // Workload resources
    pub deployments: Vec<Deployment>,
    pub statefulsets: Vec<StatefulSet>,
    pub daemonsets: Vec<DaemonSet>,
    pub jobs: Vec<Job>,
    pub cronjobs: Vec<CronJob>,
    pub replicasets: Vec<ReplicaSet>,
    pub configmaps: Vec<ConfigMap>,
    pub secrets: Vec<Secret>,

    // Storage resources
    pub pvcs: Vec<PersistentVolumeClaim>,
    pub pvs: Vec<PersistentVolume>,
    pub storage_classes: Vec<StorageClass>,
    pub volume_attachments: Vec<VolumeAttachment>,

    // Networking resources
    pub routes: Vec<Route>,
    pub services: Vec<Service>,
    pub endpoints: Vec<Endpoints>,
    pub networkpolicies: Vec<NetworkPolicy>,
    pub ingress_controllers: Vec<IngressController>,

    // Platform-specific resources
    pub virtualization: VirtualizationResources,

    // Platform detection
    pub platform_info: PlatformInfo,
}

impl MustGather {
    /// Build a MustGather from a path to a directory containing the root.
    pub fn from(path: impl AsRef<Path>) -> Result<MustGather> {
        Self::from_path(path.as_ref())
    }

    pub fn from_path(path: &Path) -> Result<MustGather> {
        let path = find_must_gather_root(path)?;
        let title = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("must-gather")
            .to_string();
        let collection_timestamp = normalize_must_gather_timestamp(&title)
            .or_else(|| format_path_modified_timestamp(&path));
        let version = get_cluster_version(&path);
        let platformtype = get_cluster_platform_type(&path);

        let manifestpath =
            build_manifest_path(&path, "", "", "clusteroperators", "config.openshift.io");
        let clusteroperators = get_resources::<ClusterOperator>(&manifestpath);

        let operators = get_cluster_resources(&path, "operators.coreos.com", "operators");

        let manifestpath =
            build_manifest_path(&path, "", "", "clusterroles", "rbac.authorization.k8s.io");
        let clusterroles = get_resources::<ClusterRole>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "clusterrolebindings",
            "rbac.authorization.k8s.io",
        );
        let clusterrolebindings = get_resources::<ClusterRoleBinding>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "securitycontextconstraints",
            "security.openshift.io",
        );
        let securitycontextconstraints = get_resources::<SecurityContextConstraint>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-machine-api",
            "machines",
            "machine.openshift.io",
        );
        let machines = get_resources::<Machine>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-machine-api",
            "machinehealthchecks",
            "machine.openshift.io",
        );
        let machinehealthchecks = get_resources::<MachineHealthCheck>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-machine-api",
            "machinesets",
            "machine.openshift.io",
        );
        let machinesets = get_resources::<MachineSet>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "machineconfigurations",
            "operator.openshift.io",
        );
        let machineconfigurations = get_resources::<MachineConfiguration>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "machineconfigpools",
            "machineconfiguration.openshift.io",
        );
        let machineconfigpools = get_resources::<MachineConfigPool>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "machineconfigs",
            "machineconfiguration.openshift.io",
        );
        let machineconfigs = get_resources::<MachineConfig>(&manifestpath);

        let manifestpath = build_manifest_path(&path, "", "", "nodes", "core");
        let nodes = get_resources::<Node>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "certificatesigningrequests",
            "certificates.k8s.io",
        );
        let csrs = get_resources::<CertificateSigningRequest>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "",
            "clusterautoscalers",
            "autoscaling.openshift.io",
        );
        let clusterautoscalers = get_resources::<ClusterAutoscaler>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-machine-api",
            "machineautoscalers",
            "autoscaling.openshift.io",
        );
        let machineautoscalers = get_resources::<MachineAutoscaler>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-machine-api",
            "baremetalhosts",
            "metal3.io",
        );
        let baremetalhosts = get_resources::<BareMetalHost>(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-machine-api",
            "controlplanemachinesets",
            "machine.openshift.io",
        );
        let controlplanemachinesets = get_resources::<ControlPlaneMachineSet>(&manifestpath);

        let manifestpath = build_manifest_path(&path, "", "openshift-machine-api", "pods", "");
        let mapipods = get_pods(&manifestpath);

        let manifestpath =
            build_manifest_path(&path, "", "openshift-machine-config-operator", "pods", "");
        let mcopods = get_pods(&manifestpath);

        let manifestpath = build_manifest_path(
            &path,
            "",
            "openshift-cloud-controller-manager-operator",
            "pods",
            "",
        );
        let ccmopods = get_pods(&manifestpath);

        let manifestpath =
            build_manifest_path(&path, "", "openshift-cloud-controller-manager", "pods", "");
        let ccmpods = get_pods(&manifestpath);

        // Collect namespaces by scanning the namespaces directory
        let namespaces = get_namespaces(&path);
        let cluster_settings = get_cluster_settings(&path);
        let resourcequotas = get_namespaced_core_resources(&path, "resourcequotas");
        let limitranges = get_namespaced_core_resources(&path, "limitranges");
        let customresourcedefinitions = get_custom_resource_definitions(&path);
        let dynamicplugins = get_cluster_resources(&path, "console.openshift.io", "consoleplugins");
        let events = get_events(&path);

        // Collect workload resources
        let deployments = get_deployments(&path);
        let statefulsets = get_statefulsets(&path);
        let daemonsets = get_daemonsets(&path);
        let jobs = get_jobs(&path);
        let cronjobs = get_cronjobs(&path);
        let replicasets = get_replicasets(&path);
        let configmaps = get_configmaps(&path);
        let secrets = get_secrets(&path);

        // Collect all pods from all namespaces
        let pods = get_all_pods(&path);

        // Collect storage resources
        let pvcs = get_pvcs(&path);
        let pvs = get_pvs(&path);
        let storage_classes = get_storage_classes(&path);
        let volume_attachments = get_volume_attachments(&path);

        // Collect networking resources
        let routes = get_routes(&path);
        let services = get_services(&path);
        let endpoints = get_endpoints(&path);
        let networkpolicies = get_networkpolicies(&path);
        let ingress_controllers = get_ingress_controllers(&path);

        // Collect platform-specific resources
        let virtualization = get_virtualization_resources(&path);

        // Detect platforms
        let mut platform_info = PlatformInfo::detect(&namespaces);
        if !virtualization.is_empty() {
            platform_info.virtualization_detected = true;
        }

        Ok(MustGather {
            title,
            collection_timestamp,
            version,
            platformtype,
            clusteroperators,
            operators,
            clusterroles,
            clusterrolebindings,
            securitycontextconstraints,
            machines,
            machinehealthchecks,
            machinesets,
            machineconfigurations,
            machineconfigpools,
            machineconfigs,
            nodes,
            namespaces,
            cluster_settings,
            resourcequotas,
            limitranges,
            customresourcedefinitions,
            dynamicplugins,
            events,
            csrs,
            clusterautoscalers,
            machineautoscalers,
            baremetalhosts,
            controlplanemachinesets,
            mapipods,
            mcopods,
            ccmopods,
            ccmpods,
            pods,
            deployments,
            statefulsets,
            daemonsets,
            jobs,
            cronjobs,
            replicasets,
            configmaps,
            secrets,
            pvcs,
            pvs,
            storage_classes,
            volume_attachments,
            routes,
            services,
            endpoints,
            networkpolicies,
            ingress_controllers,
            virtualization,
            platform_info,
        })
    }
}

fn normalize_must_gather_timestamp(name: &str) -> Option<String> {
    let base = name
        .trim_end_matches(".tar.gz")
        .trim_end_matches(".tgz")
        .trim_end_matches(".zip")
        .trim_end_matches("_unpack");

    if let Some(rest) = base.strip_prefix("must-gather-") {
        let parts: Vec<&str> = rest.split('-').collect();

        if parts.len() >= 6
            && parts[0].len() == 2
            && parts[1].len() == 2
            && parts[2].len() == 4
            && parts[3].len() == 2
            && parts[4].len() == 2
            && parts[5].len() == 2
            && parts
                .iter()
                .take(6)
                .all(|p| p.chars().all(|c| c.is_ascii_digit()))
        {
            return Some(format!(
                "{}-{}-{} {}:{}:{}",
                parts[2], parts[0], parts[1], parts[3], parts[4], parts[5]
            ));
        }

        if let Some((date, time)) = rest.split_once('-') {
            if date.len() == 8
                && time.len() >= 6
                && date.chars().all(|c| c.is_ascii_digit())
                && time.chars().take(6).all(|c| c.is_ascii_digit())
            {
                return Some(format!(
                    "{}-{}-{} {}:{}:{}",
                    &date[0..4],
                    &date[4..6],
                    &date[6..8],
                    &time[0..2],
                    &time[2..4],
                    &time[4..6]
                ));
            }
        }
    }

    None
}

fn format_path_modified_timestamp(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    format_system_time_utc(modified)
}

fn format_system_time_utc(time: SystemTime) -> Option<String> {
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    let total_seconds = duration.as_secs() as i64;
    let days = total_seconds.div_euclid(86_400);
    let seconds_of_day = total_seconds.rem_euclid(86_400);

    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    Some(format!(
        "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
    ))
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// Build a path to a resource, does not guarantee that it exists.
/// If a name is provided the path will include a yaml file. If the name is
/// an empty string the path will be to the directory containing the resource
/// manifest yaml files.
/// If the namespace is an emptry string then the path will be to cluster
/// scoped resources.
/// Example - finding node resources
/// build_manifest_path(mgroot, "", "", "nodes", "core")
/// Example - finding a specific machine
/// build_manifest_path(mgroot, "machine-name", "openshift-machine-api", "machines", "machine.openshift.io")
fn build_manifest_path(
    path: &Path,
    name: &str,
    namespace: &str,
    kind: &str,
    group: &str,
) -> PathBuf {
    let mut manifestpath = path.to_path_buf();

    if namespace.is_empty() {
        manifestpath.push("cluster-scoped-resources");
    } else {
        manifestpath.push("namespaces");
        manifestpath.push(namespace);
    }

    if !group.is_empty() {
        manifestpath.push(group);
    }

    manifestpath.push(kind);

    if !name.is_empty() {
        manifestpath.push(format!("{}.yaml", name));
    }

    manifestpath
}

/// Find the root of a must-gather directory structure given a path.
///
/// Search wrapper directories for a usable root. Some unpacking tools leave empty image
/// directories or partially extracted resources alongside the complete must-gather.
fn find_must_gather_root(path: &Path) -> Result<PathBuf> {
    let mut directories = VecDeque::from([(path.to_path_buf(), 0)]);
    let mut visited = HashSet::new();
    let mut best_root: Option<(usize, u8, PathBuf)> = None;

    while let Some((candidate, depth)) = directories.pop_front() {
        if best_root
            .as_ref()
            .is_some_and(|(best_depth, _, _)| depth > best_depth.saturating_add(1))
        {
            break;
        }

        let visit_key = candidate
            .canonicalize()
            .unwrap_or_else(|_| candidate.clone());
        if !visited.insert(visit_key) {
            continue;
        }

        let score = must_gather_root_score(&candidate);
        if score > 0 {
            let replace_best = match &best_root {
                Some((best_depth, best_score, best_path)) => {
                    score > *best_score
                        || (score == *best_score
                            && (depth < *best_depth
                                || (depth == *best_depth && candidate < *best_path)))
                }
                None => true,
            };

            if replace_best {
                best_root = Some((depth, score, candidate.clone()));
            }

            if score == 8 {
                return Ok(candidate);
            }
        }

        if depth >= MAX_MUST_GATHER_ROOT_SEARCH_DEPTH
            || best_root
                .as_ref()
                .is_some_and(|(best_depth, _, _)| depth >= best_depth.saturating_add(1))
        {
            continue;
        }

        let Ok(entries) = fs::read_dir(&candidate) else {
            continue;
        };

        for entry in entries.flatten() {
            if entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                directories.push_back((entry.path(), depth + 1));
            }
        }
    }

    best_root
        .map(|(_, _, root)| root)
        .ok_or_else(|| anyhow::anyhow!("Cannot determine root of must-gather in {:?}", path))
}

fn must_gather_root_score(path: &Path) -> u8 {
    let has_version = path.join("version").is_file();
    let has_namespaces = path.join("namespaces").is_dir();
    let has_cluster_resources = path.join("cluster-scoped-resources").is_dir();

    if !has_version && !(has_namespaces && has_cluster_resources) {
        return 0;
    }

    u8::from(has_version) * 4 + u8::from(has_namespaces) * 2 + u8::from(has_cluster_resources) * 2
}

/// Get the platform type.
/// If unable to determine the platform, "Unknown" will be returned.
fn get_cluster_platform_type(path: &Path) -> String {
    let mut manifestpath =
        build_manifest_path(path, "", "", "infrastructures", "config.openshift.io");
    manifestpath.push("cluster.yaml");
    let version = match Manifest::from(manifestpath) {
        Ok(v) => v,
        Err(_) => return String::from("Unknown"),
    };
    match version.as_yaml()["status"]["platformStatus"]["type"].as_str() {
        Some(v) => String::from(v),
        None => String::from("Unknown"),
    }
}

/// Get the version string.
/// If unable to determine the version, "Unknown" will be returned.
fn get_cluster_version(path: &Path) -> String {
    let mut manifestpath =
        build_manifest_path(path, "", "", "clusterversions", "config.openshift.io");
    manifestpath.push("version.yaml");
    let version = match Manifest::from(manifestpath) {
        Ok(v) => v,
        Err(_) => return String::from("Unknown"),
    };
    match version.as_yaml()["status"]["desired"]["version"].as_str() {
        Some(v) => String::from(v),
        None => String::from("Unknown"),
    }
}

/// Get a pod from a path.
/// Will attempt to determine the pod name and containers, if it cannot
/// find the files or encounters an error, it will return None.
fn get_pod(pod_dir: &PathBuf) -> Option<Pod> {
    let manifest_yaml = match pod_dir.file_name() {
        Some(basename) => format!("{}.yaml", basename.to_str().unwrap_or("not_found")),
        None => return None,
    };

    let mut manifest_file = pod_dir.clone();
    manifest_file.push(manifest_yaml);
    let mut pod = Pod::new();
    if manifest_file.exists() {
        pod = match Manifest::from(manifest_file) {
            Ok(m) => <Pod as Resource>::from(m),
            Err(_) => return None,
        }
    }

    if let Ok(container_dirs) = fs::read_dir(pod_dir) {
        // loop through container dirs
        for container_dir in container_dirs {
            let container_dir = container_dir.ok()?.path();
            //   build path to log file
            let container_name = match container_dir.file_name() {
                Some(basename) => basename.to_str().unwrap_or("not_found"),
                None => continue,
            };
            let mut current_log_filename = container_dir.clone();
            current_log_filename.push(container_name);
            current_log_filename.push("logs");
            current_log_filename.push("current.log");
            if current_log_filename.exists() {
                //   if it exists open and read into a new string
                let raw: String = match fs::read_to_string(current_log_filename.as_path()) {
                    Ok(contents) => contents,
                    Err(_) => continue,
                };
                let mut logoutput = String::new();
                let mut revlines = raw.lines().rev();
                if revlines.clone().count() > MAX_LOG_LINES {
                    for _ in 0..MAX_LOG_LINES {
                        logoutput = match revlines.next() {
                            Some(l) => l.to_owned() + "\n" + &logoutput,
                            None => logoutput,
                        };
                    }

                    logoutput = String::from(
                        "camgi warning: log file has been truncated to end, see full contents in the must-gather\n",
                    ) + &logoutput;
                } else {
                    logoutput = raw;
                }

                //   create a Container and add it to the Pod
                pod.push_container(Container {
                    name: container_name.to_string(),
                    current_log: logoutput,
                    current_log_path: Some(current_log_filename.display().to_string()),
                });
            }
        }
    }

    Some(pod)
}

/// Get all pods in a path.
/// Pod files within a must gather also include the associated logs for each
/// container. This function will find all the pod files within a path and
/// return the structured versions.
fn get_pods(path: &Path) -> Vec<Pod> {
    let mut pods = Vec::new();

    // each pod has a subdirectory with its name
    let pod_dirs = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return pods,
    };
    let pod_dirs: Vec<PathBuf> = pod_dirs
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .collect();

    for pod_dir in pod_dirs {
        let pod = match get_pod(&pod_dir) {
            Some(p) => p,
            None => continue,
        };
        pods.push(pod);
    }

    pods
}

/// Get all the resources of a given type.
/// If the resource path does not exist, will return an empty list.
fn get_resources<T: Resource>(path: &Path) -> Vec<T> {
    let mut resources = Vec::new();
    let files = match fs::read_dir(path) {
        Ok(p) => p,
        Err(_) => return resources,
    };
    let yamlfiles: Vec<PathBuf> = files
        .into_iter()
        .flatten()
        .map(|m| m.path())
        .filter(|m| m.extension().and_then(|extension| extension.to_str()) == Some("yaml"))
        .collect();

    for path in yamlfiles {
        match Manifest::from(path) {
            Ok(m) => resources.push(T::from(m)),
            Err(_) => continue,
        }
    }
    resources
}

fn get_cluster_resources(path: &Path, api_group: &str, resource: &str) -> Vec<GenericResource> {
    let manifestpath = build_manifest_path(path, "", "", resource, api_group);
    let mut resources = get_resources::<GenericResource>(&manifestpath);
    resources.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    resources
}

fn get_custom_resource_definitions(path: &Path) -> Vec<GenericResource> {
    let mut crds = get_cluster_resources(path, "apiextensions.k8s.io", "customresourcedefinitions");
    add_custom_resource_instance_relationships(path, &mut crds);
    crds
}

fn add_custom_resource_instance_relationships(path: &Path, crds: &mut [GenericResource]) {
    for crd in crds {
        let Some(group) = crd.crd_group().map(str::to_string) else {
            continue;
        };
        let Some(plural) = crd.crd_plural().map(str::to_string) else {
            continue;
        };
        let kind = crd.crd_kind().unwrap_or(ResourceV2::kind(crd)).to_string();
        let scope = crd.crd_scope().unwrap_or("");

        let mut relationships = Vec::new();
        let mut seen = HashSet::new();

        if scope != "Namespaced" {
            let resource_dir = path
                .join("cluster-scoped-resources")
                .join(&group)
                .join(&plural);
            collect_custom_resource_instance_links(
                &resource_dir,
                &kind,
                None,
                &mut relationships,
                &mut seen,
            );
        }

        if scope != "Cluster" {
            let namespaces_path = path.join("namespaces");
            if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
                for namespace_entry in namespace_dirs.flatten() {
                    let namespace_path = namespace_entry.path();
                    if !namespace_path.is_dir() {
                        continue;
                    }
                    let namespace = namespace_entry.file_name().to_string_lossy().to_string();
                    let resource_dir = namespace_path.join(&group).join(&plural);
                    collect_custom_resource_instance_links(
                        &resource_dir,
                        &kind,
                        Some(namespace),
                        &mut relationships,
                        &mut seen,
                    );
                }
            }
        }

        crd.add_relationships(relationships);
    }
}

fn collect_custom_resource_instance_links(
    resource_dir: &Path,
    default_kind: &str,
    fallback_namespace: Option<String>,
    links: &mut Vec<ResourceLink>,
    seen: &mut HashSet<(String, String, Option<String>)>,
) {
    let Ok(entries) = fs::read_dir(resource_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }

        let Ok(manifest) = Manifest::from(path) else {
            continue;
        };
        let name = manifest.name.clone();
        if name.is_empty() || name == "Unknown" {
            continue;
        }

        let kind = manifest.as_yaml()["kind"]
            .as_str()
            .unwrap_or(default_kind)
            .to_string();
        let namespace = manifest.namespace().or_else(|| fallback_namespace.clone());
        let key = (kind.clone(), name.clone(), namespace.clone());
        if seen.insert(key) {
            links.push(ResourceLink {
                kind,
                name,
                namespace,
                relationship: RelationshipType::References,
            });
        }
    }
}

fn get_namespaced_core_resources(path: &Path, resource: &str) -> Vec<GenericResource> {
    let mut resources = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            let resource_dir = namespace_entry.path().join("core").join(resource);
            if resource_dir.is_dir() {
                resources.extend(get_resources::<GenericResource>(&resource_dir));
            }
        }
    }

    resources.sort_by(|a, b| {
        ResourceV2::namespace(a)
            .cmp(&ResourceV2::namespace(b))
            .then(Resource::name(a).cmp(Resource::name(b)))
    });
    resources
}

fn get_namespaced_list_resources(
    path: &Path,
    api_group: &str,
    resource_file: &str,
) -> Vec<GenericResource> {
    let mut resources = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            let resource_path = namespace_entry
                .path()
                .join(api_group)
                .join(format!("{resource_file}.yaml"));
            if resource_path.is_file() {
                let Ok(manifests) = Manifest::from_list(resource_path) else {
                    continue;
                };
                resources.extend(
                    manifests
                        .into_iter()
                        .map(<GenericResource as Resource>::from),
                );
            }
        }
    }

    resources.sort_by(|a, b| {
        ResourceV2::kind(a)
            .cmp(ResourceV2::kind(b))
            .then(ResourceV2::namespace(a).cmp(&ResourceV2::namespace(b)))
            .then(Resource::name(a).cmp(Resource::name(b)))
    });
    resources
}

fn get_virtualization_resources(path: &Path) -> VirtualizationResources {
    VirtualizationResources {
        hyperconvergeds: get_namespaced_list_resources(path, "hco.kubevirt.io", "hyperconvergeds"),
        kubevirts: get_namespaced_list_resources(path, "kubevirt.io", "kubevirts"),
        virtual_machines: get_namespaced_list_resources(path, "kubevirt.io", "virtualmachines"),
        virtual_machine_instances: get_namespaced_list_resources(
            path,
            "kubevirt.io",
            "virtualmachineinstances",
        ),
        virtual_machine_pools: get_namespaced_list_resources(
            path,
            "pool.kubevirt.io",
            "virtualmachinepools",
        ),
        virtual_machine_exports: get_namespaced_list_resources(
            path,
            "export.kubevirt.io",
            "virtualmachineexports",
        ),
        virtual_machine_clones: get_namespaced_list_resources(
            path,
            "clone.kubevirt.io",
            "virtualmachineclones",
        ),
        virtual_machine_snapshots: get_namespaced_list_resources(
            path,
            "snapshot.kubevirt.io",
            "virtualmachinesnapshots",
        ),
        virtual_machine_snapshot_contents: get_namespaced_list_resources(
            path,
            "snapshot.kubevirt.io",
            "virtualmachinesnapshotcontents",
        ),
        virtual_machine_restores: get_namespaced_list_resources(
            path,
            "snapshot.kubevirt.io",
            "virtualmachinerestores",
        ),
        data_volumes: get_namespaced_list_resources(path, "cdi.kubevirt.io", "datavolumes"),
        data_sources: get_namespaced_list_resources(path, "cdi.kubevirt.io", "datasources"),
        data_import_crons: get_namespaced_list_resources(
            path,
            "cdi.kubevirt.io",
            "dataimportcrons",
        ),
        instance_types: get_namespaced_list_resources(
            path,
            "instancetype.kubevirt.io",
            "virtualmachineinstancetypes",
        ),
        preferences: get_namespaced_list_resources(
            path,
            "instancetype.kubevirt.io",
            "virtualmachinepreferences",
        ),
    }
}

fn get_cluster_settings(path: &Path) -> Vec<GenericResource> {
    let mut resources = Vec::new();
    let settings_path = path
        .join("cluster-scoped-resources")
        .join("config.openshift.io");

    if let Ok(setting_dirs) = fs::read_dir(settings_path) {
        for setting_entry in setting_dirs.flatten() {
            if setting_entry.file_name() == "clusteroperators" {
                continue;
            }
            if setting_entry.path().is_dir() {
                resources.extend(get_resources::<GenericResource>(&setting_entry.path()));
            }
        }
    }

    resources.sort_by(|a, b| {
        ResourceV2::kind(a)
            .cmp(ResourceV2::kind(b))
            .then(Resource::name(a).cmp(Resource::name(b)))
    });
    resources
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_build_manifest_path_cluster_scoped() {
        assert_eq!(
            build_manifest_path(&PathBuf::from("/foo"), "", "", "nodes", "core"),
            PathBuf::from("/foo/cluster-scoped-resources/core/nodes")
        )
    }

    #[test]
    fn test_build_manifest_path_cluster_scoped_named_resource() {
        assert_eq!(
            build_manifest_path(&PathBuf::from("/foo"), "node1", "", "nodes", "core"),
            PathBuf::from("/foo/cluster-scoped-resources/core/nodes/node1.yaml")
        )
    }

    #[test]
    fn test_build_manifest_path_namespace_scoped() {
        assert_eq!(
            build_manifest_path(
                &PathBuf::from("/foo"),
                "",
                "openshift-machine-api",
                "machines",
                "machine.openshift.io"
            ),
            PathBuf::from("/foo/namespaces/openshift-machine-api/machine.openshift.io/machines")
        )
    }

    #[test]
    fn test_build_manifest_path_namespace_scoped_named_resource() {
        assert_eq!(
            build_manifest_path(
                &PathBuf::from("/foo"),
                "machine1",
                "openshift-machine-api",
                "machines",
                "machine.openshift.io"
            ),
            PathBuf::from(
                "/foo/namespaces/openshift-machine-api/machine.openshift.io/machines/machine1.yaml"
            )
        )
    }

    #[test]
    fn test_get_cluster_version() {
        assert_eq!(
            get_cluster_version(&PathBuf::from(
                "testdata/must-gather-valid/sample-openshift-release"
            )),
            "X.Y.Z-fake-test"
        )
    }

    #[test]
    fn test_get_pod_containers_count() {
        let path = PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/pods/machine-api-controllers-86c6c8f96d-ssrp8",
        );
        let pod = get_pod(&path).unwrap();
        assert_eq!(pod.containers.len(), 7)
    }

    #[test]
    fn test_get_pod_log_file() {
        let path = PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/pods/cluster-baremetal-operator-6955c869bf-9ccgk",
        );
        let pod = get_pod(&path).unwrap();
        assert_eq!(pod.containers.len(), 1);
        let expected = include_str!(
            "../testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/pods/cluster-baremetal-operator-6955c869bf-9ccgk/cluster-baremetal-operator/cluster-baremetal-operator/logs/current.log"
        );
        assert_eq!(pod.containers[0].current_log.as_str(), expected);
    }

    #[test]
    fn test_get_pod_log_file_truncated() {
        let path = PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/pods/cluster-autoscaler-default-f548ffc66-bck7p",
        );
        let pod = get_pod(&path).unwrap();
        assert_eq!(pod.containers.len(), 1);
        let expected = include_str!("../testdata/truncated-log-file.txt");
        assert_eq!(pod.containers[0].current_log.as_str(), expected);
    }

    #[test]
    fn test_get_pods_success() {
        let path = PathBuf::from("testdata/must-gather-valid/sample-openshift-release");
        let manifestpath = build_manifest_path(&path, "", "openshift-machine-api", "pods", "");
        assert_eq!(get_pods(&manifestpath).len(), 4)
    }

    #[test]
    fn test_get_resources_success() {
        let path = PathBuf::from("testdata/must-gather-valid/sample-openshift-release");
        let manifestpath = build_manifest_path(&path, "", "", "nodes", "core");
        assert_eq!(get_resources::<Node>(&manifestpath).len(), 4)
    }

    #[test]
    fn test_get_resources_non_existant() {
        let path = PathBuf::from("testdata/must-gather-invalid/sample-openshift-release");
        let manifestpath = build_manifest_path(&path, "", "fake", "kind", "group");
        assert_eq!(get_resources::<Node>(&manifestpath).len(), 0)
    }

    #[test]
    fn test_find_must_gather_root_skips_empty_image_directory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let wrapper = std::env::temp_dir().join(format!("mga-root-search-test-{}", unique));
        let incomplete_root = wrapper.join("quay-image-short-digest");
        let complete_root = wrapper.join("quay-image-full-digest");
        fs::create_dir_all(&incomplete_root).unwrap();
        fs::create_dir_all(complete_root.join("namespaces")).unwrap();
        fs::create_dir_all(complete_root.join("cluster-scoped-resources")).unwrap();
        fs::write(complete_root.join("version"), "test-version").unwrap();

        assert_eq!(find_must_gather_root(&wrapper).unwrap(), complete_root);
    }

    #[test]
    fn test_find_must_gather_root_prefers_complete_child_over_partial_wrapper() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let wrapper =
            std::env::temp_dir().join(format!("mga-root-preference-test-{}", unique));
        let complete_root = wrapper.join("quay-image-full-digest");
        fs::create_dir_all(wrapper.join("namespaces")).unwrap();
        fs::create_dir_all(wrapper.join("cluster-scoped-resources")).unwrap();
        fs::create_dir_all(complete_root.join("namespaces")).unwrap();
        fs::create_dir_all(complete_root.join("cluster-scoped-resources")).unwrap();
        fs::write(complete_root.join("version"), "test-version").unwrap();

        assert_eq!(find_must_gather_root(&wrapper).unwrap(), complete_root);
    }

    #[test]
    fn test_get_resources_ignores_files_without_extensions() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let manifest_dir =
            std::env::temp_dir().join(format!("mga-resource-files-test-{}", unique));
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(manifest_dir.join("README"), "not a manifest").unwrap();
        fs::write(
            manifest_dir.join("test-node.yaml"),
            r#"apiVersion: v1
kind: Node
metadata:
  name: test-node
"#,
        )
        .unwrap();

        assert_eq!(get_resources::<Node>(&manifest_dir).len(), 1);
    }

    #[test]
    fn test_get_namespaces_loads_namespace_manifest_yaml() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-mustgather-test-{}", unique));
        let manifest_dir = root.join("namespaces").join("test-ns");
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("test-ns.yaml"),
            r#"apiVersion: v1
kind: Namespace
metadata:
  name: test-ns
  uid: 5678
status:
  phase: Active
"#,
        )
        .unwrap();

        let namespaces = get_namespaces(&root);

        assert_eq!(namespaces.len(), 1);
        assert_eq!(Resource::name(&namespaces[0]), "test-ns");
        assert!(Resource::raw(&namespaces[0]).contains("kind: Namespace"));
    }

    #[test]
    fn test_get_namespaces_loads_legacy_core_namespace_manifest_yaml() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-mustgather-test-legacy-{}", unique));
        let manifest_dir = root
            .join("namespaces")
            .join("test-ns")
            .join("core")
            .join("namespaces");
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("test-ns.yaml"),
            r#"apiVersion: v1
kind: Namespace
metadata:
  name: test-ns
  uid: 9012
status:
  phase: Active
"#,
        )
        .unwrap();

        let namespaces = get_namespaces(&root);

        assert_eq!(namespaces.len(), 1);
        assert_eq!(Resource::name(&namespaces[0]), "test-ns");
        assert!(Resource::raw(&namespaces[0]).contains("kind: Namespace"));
    }

    #[test]
    fn test_get_namespaced_core_resources_loads_administration_resources() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-admin-resource-test-{}", unique));
        let manifest_dir = root
            .join("namespaces")
            .join("test-ns")
            .join("core")
            .join("resourcequotas");
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("test-quota.yaml"),
            r#"apiVersion: v1
kind: ResourceQuota
metadata:
  name: test-quota
  namespace: test-ns
"#,
        )
        .unwrap();

        let resources = get_namespaced_core_resources(&root, "resourcequotas");

        assert_eq!(resources.len(), 1);
        assert_eq!(Resource::name(&resources[0]), "test-quota");
        assert_eq!(ResourceV2::kind(&resources[0]), "ResourceQuota");
        assert_eq!(ResourceV2::namespace(&resources[0]), Some("test-ns"));
    }

    #[test]
    fn test_get_cluster_resources_loads_olm_operators() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-operator-resource-test-{}", unique));
        let manifest_dir = root
            .join("cluster-scoped-resources")
            .join("operators.coreos.com")
            .join("operators");
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("isf-operator.ibm-spectrum-fusion-ns.yaml"),
            r#"apiVersion: operators.coreos.com/v1
kind: Operator
metadata:
  name: isf-operator.ibm-spectrum-fusion-ns
"#,
        )
        .unwrap();

        let resources = get_cluster_resources(&root, "operators.coreos.com", "operators");

        assert_eq!(resources.len(), 1);
        assert_eq!(
            Resource::name(&resources[0]),
            "isf-operator.ibm-spectrum-fusion-ns"
        );
        assert_eq!(ResourceV2::kind(&resources[0]), "Operator");
    }

    #[test]
    fn test_get_custom_resource_definitions_links_custom_resource_instances() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-crd-instance-test-{}", unique));
        let crd_dir = root
            .join("cluster-scoped-resources")
            .join("apiextensions.k8s.io")
            .join("customresourcedefinitions");
        let volume_dir = root
            .join("namespaces")
            .join("longhorn-system")
            .join("longhorn.io")
            .join("volumes");
        fs::create_dir_all(&crd_dir).unwrap();
        fs::create_dir_all(&volume_dir).unwrap();
        fs::write(
            crd_dir.join("volumes.longhorn.io.yaml"),
            r#"apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: volumes.longhorn.io
spec:
  group: longhorn.io
  names:
    kind: Volume
    plural: volumes
  scope: Namespaced
"#,
        )
        .unwrap();
        fs::write(
            volume_dir.join("pvc-98204ab1-e4c2-4d6f-8de1-f389ecd91a14.yaml"),
            r#"apiVersion: longhorn.io/v1beta2
kind: Volume
metadata:
  name: pvc-98204ab1-e4c2-4d6f-8de1-f389ecd91a14
  namespace: longhorn-system
status:
  state: attached
"#,
        )
        .unwrap();

        let resources = get_custom_resource_definitions(&root);
        let crd = resources
            .iter()
            .find(|resource| Resource::name(*resource) == "volumes.longhorn.io")
            .unwrap();
        let relationships = crd.relationships();

        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].kind, "Volume");
        assert_eq!(
            relationships[0].name,
            "pvc-98204ab1-e4c2-4d6f-8de1-f389ecd91a14"
        );
        assert_eq!(
            relationships[0].namespace.as_deref(),
            Some("longhorn-system")
        );
        assert_eq!(relationships[0].relationship, RelationshipType::References);
    }

    #[test]
    fn test_get_virtualization_resources_loads_namespaced_list_files() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-virt-resource-test-{}", unique));
        let vm_dir = root.join("namespaces").join("test-vms").join("kubevirt.io");
        let hco_dir = root
            .join("namespaces")
            .join("openshift-cnv")
            .join("hco.kubevirt.io");
        fs::create_dir_all(&vm_dir).unwrap();
        fs::create_dir_all(&hco_dir).unwrap();
        fs::write(
            vm_dir.join("virtualmachines.yaml"),
            r#"apiVersion: kubevirt.io/v1
kind: VirtualMachineList
items:
- apiVersion: kubevirt.io/v1
  kind: VirtualMachine
  metadata:
    name: test-vm
    namespace: test-vms
"#,
        )
        .unwrap();
        fs::write(
            hco_dir.join("hyperconvergeds.yaml"),
            r#"apiVersion: hco.kubevirt.io/v1beta1
kind: HyperConvergedList
items:
- apiVersion: hco.kubevirt.io/v1beta1
  kind: HyperConverged
  metadata:
    name: kubevirt-hyperconverged
    namespace: openshift-cnv
  status:
    conditions:
    - type: Available
      status: "True"
"#,
        )
        .unwrap();

        let resources = get_virtualization_resources(&root);

        assert_eq!(resources.virtual_machines.len(), 1);
        assert_eq!(Resource::name(&resources.virtual_machines[0]), "test-vm");
        assert_eq!(
            ResourceV2::namespace(&resources.virtual_machines[0]),
            Some("test-vms")
        );
        assert_eq!(resources.hyperconvergeds.len(), 1);
        assert_eq!(
            ResourceV2::kind(&resources.hyperconvergeds[0]),
            "HyperConverged"
        );
        assert!(!resources.is_empty());
    }

    #[test]
    fn test_get_cluster_settings_excludes_cluster_operators() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mga-cluster-settings-test-{}", unique));
        let settings_dir = root
            .join("cluster-scoped-resources")
            .join("config.openshift.io");
        let authentication_dir = settings_dir.join("authentications");
        let cluster_operators_dir = settings_dir.join("clusteroperators");
        fs::create_dir_all(&authentication_dir).unwrap();
        fs::create_dir_all(&cluster_operators_dir).unwrap();
        fs::write(
            authentication_dir.join("cluster.yaml"),
            r#"apiVersion: config.openshift.io/v1
kind: Authentication
metadata:
  name: cluster
"#,
        )
        .unwrap();
        fs::write(
            cluster_operators_dir.join("authentication.yaml"),
            r#"apiVersion: config.openshift.io/v1
kind: ClusterOperator
metadata:
  name: authentication
"#,
        )
        .unwrap();

        let resources = get_cluster_settings(&root);

        assert_eq!(resources.len(), 1);
        assert_eq!(ResourceV2::kind(&resources[0]), "Authentication");
    }
}

/// Get all namespaces by scanning the namespaces directory
fn get_namespaces(path: &Path) -> Vec<Namespace> {
    let mut namespaces = Vec::new();

    // Find the namespaces directory
    let namespaces_path = path.join("namespaces");
    if !namespaces_path.exists() {
        return namespaces;
    }

    // Read all subdirectories (each is a namespace)
    let namespace_dirs = match fs::read_dir(&namespaces_path) {
        Ok(entries) => entries,
        Err(_) => return namespaces,
    };

    for entry in namespace_dirs {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        namespaces.push(get_namespace(&path, name_str));
                    }
                }
            }
        }
    }

    // Sort namespaces by name
    namespaces.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));

    namespaces
}

fn get_namespace(path: &Path, namespace_name: &str) -> Namespace {
    let manifest_path = path.join(format!("{}.yaml", namespace_name));

    if manifest_path.is_file() {
        if let Ok(manifest) = Manifest::from(manifest_path) {
            return <Namespace as Resource>::from(manifest);
        }
    }

    let manifest_dir = path.join("core").join("namespaces");
    let manifest_path = manifest_dir.join(format!("{}.yaml", namespace_name));

    if manifest_path.is_file() {
        if let Ok(manifest) = Manifest::from(manifest_path) {
            return <Namespace as Resource>::from(manifest);
        }
    }

    if manifest_dir.is_dir() {
        if let Some(namespace) = get_resources::<Namespace>(&manifest_dir)
            .into_iter()
            .find(|ns| ResourceV2::name(ns) == namespace_name)
        {
            return namespace;
        }
    }

    let manifest_list_path = path.join("core").join("namespaces.yaml");
    if manifest_list_path.is_file() {
        if let Ok(manifests) = Manifest::from_list(manifest_list_path) {
            if let Some(manifest) = manifests
                .into_iter()
                .find(|manifest| manifest.name == namespace_name)
            {
                return <Namespace as Resource>::from(manifest);
            }
        }
    }

    Namespace::new(namespace_name.to_string())
}

/// Get all deployments across all namespaces
fn get_deployments(path: &Path) -> Vec<Deployment> {
    let mut deployments = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let apps_path = namespace_entry.path().join("apps");

                // Check for individual deployment files in a directory
                let deployment_dir = apps_path.join("deployments");
                if deployment_dir.exists() && deployment_dir.is_dir() {
                    deployments.extend(get_resources::<Deployment>(&deployment_dir));
                }

                // Check for a single deployments.yaml file (List format)
                let deployment_file = apps_path.join("deployments.yaml");
                if deployment_file.exists() && deployment_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(deployment_file) {
                        for manifest in manifests {
                            deployments.push(<Deployment as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    deployments.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    deployments
}

/// Get all statefulsets across all namespaces
fn get_statefulsets(path: &Path) -> Vec<StatefulSet> {
    let mut statefulsets = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let apps_path = namespace_entry.path().join("apps");

                // Check for individual statefulset files in a directory
                let statefulset_dir = apps_path.join("statefulsets");
                if statefulset_dir.exists() && statefulset_dir.is_dir() {
                    statefulsets.extend(get_resources::<StatefulSet>(&statefulset_dir));
                }

                // Check for a single statefulsets.yaml file (List format)
                let statefulset_file = apps_path.join("statefulsets.yaml");
                if statefulset_file.exists() && statefulset_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(statefulset_file) {
                        for manifest in manifests {
                            statefulsets.push(<StatefulSet as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    statefulsets.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    statefulsets
}

/// Get all daemonsets across all namespaces
fn get_daemonsets(path: &Path) -> Vec<DaemonSet> {
    let mut daemonsets = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let apps_path = namespace_entry.path().join("apps");

                // Check for individual daemonset files in a directory
                let daemonset_dir = apps_path.join("daemonsets");
                if daemonset_dir.exists() && daemonset_dir.is_dir() {
                    daemonsets.extend(get_resources::<DaemonSet>(&daemonset_dir));
                }

                // Check for a single daemonsets.yaml file (List format)
                let daemonset_file = apps_path.join("daemonsets.yaml");
                if daemonset_file.exists() && daemonset_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(daemonset_file) {
                        for manifest in manifests {
                            daemonsets.push(<DaemonSet as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    daemonsets.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    daemonsets
}

/// Get all jobs across all namespaces
fn get_jobs(path: &Path) -> Vec<Job> {
    let mut jobs = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let batch_path = namespace_entry.path().join("batch");

                // Check for individual job files in a directory
                let job_dir = batch_path.join("jobs");
                if job_dir.exists() && job_dir.is_dir() {
                    jobs.extend(get_resources::<Job>(&job_dir));
                }

                // Check for a single jobs.yaml file (List format)
                let job_file = batch_path.join("jobs.yaml");
                if job_file.exists() && job_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(job_file) {
                        for manifest in manifests {
                            jobs.push(<Job as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    jobs.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    jobs
}

/// Get all cronjobs across all namespaces
fn get_cronjobs(path: &Path) -> Vec<CronJob> {
    let mut cronjobs = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let batch_path = namespace_entry.path().join("batch");

                // Check for individual cronjob files in a directory
                let cronjob_dir = batch_path.join("cronjobs");
                if cronjob_dir.exists() && cronjob_dir.is_dir() {
                    cronjobs.extend(get_resources::<CronJob>(&cronjob_dir));
                }

                // Check for a single cronjobs.yaml file (List format)
                let cronjob_file = batch_path.join("cronjobs.yaml");
                if cronjob_file.exists() && cronjob_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(cronjob_file) {
                        for manifest in manifests {
                            cronjobs.push(<CronJob as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    cronjobs.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    cronjobs
}

/// Get all replicasets across all namespaces
fn get_replicasets(path: &Path) -> Vec<ReplicaSet> {
    let mut replicasets = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let apps_path = namespace_entry.path().join("apps");

                // Check for individual replicaset files in a directory
                let replicaset_dir = apps_path.join("replicasets");
                if replicaset_dir.exists() && replicaset_dir.is_dir() {
                    replicasets.extend(get_resources::<ReplicaSet>(&replicaset_dir));
                }

                // Check for a single replicasets.yaml file (List format)
                let replicaset_file = apps_path.join("replicasets.yaml");
                if replicaset_file.exists() && replicaset_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(replicaset_file) {
                        for manifest in manifests {
                            replicasets.push(<ReplicaSet as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    replicasets.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    replicasets
}

/// Get all pods across all namespaces
fn get_all_pods(path: &Path) -> Vec<Pod> {
    let mut pods = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let pods_path = namespace_entry.path().join("pods");
                if pods_path.exists() {
                    pods.extend(get_pods(&pods_path));
                }
            }
        }
    }

    pods.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    pods
}

/// Get all Events across all namespaces
fn get_events(path: &Path) -> Vec<Event> {
    let mut events = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let core_path = namespace_entry.path().join("core");
                let events_file = core_path.join("events.yaml");
                if events_file.exists() && events_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(events_file) {
                        for manifest in manifests {
                            events.push(<Event as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    events.sort_by(|a, b| {
        crate::resources::ResourceV2::namespace(a)
            .cmp(&crate::resources::ResourceV2::namespace(b))
            .then(crate::resources::ResourceV2::name(a).cmp(crate::resources::ResourceV2::name(b)))
    });
    events
}

/// Get all ConfigMaps across all namespaces
fn get_configmaps(path: &Path) -> Vec<ConfigMap> {
    let mut configmaps = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let core_path = namespace_entry.path().join("core");

                let configmap_dir = core_path.join("configmaps");
                if configmap_dir.exists() && configmap_dir.is_dir() {
                    configmaps.extend(get_resources::<ConfigMap>(&configmap_dir));
                }

                let configmap_file = core_path.join("configmaps.yaml");
                if configmap_file.exists() && configmap_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(configmap_file) {
                        for manifest in manifests {
                            configmaps.push(<ConfigMap as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    configmaps.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    configmaps
}

/// Get all Secrets across all namespaces
fn get_secrets(path: &Path) -> Vec<Secret> {
    let mut secrets = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let core_path = namespace_entry.path().join("core");

                let secret_dir = core_path.join("secrets");
                if secret_dir.exists() && secret_dir.is_dir() {
                    secrets.extend(get_resources::<Secret>(&secret_dir));
                }

                let secret_file = core_path.join("secrets.yaml");
                if secret_file.exists() && secret_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(secret_file) {
                        for manifest in manifests {
                            secrets.push(<Secret as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    secrets.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    secrets
}

/// Get all PVCs across all namespaces
fn get_pvcs(path: &Path) -> Vec<PersistentVolumeClaim> {
    let mut pvcs = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let core_path = namespace_entry.path().join("core");

                // Check for individual PVC files in a directory
                let pvc_dir = core_path.join("persistentvolumeclaims");
                if pvc_dir.exists() && pvc_dir.is_dir() {
                    pvcs.extend(get_resources::<PersistentVolumeClaim>(&pvc_dir));
                }

                // Check for a single persistentvolumeclaims.yaml file (List format)
                let pvc_file = core_path.join("persistentvolumeclaims.yaml");
                if pvc_file.exists() && pvc_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(pvc_file) {
                        for manifest in manifests {
                            pvcs.push(<PersistentVolumeClaim as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    if pvcs.is_empty() {
        let pvs = get_pvs(path);
        for pv in pvs {
            let key_fields = crate::resources::ResourceV2::key_fields(&pv);
            let Some(claim_ref) = key_fields.get("claim_ref") else {
                continue;
            };

            let mut parts = claim_ref.splitn(2, '/');
            let Some(namespace) = parts.next() else {
                continue;
            };
            let Some(name) = parts.next() else {
                continue;
            };

            pvcs.push(PersistentVolumeClaim::synthetic_bound(
                name.to_string(),
                namespace.to_string(),
                crate::resources::ResourceV2::name(&pv).to_string(),
                key_fields.get("storage_class").cloned(),
                key_fields.get("capacity").cloned(),
            ));
        }
    }

    pvcs.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    pvcs.dedup_by(|a, b| {
        Resource::name(a) == Resource::name(b)
            && crate::resources::ResourceV2::namespace(a)
                == crate::resources::ResourceV2::namespace(b)
    });
    pvcs
}

/// Get all PVs (cluster-scoped)
fn get_pvs(path: &Path) -> Vec<PersistentVolume> {
    let manifestpath = build_manifest_path(path, "", "", "persistentvolumes", "core");
    let mut pvs = get_resources::<PersistentVolume>(&manifestpath);
    pvs.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    pvs
}

/// Get all storage classes (cluster-scoped)
fn get_storage_classes(path: &Path) -> Vec<StorageClass> {
    let manifestpath = build_manifest_path(path, "", "", "storageclasses", "storage.k8s.io");
    let mut storage_classes = get_resources::<StorageClass>(&manifestpath);
    storage_classes.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    storage_classes
}

/// Get all volume attachments (cluster-scoped)
fn get_volume_attachments(path: &Path) -> Vec<VolumeAttachment> {
    let manifestpath = build_manifest_path(path, "", "", "volumeattachments", "storage.k8s.io");
    let mut volume_attachments = get_resources::<VolumeAttachment>(&manifestpath);
    volume_attachments.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    volume_attachments
}

/// Get all routes across all namespaces
fn get_routes(path: &Path) -> Vec<Route> {
    let mut routes = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let route_openshift_path = namespace_entry.path().join("route.openshift.io");

                // Check for individual route files in a directory
                let route_dir = route_openshift_path.join("routes");
                if route_dir.exists() && route_dir.is_dir() {
                    routes.extend(get_resources::<Route>(&route_dir));
                }

                // Check for a single routes.yaml file (List format)
                let route_file = route_openshift_path.join("routes.yaml");
                if route_file.exists() && route_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(route_file) {
                        for manifest in manifests {
                            routes.push(<Route as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    routes.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    routes
}

/// Get all services across all namespaces
fn get_services(path: &Path) -> Vec<Service> {
    let mut services = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let core_path = namespace_entry.path().join("core");

                // Check for individual service files in a directory
                let service_dir = core_path.join("services");
                if service_dir.exists() && service_dir.is_dir() {
                    services.extend(get_resources::<Service>(&service_dir));
                }

                // Check for a single services.yaml file (List format)
                let service_file = core_path.join("services.yaml");
                if service_file.exists() && service_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(service_file) {
                        for manifest in manifests {
                            services.push(<Service as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    services.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    services
}

/// Get all endpoints across all namespaces
fn get_endpoints(path: &Path) -> Vec<Endpoints> {
    let mut endpoints = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let core_path = namespace_entry.path().join("core");

                // Check for individual endpoint files in a directory
                let endpoint_dir = core_path.join("endpoints");
                if endpoint_dir.exists() && endpoint_dir.is_dir() {
                    endpoints.extend(get_resources::<Endpoints>(&endpoint_dir));
                }

                // Check for a single endpoints.yaml file (List format)
                let endpoint_file = core_path.join("endpoints.yaml");
                if endpoint_file.exists() && endpoint_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(endpoint_file) {
                        for manifest in manifests {
                            endpoints.push(<Endpoints as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    endpoints.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    endpoints
}

/// Get all network policies across all namespaces
fn get_networkpolicies(path: &Path) -> Vec<NetworkPolicy> {
    let mut networkpolicies = Vec::new();
    let namespaces_path = path.join("namespaces");

    if let Ok(namespace_dirs) = fs::read_dir(&namespaces_path) {
        for namespace_entry in namespace_dirs.flatten() {
            if namespace_entry.path().is_dir() {
                let networking_path = namespace_entry.path().join("networking.k8s.io");

                let networkpolicy_dir = networking_path.join("networkpolicies");
                if networkpolicy_dir.exists() && networkpolicy_dir.is_dir() {
                    networkpolicies.extend(get_resources::<NetworkPolicy>(&networkpolicy_dir));
                }

                let networkpolicy_file = networking_path.join("networkpolicies.yaml");
                if networkpolicy_file.exists() && networkpolicy_file.is_file() {
                    if let Ok(manifests) = Manifest::from_list(networkpolicy_file) {
                        for manifest in manifests {
                            networkpolicies.push(<NetworkPolicy as Resource>::from(manifest));
                        }
                    }
                }
            }
        }
    }

    networkpolicies.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    networkpolicies
}

/// Get all ingress controllers (cluster-scoped)
fn get_ingress_controllers(path: &Path) -> Vec<IngressController> {
    let manifestpath =
        build_manifest_path(path, "", "", "ingresscontrollers", "operator.openshift.io");
    let mut ingress_controllers = get_resources::<IngressController>(&manifestpath);
    ingress_controllers.sort_by(|a, b| Resource::name(a).cmp(Resource::name(b)));
    ingress_controllers
}
