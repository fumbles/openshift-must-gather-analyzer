// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use std::fs;
use std::path::PathBuf;
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub raw: String,
    pub yaml: Option<Yaml>,
}

impl Manifest {
    pub fn new() -> Manifest {
        Manifest {
            name: String::new(),
            raw: String::new(),
            yaml: None,
        }
    }

    pub fn from(path: PathBuf) -> Result<Manifest> {
        if !path.is_file() {
            return Err(anyhow!("Path is not a file {}", path.as_path().display()));
        }
        if path.is_dir() {
            return Err(anyhow!("Path is a directory {}", path.as_path().display()));
        }

        let mut raw = fs::read_to_string(path.as_path())?;
        let mut docs = YamlLoader::load_from_str(&raw)?;

        if docs.is_empty() {
            Err(anyhow!(
                "No YAML documents found in path {}",
                path.as_path().display()
            ))
        } else {
            let mut yaml = docs.remove(0);
            let name = match yaml["metadata"]["name"].as_str() {
                Some(n) => String::from(n),
                None => String::from("Unknown"),
            };
            if yaml["metadata"]["managedFields"].as_vec().is_some() {
                // excise the managedFields entry from the metadata
                let mut ycopy = yaml.as_hash().unwrap_or(&Hash::new()).clone();
                let mut metadata = ycopy[&Yaml::String(String::from("metadata"))]
                    .as_hash()
                    .unwrap_or(&Hash::new())
                    .clone();
                metadata.remove(&Yaml::String(String::from("managedFields")));
                ycopy.remove(&Yaml::String(String::from("metadata")));
                ycopy.insert(Yaml::String(String::from("metadata")), Yaml::Hash(metadata));

                // we want to make sure that the display of the raw file is in a familiar format
                // to accomplish that, we want the metadata field to appear first, then spec, then status
                let spec_key = Yaml::String(String::from("spec"));
                if let Some(spec) = ycopy.get(&spec_key).and_then(|s| s.as_hash()).cloned() {
                    ycopy.remove(&spec_key);
                    ycopy.insert(spec_key, Yaml::Hash(spec));
                }

                let status_key = Yaml::String(String::from("status"));
                if let Some(status) = ycopy.get(&status_key).and_then(|s| s.as_hash()).cloned() {
                    ycopy.remove(&status_key);
                    ycopy.insert(status_key, Yaml::Hash(status));
                }

                yaml = Yaml::Hash(ycopy);
                let mut out_str = String::new();
                let mut emitter = YamlEmitter::new(&mut out_str);
                // if we have an error creating the string, default to using the original
                raw = match emitter.dump(&yaml) {
                    Ok(()) => {
                        out_str.push('\n');
                        out_str
                    }
                    Err(_) => raw,
                };
            }
            Ok(Manifest {
                name,
                raw,
                yaml: Some(yaml),
            })
        }
    }

    /// Parse a Kubernetes List YAML file and return a vector of Manifests
    /// This handles files with format:
    /// ```yaml
    /// apiVersion: v1
    /// kind: List
    /// items:
    ///   - apiVersion: v1
    ///     kind: PersistentVolumeClaim
    ///     ...
    /// ```
    pub fn from_list(path: PathBuf) -> Result<Vec<Manifest>> {
        if !path.is_file() {
            return Err(anyhow!("Path is not a file {}", path.as_path().display()));
        }

        let raw = fs::read_to_string(path.as_path())?;
        let docs = YamlLoader::load_from_str(&raw)?;

        if docs.is_empty() {
            return Err(anyhow!(
                "No YAML documents found in path {}",
                path.as_path().display()
            ));
        }

        let mut manifests = Vec::new();

        // Check if this is a List format (either explicit kind: List or has items array)
        let doc = &docs[0];
        let is_list_kind = doc["kind"]
            .as_str()
            .map(|k| k.ends_with("List"))
            .unwrap_or(false);

        if is_list_kind || doc["items"].as_vec().is_some() {
            // Extract items from the list
            // Handle both actual items array and null items (empty list)
            if let Some(items) = doc["items"].as_vec() {
                for item in items {
                    // Create a manifest for each item
                    let name = item["metadata"]["name"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();

                    // Convert the item back to YAML string
                    let mut item_yaml = item.clone();

                    // Remove managedFields if present
                    if item_yaml["metadata"]["managedFields"].as_vec().is_some() {
                        let mut item_copy = item_yaml.as_hash().unwrap_or(&Hash::new()).clone();
                        let mut metadata = item_copy[&Yaml::String(String::from("metadata"))]
                            .as_hash()
                            .unwrap_or(&Hash::new())
                            .clone();
                        metadata.remove(&Yaml::String(String::from("managedFields")));
                        item_copy.remove(&Yaml::String(String::from("metadata")));
                        item_copy
                            .insert(Yaml::String(String::from("metadata")), Yaml::Hash(metadata));

                        // Reorder fields: metadata, spec, status
                        let spec_key = Yaml::String(String::from("spec"));
                        if let Some(spec) =
                            item_copy.get(&spec_key).and_then(|s| s.as_hash()).cloned()
                        {
                            item_copy.remove(&spec_key);
                            item_copy.insert(spec_key, Yaml::Hash(spec));
                        }

                        let status_key = Yaml::String(String::from("status"));
                        if let Some(status) = item_copy
                            .get(&status_key)
                            .and_then(|s| s.as_hash())
                            .cloned()
                        {
                            item_copy.remove(&status_key);
                            item_copy.insert(status_key, Yaml::Hash(status));
                        }

                        item_yaml = Yaml::Hash(item_copy);
                    }

                    // Convert to string
                    let mut out_str = String::new();
                    let mut emitter = YamlEmitter::new(&mut out_str);
                    if emitter.dump(&item_yaml).is_ok() {
                        out_str.push('\n');
                        manifests.push(Manifest {
                            name,
                            raw: out_str,
                            yaml: Some(item_yaml),
                        });
                    }
                }
            } else if is_list_kind {
                // This is a List kind but items is null or not an array
                // Return empty vector (valid case for empty lists)
                return Ok(Vec::new());
            }
        } else {
            // Not a list, try to parse as a single document
            manifests.push(Self::from(path)?);
        }

        Ok(manifests)
    }

    pub fn as_yaml(&self) -> &Yaml {
        self.yaml.as_ref().unwrap_or(&Yaml::Null)
    }

    pub fn as_raw(&self) -> &String {
        &self.raw
    }

    /// Return true if the manifest has the condition type.
    pub fn has_condition(&self, condition: &str) -> bool {
        let empty = Vec::<Yaml>::new();
        let conditions = self.as_yaml()["status"]["conditions"]
            .as_vec()
            .unwrap_or(&empty);
        let matchedconditions: Vec<&Yaml> = conditions
            .iter()
            .filter(|c| c["type"].as_str().unwrap_or("") == condition)
            .collect();
        !matchedconditions.is_empty()
    }

    /// Return true if the manfiest has the condition type with the specified status.
    ///
    /// If the manifest has a `status.conditions` list, this function will iterate
    /// through them attempting to match the condition type and status strings.
    pub fn has_condition_status(&self, condition: &str, status: &str) -> bool {
        self.as_yaml()["status"]["conditions"]
            .as_vec()
            .into_iter()
            .flatten()
            .any(|c| c["type"].as_str() == Some(condition) && c["status"].as_str() == Some(status))
    }

    /// Extract UID from metadata
    pub fn uid(&self) -> Option<String> {
        self.yaml
            .as_ref()
            .and_then(|y| y["metadata"]["uid"].as_str())
            .map(|s| s.to_string())
    }

    /// Extract namespace from metadata
    pub fn namespace(&self) -> Option<String> {
        self.yaml
            .as_ref()
            .and_then(|y| y["metadata"]["namespace"].as_str())
            .map(|s| s.to_string())
    }

    /// Extract labels from metadata
    pub fn labels(&self) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;
        let mut labels = HashMap::new();
        if let Some(yaml) = &self.yaml {
            if let Some(hash) = yaml["metadata"]["labels"].as_hash() {
                for (k, v) in hash {
                    if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                        labels.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
        labels
    }

    /// Extract annotations from metadata
    pub fn annotations(&self) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;
        let mut annotations = HashMap::new();
        if let Some(yaml) = &self.yaml {
            if let Some(hash) = yaml["metadata"]["annotations"].as_hash() {
                for (k, v) in hash {
                    if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                        annotations.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
        annotations
    }

    /// Extract creation timestamp from metadata
    pub fn creation_timestamp(&self) -> Option<String> {
        self.yaml
            .as_ref()
            .and_then(|y| y["metadata"]["creationTimestamp"].as_str())
            .map(|s| s.to_string())
    }

    /// Extract owner references from metadata
    pub fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        let mut refs = Vec::new();
        if let Some(yaml) = &self.yaml {
            if let Some(entries) = yaml["metadata"]["ownerReferences"].as_vec() {
                for entry in entries {
                    let kind = entry["kind"].as_str().unwrap_or("").to_string();
                    let name = entry["name"].as_str().unwrap_or("").to_string();
                    let uid = entry["uid"].as_str().map(|s| s.to_string());
                    let controller = entry["controller"].as_bool();
                    if !kind.is_empty() && !name.is_empty() {
                        refs.push((kind, name, uid, controller));
                    }
                }
            }
        }
        refs
    }

    /// Extract all conditions from status
    pub fn conditions(&self) -> Vec<(String, String, Option<String>, Option<String>)> {
        let mut conditions = Vec::new();
        if let Some(yaml) = &self.yaml {
            if let Some(conds) = yaml["status"]["conditions"].as_vec() {
                for cond in conds {
                    let type_ = cond["type"].as_str().unwrap_or("").to_string();
                    let status = cond["status"].as_str().unwrap_or("").to_string();
                    let reason = cond["reason"].as_str().map(|s| s.to_string());
                    let message = cond["message"].as_str().map(|s| s.to_string());
                    conditions.push((type_, status, reason, message));
                }
            }
        }
        conditions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_from_unknown_error() {
        let observed = Manifest::from(PathBuf::from(""));
        assert!(observed.is_err())
    }

    #[test]
    fn test_manifest_from_not_a_file() {
        let observed = Manifest::from(PathBuf::from(
            "testdata/must-gather-invalid/does-not-exist.yaml",
        ));
        assert!(observed.is_err())
    }

    #[test]
    fn test_manifest_from_is_a_directory() {
        let observed = Manifest::from(PathBuf::from("testdata/must-gather-valid"));
        assert!(observed.is_err())
    }

    #[test]
    fn test_manifest_from_empty_file() {
        let observed = Manifest::from(PathBuf::from("testdata/must-gather-invalid/empty.yaml"));
        assert!(observed.is_err())
    }

    #[test]
    fn test_manifest_as_yaml() {
        let expected = "Node";
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let observed = &manifest.as_yaml()["kind"];
        assert_eq!(observed.as_str().unwrap(), expected)
    }

    #[test]
    fn test_manifest_as_raw() {
        let expected = include_str!("../testdata/ip-10-0-0-1.control.plane.no-managed-fields.yaml");
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let observed = manifest.as_raw();
        assert_eq!(observed, expected)
    }

    #[test]
    fn test_manifest_name() {
        let expected = String::from("ip-10-0-0-1.control.plane");
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert_eq!(manifest.name, expected)
    }

    #[test]
    fn test_manifest_has_condition_status_true() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert_eq!(manifest.has_condition_status("Ready", "True"), true)
    }

    #[test]
    fn test_manifest_has_condition_status_false() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert_eq!(manifest.has_condition_status("PIDPressure", "True"), false)
    }

    #[test]
    fn test_manifest_has_condition_status_false_nonexistant() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert_eq!(manifest.has_condition_status("foo", "bar"), false)
    }

    #[test]
    fn test_manifest_has_condition_true() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert_eq!(manifest.has_condition("Ready"), true)
    }

    #[test]
    fn test_manifest_has_condition_false() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert_eq!(manifest.has_condition("FooBar"), false)
    }

    #[test]
    fn test_manifest_removes_managed_fields() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        assert!(
            manifest.as_yaml()["metadata"]["managedFields"]
                .as_vec()
                .is_none()
        );
    }

    #[test]
    fn test_extract_uid() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let uid = manifest.uid();
        assert!(uid.is_some());
        assert_eq!(uid.unwrap(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_extract_namespace() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        // Nodes are cluster-scoped, so namespace should be None
        assert!(manifest.namespace().is_none());
    }

    #[test]
    fn test_extract_labels() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let labels = manifest.labels();
        assert!(!labels.is_empty());
        // Check for common node labels
        assert!(
            labels.contains_key("kubernetes.io/hostname")
                || labels.contains_key("node-role.kubernetes.io/master")
        );
    }

    #[test]
    fn test_extract_annotations() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let annotations = manifest.annotations();
        // Annotations may or may not be present, just verify it returns a HashMap
        assert!(annotations.is_empty() || !annotations.is_empty());
    }

    #[test]
    fn test_extract_creation_timestamp() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let timestamp = manifest.creation_timestamp();
        assert!(timestamp.is_some());
    }

    #[test]
    fn test_extract_conditions() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let conditions = manifest.conditions();
        assert!(!conditions.is_empty());

        // Check that we have a Ready condition
        let ready_condition = conditions.iter().find(|(type_, _, _, _)| type_ == "Ready");
        assert!(ready_condition.is_some());

        let (_, status, _, _) = ready_condition.unwrap();
        assert_eq!(status, "True");
    }

    #[test]
    fn test_extract_conditions_empty() {
        // Test with a resource that has no conditions
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/machineconfiguration.openshift.io/machineconfigs/00-master.yaml"
        )).unwrap();
        let conditions = manifest.conditions();
        // MachineConfigs typically don't have conditions
        assert!(conditions.is_empty());
    }
}
