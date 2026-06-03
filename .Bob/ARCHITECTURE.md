# Architecture

## Runtime Model

The project is a Rust CLI that emits a static web app. There is no backend server at runtime. The generated site loads static JavaScript, CSS, summary data, resource details, raw YAML, and logs from files in the output directory.

Default output:

```text
must-gather-analyze.<timestamp>/
  index.html
  assets/
    index.js
    index.css
  data/
    summary.js
  resources/
    <resource-detail-json-files>
  raw/
    <raw-yaml-files>
  logs/
    <pod-log-files>
```

Single-file mode embeds CSS, JavaScript, and all data into one HTML file. Site mode is the default because large must-gathers are too heavy for one HTML file.

## End-to-End Data Flow

1. `src/main.rs` parses CLI args.
2. `MustGather::from` in `src/mustgather.rs` resolves the real must-gather root.
3. `src/mustgather.rs` loads manifests, pods, logs, namespaces, cluster-scoped resources, storage, networking, security, administration, platform, and workload resources.
4. Resource structs implement traits from `src/resources/mod.rs`.
5. `src/analyzers/` produces health analysis for supported resource kinds.
6. `src/html_v2.rs` converts the `MustGather` into serializable `TriageMustGatherData`.
7. `generate_site` writes `index.html`, assets, summary data, detail JSON, raw YAML, and logs.
8. The React app reads `window.__MGA_DATA__` from `data/summary.js`, then lazy-loads detail files through hooks.

## Rust Layers

### CLI Layer

`src/main.rs` owns:

- CLI flags: `--site`, `--single-file`, `--v1`, `--tar`.
- Default output directory naming.
- Tar and tar.gz extraction.
- Choosing legacy v1 HTML, v2 single-file HTML, or v2 multi-file site.

### Parsing Layer

`src/mustgather.rs` owns:

- Finding the usable must-gather root.
- Loading specific resource collections from expected must-gather paths.
- Loading pod current logs.
- Namespace discovery.
- Platform detection from namespace names.
- Loading platform-specific resources, currently OpenShift Virtualization list files from KubeVirt, HCO, CDI, snapshot, clone, export, pool, and instance type API groups.

### Manifest Layer

`src/manifest.rs` owns:

- Reading YAML files.
- Parsing YAML into `yaml-rust2` data.
- Removing noisy metadata where needed.
- Extracting common metadata, conditions, labels, annotations, namespace, UID.

### Resource Layer

`src/resources/mod.rs` defines shared types and traits:

- `Resource`: original trait used for parsing and raw access.
- `ResourceV2`: richer UI-facing trait with metadata, status, relationships, key fields, logs, and summaries.
- `HealthStatus`, `Condition`, `ResourceMetadata`, `ResourceLink`.

Each file under `src/resources/` models one Kubernetes/OpenShift resource kind. Use `GenericResource` for broad resources where custom parsing is not yet worth it, such as Administration entries.

### Analyzer Layer

`src/analyzers/` defines:

- `HealthAnalyzer` trait.
- `AnalyzerRegistry`.
- Analyzer implementations for nodes, pods, operators, machines, workloads, storage, and networking.
- `HealthAnalysis`, `Issue`, `Recommendation`, severity, and category types.

If no analyzer supports a resource kind, the default analysis is healthy with summary `No analysis performed`.

### Site Generation Layer

`src/html_v2.rs` owns:

- Data structures serialized to the frontend.
- Summary versus full data generation.
- Writing the multi-file site.
- Writing raw YAML and logs.
- Embedding frontend bundles with `include_str!`.
- Generating cache-busted asset links in site mode.

`src/html.rs` is the older v1 UI and should not be the default target for new UI work.

## Frontend Layers

### Entry

- `frontend/src/main.jsx`: bootstraps React and normalizes loaded data.
- `frontend/src/App.jsx`: top-level routes, shared resource split view, dashboard, events, compute/security/admin pages.
- `PlatformView`: renders `Platform -> Virtualization` from serialized virtualization collections.

### Major Components

- `Header`: top bar, theme toggle, report bug link, global search.
- `Hero`: cluster summary and top stats.
- `Sidebar`: left navigation and child section routing.
- `Workloads`: Workloads, Networking, Platform-style resource panes.
- `Storage`: Storage resource panes.
- `ResourceSplitView`: generic split view used by Cluster Health, Nodes, Namespaces, Compute, Security, and Administration.
- `ResourceDetailsPanel`: YAML, related resources, logs, analysis, errors, warnings, metadata.
- `Tabs`, `YAMLViewer`, `ContainerLogs`, `HealthAnalysis`: detail panel internals.

### Data Loading

- Summary data comes from `window.__MGA_DATA__` in `data/summary.js`.
- Details are lazy-loaded by `frontend/src/hooks/useResourceDetail.js`.
- Raw YAML and logs are stored outside the summary for performance.

## Build Embedding

`build.rs` runs `npm run build` in `frontend/` before Rust compilation. Then `src/html_v2.rs` embeds:

```rust
const REACT_JS: &str = include_str!("../frontend/dist/assets/index.js");
const REACT_CSS: &str = include_str!("../frontend/dist/assets/index.css");
```

For Linux release builds in the musl container, the script sets `CAMGI_SKIP_FRONTEND_BUILD=1` because the frontend has already been built on the host.

## Release Architecture

`build-release-artifacts`:

1. Optionally increments or sets `Cargo.toml` version.
2. Removes old `releases/`.
3. Builds frontend once.
4. Builds macOS arm64 natively unless skipped.
5. Builds Linux x86_64 static musl binary via `podman` unless skipped.
6. Packages each platform with `mga`, `analyze-mg`, and `README.txt`.
7. Writes `SHA256SUMS`.
8. Optionally publishes with `gh release create`.
