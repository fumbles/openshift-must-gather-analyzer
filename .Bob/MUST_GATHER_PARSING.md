# Must-Gather Parsing Notes

## Expected OpenShift Must-Gather Shape

A normal root usually contains:

```text
version
namespaces/
cluster-scoped-resources/
nodes/
static-pods/
monitoring/
network_logs/
host_service_logs/
etcd_info/
```

The analyzer relies heavily on:

```text
<root>/namespaces/<namespace>/<api-group>/<resource-kind>/*.yaml
<root>/cluster-scoped-resources/<api-group>/<resource-kind>/*.yaml
```

Core resources use `core` as the API group in generated paths.

## Root Discovery

Root discovery is in `find_must_gather_root` in `src/mustgather.rs`.

It must handle:

- A direct must-gather root.
- Wrapper directories with one nested image directory.
- Weird unpacked archives containing multiple image-looking directories.
- Empty sibling image directories next to the real root.
- Partially flattened wrapper directories.

The current root scoring prefers a complete root with:

```text
version
namespaces/
cluster-scoped-resources/
```

over a weaker partial root with only `namespaces/` and `cluster-scoped-resources/`.

Do not reintroduce `canonicalize().unwrap()` in root discovery. Some user paths have caused this to panic.

## Flattened or Corrupt Must-Gathers

Some customer unpacked directories are damaged, with hundreds of normally nested directories all appearing as siblings. Example symptoms:

- Namespace names, pod names, API groups, and resource kinds all at one level.
- `namespaces` and `cluster-scoped-resources` exist, but many expected child paths are missing or mixed.
- Empty `quay-io-...` image directory plus a longer valid sibling directory.

If a complete nested root exists, analyze that. If only the flattened directory exists, the report may be incomplete or misleading and the user should request a fresh must-gather.

Useful commands for triage:

```bash
find . -type f -name version -print
find . -type d -name namespaces -print
find . -type d -name cluster-scoped-resources -print
```

A usable candidate should have all three under the same directory.

## Resource Loading

Important functions in `src/mustgather.rs`:

- `build_manifest_path`: builds namespace-scoped and cluster-scoped resource paths.
- `get_resources<T>`: reads YAML files from a resource directory.
- `get_pods`: reads pod manifests and container current logs.
- `get_namespaces`: supports current and legacy namespace manifest locations.
- `get_cluster_settings`: gathers config.openshift.io resources except cluster operators.
- `get_cluster_resources`: gathers broad cluster-scoped resources, including OLM Operators from `cluster-scoped-resources/operators.coreos.com/operators`.
- `get_namespaced_core_resources`: gathers administration resources such as ResourceQuotas and LimitRanges.

`get_resources<T>` must ignore non-YAML files. Some directories contain README-like files or other artifacts.

## Pod Logs and Containers

Pods can have multiple containers. The parser scans container directories and reads:

```text
<pod-dir>/<container>/<container>/logs/current.log
```

Logs are truncated to `MAX_LOG_LINES` from the end for summary output. The full log path is preserved when available.

The frontend supports selecting among multiple containers in the Logs tab.

## Administration Resources

Administration data includes:

- Cluster Settings
- Namespaces
- ResourceQuotas
- LimitRanges
- CustomResourceDefinitions
- Dynamic Plugins

Most are loaded as `GenericResource`. This is deliberate: the UI primarily needs YAML, status, key metadata, and compact cards.

CRDs should stay compact in the left column. Do not make CRD cards large unless they have errors or warnings.

CRDs are enriched with related custom resource instances when the must-gather includes matching resource directories. The linker uses `spec.group`, `spec.names.plural`, `spec.names.kind`, and `spec.scope` from the CRD, then scans:

- `cluster-scoped-resources/<group>/<plural>/*.yaml` for cluster-scoped instances.
- `namespaces/<namespace>/<group>/<plural>/*.yaml` for namespaced instances.

Those instances appear in the CRD Related tab as references. CRDs always expose the Related tab in the UI; if a must-gather only includes the CRD definitions and not the custom resource instances, the tab shows an empty state with the exact paths the analyzer checked.

## Operators vs ClusterOperators

These are different resource families:

- `ClusterOperator` resources come from `config.openshift.io/clusteroperators` and drive Home -> Cluster Health and Home -> Cluster Operators.
- OLM `Operator` resources come from `operators.coreos.com/operators` and drive Home -> Operators.

Keep these separate in naming and navigation. ClusterOperators answer control-plane health questions; Operators show installed OLM operator resources similar to `oc get operator`.

## Security Resources

Security data includes:

- Cluster Roles
- Cluster Role Bindings
- Security Context Constraints

These use the generic split view in the frontend and can produce large left lists. Treat independent left-pane scrolling as a regression-sensitive behavior.

## Platform Detection

`PlatformInfo::detect` uses namespace names to infer:

- IBM Spectrum Fusion
- ODF
- Service Mesh
- ACM
- OpenShift Virtualization
- Cloud Pak for Data

This is heuristic. Do not treat it as authoritative cluster inventory.

## OpenShift Virtualization Artifacts

OpenShift Virtualization support is more than namespace detection. `get_virtualization_resources` in `src/mustgather.rs` scans every namespace for list-style files and expands `items[]` with `Manifest::from_list`.

Important paths include:

```text
namespaces/<namespace>/hco.kubevirt.io/hyperconvergeds.yaml
namespaces/<namespace>/kubevirt.io/kubevirts.yaml
namespaces/<namespace>/kubevirt.io/virtualmachines.yaml
namespaces/<namespace>/kubevirt.io/virtualmachineinstances.yaml
namespaces/<namespace>/pool.kubevirt.io/virtualmachinepools.yaml
namespaces/<namespace>/export.kubevirt.io/virtualmachineexports.yaml
namespaces/<namespace>/clone.kubevirt.io/virtualmachineclones.yaml
namespaces/<namespace>/snapshot.kubevirt.io/virtualmachinesnapshots.yaml
namespaces/<namespace>/snapshot.kubevirt.io/virtualmachinesnapshotcontents.yaml
namespaces/<namespace>/snapshot.kubevirt.io/virtualmachinerestores.yaml
namespaces/<namespace>/cdi.kubevirt.io/datavolumes.yaml
namespaces/<namespace>/cdi.kubevirt.io/datasources.yaml
namespaces/<namespace>/cdi.kubevirt.io/dataimportcrons.yaml
namespaces/<namespace>/instancetype.kubevirt.io/virtualmachineinstancetypes.yaml
namespaces/<namespace>/instancetype.kubevirt.io/virtualmachinepreferences.yaml
```

These resources are stored as `GenericResource` because the UI primarily needs name, kind, namespace, status conditions, raw YAML, and compact cards. If the collection is non-empty, `platform_info.virtualization_detected` is forced true even if namespace heuristics missed it.

The Palo Alto fixture has at least:

```text
hyperconvergeds: 1
kubevirts: 1
```

Use it as a regression check for `Platform -> Virtualization`.

## Tests to Preserve

The root discovery regression tests cover:

- Empty image directory sibling is skipped.
- Complete child root wins over partial wrapper.
- Resource loading ignores files without extensions.

If you change root discovery or resource loading, update or expand these tests.
