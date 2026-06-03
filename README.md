# OpenShift Must-Gather Analyzer

A modern tool for examining [OKD/OpenShift must-gather][mustgather] records with advanced analysis capabilities, interactive UI, and automated health checks.

## Features

### 🎨 Modern React-Based UI (v2)
- **Dark theme** inspired by yamlwrangler with excellent readability
- **Interactive navigation** with search, filters, and keyboard shortcuts
- **Resource cards** with health status indicators and quick actions
- **Tabbed interface** for YAML, logs, and health analysis
- **Deep linking** - share URLs to specific resources
- **Responsive design** that works on all screen sizes

### 🔍 Advanced Analysis
- **Automated health checks** for Nodes, Pods, ClusterOperators, and Machines
- **Health scoring** (0-100) with color-coded severity levels
- **Issue detection** with categorization (Availability, Performance, Configuration, etc.)
- **Actionable recommendations** for resolving detected issues
- **Condition analysis** for all resource types

### 📊 Resource Support
- Nodes (with readiness and pressure conditions)
- Pods (with container status and logs)
- ClusterOperators (with degraded/progressing/available status)
- Machines (with provisioning state)
- MachineSets (with replica counts)
- MachineConfigPools (with update status)
- ClusterAutoscaler & MachineAutoscaler
- CertificateSigningRequests
- OpenShift Virtualization artifacts under Platform → Virtualization, including HyperConverged, KubeVirt, VM, VMI, snapshot, CDI, instance type, and preference resources when present
- And more...

## Quickstart

### Installation

```bash
cargo install mga
```

### Basic Usage

1. Have a must-gather ready at `$MUST_GATHER_PATH`
2. Generate the HTML site:
   ```bash
   mga $MUST_GATHER_PATH
   ```
3. Open the generated `index.html` in your web browser

If you are working from this repository, you can use the helper script:

```bash
./analyze-mg $MUST_GATHER_PATH
```

This writes a static site beside the must-gather directory at `must-gather-analyze.<timestamp>/` by default when the must-gather name contains a recognizable timestamp, or you can pass a second argument to choose the output directory yourself.

You can also call the binary directly:

```bash
mga $MUST_GATHER_PATH
mga $MUST_GATHER_PATH report-site
```

By default, `mga` writes a multi-file static site for better performance on large must-gathers.
If you need the older single-file export, use `--single-file`:

```bash
mga --single-file $MUST_GATHER_PATH output.html
```

If you want a distributable bundle for a machine without Rust installed:

```bash
./build-dist
```

That produces a platform-specific tarball in `dist/` containing:
- `mga` - standalone executable
- `analyze-mg` - helper script that uses the bundled executable
- `README.txt` - quick usage notes

On the target system, extract the tarball and run:

```bash
./analyze-mg $MUST_GATHER_PATH
```

**Note:** The helper script redirects stderr from the binary/cargo fallback so shell noise does not pollute generated output.

### Command Line Options

```bash
mga [OPTIONS] <PATH> [OUTPUT]

OPTIONS:
    --single-file  Write a single self-contained HTML file
    --v1           Use the legacy HTML UI
    --tar          Open a must-gather archive in tar format
    -h, --help     Print help information
    -V, --version  Print version information

ARGS:
    <PATH>         The path to the must-gather directory or tar file
    [OUTPUT]       Optional output directory for site mode; defaults to
                   must-gather-analyze.<timestamp> beside the input when its
                   name matches a known must-gather pattern
                   With --single-file or --v1, this is an output HTML path and defaults to output.html
```

## Using the UI

### Keyboard Shortcuts

- **`/`** - Focus search box
- **`Escape`** - Clear search / close modals
- **`↑` / `↓`** - Navigate between resources
- **`Enter`** - Open selected resource

### Navigation

1. **Search** - Type in the search box to filter resources by name, kind, or namespace
2. **Filters** - Use the filter controls to show/hide specific resource types or health statuses
3. **Resource Cards** - Click any resource card to view details
4. **Tabs** - Switch between YAML, Logs, and Health Analysis views
5. **Health Analysis** - View automated health checks, issues, and recommendations

### Health Analysis

Each resource is automatically analyzed for common issues:

- **Health Score** (0-100): Overall health indicator
- **Issues**: Detected problems with severity levels (Info, Warning, Error, Critical)
- **Categories**: Issues grouped by type (Availability, Performance, Configuration, etc.)
- **Recommendations**: Actionable steps to resolve issues
- **Conditions**: Detailed status conditions from the resource

#### Severity Levels

- 🔴 **Critical** - Immediate action required
- 🟠 **Error** - Significant problem affecting functionality
- 🟡 **Warning** - Potential issue that should be investigated
- 🔵 **Info** - Informational message or minor concern

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/fumbles/openshift-must-gather-analyzer
cd openshift-must-gather-analyzer

# Build the frontend
cd frontend
npm install
npm run build
cd ..

# Build the Rust binary
cargo build --release

# Or build a distributable bundle
./build-dist

# Or build release artifacts for macOS arm64 + Linux x86_64
./build-release-artifacts

# Run tests
cargo test
```

### Project Structure

```
openshift-must-gather-analyzer/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── mustgather.rs        # Must-gather parsing
│   ├── manifest.rs          # YAML manifest handling
│   ├── html_v2.rs           # HTML generation with React
│   ├── resources/           # Resource type implementations
│   │   ├── node.rs
│   │   ├── pod.rs
│   │   ├── clusteroperator.rs
│   │   └── ...
│   └── analyzers/           # Health analysis system
│       ├── mod.rs           # Analyzer registry
│       ├── types.rs         # Issue, Recommendation, HealthAnalysis
│       ├── node_analyzer.rs
│       ├── pod_analyzer.rs
│       └── ...
├── frontend/                # React frontend
│   ├── src/
│   │   ├── App.jsx         # Main application
│   │   ├── components/     # React components
│   │   └── hooks/          # Custom React hooks
│   └── package.json
└── templates/              # HTML templates
```

### Adding New Analyzers

To add health analysis for a new resource type:

1. Create a new analyzer in `src/analyzers/`:
   ```rust
   use super::*;
   use crate::resources::{YourResource, ResourceV2};

   pub struct YourResourceAnalyzer;

   impl YourResourceAnalyzer {
       pub fn new() -> Self {
           Self
       }
   }

   impl HealthAnalyzer for YourResourceAnalyzer {
       fn can_analyze(&self, resource: &dyn ResourceV2) -> bool {
           resource.kind() == "YourKind"
       }

       fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
           // Implement your analysis logic
       }
   }
   ```

2. Register it in `src/analyzers/mod.rs`:
   ```rust
   pub fn new() -> Self {
       let mut registry = Self {
           analyzers: Vec::new(),
       };
       registry.analyzers.push(Box::new(your_analyzer::YourResourceAnalyzer::new()));
       registry
   }
   ```

## History

This started as a modernized conversion of the [okd-camgi][okdcamgi] tool from Python into Rust.
The conversion makes usage and distribution easier by providing a single compiled binary
with no runtime dependencies. The v2 UI adds modern web technologies (React, Tailwind CSS)
and automated health analysis capabilities.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

GPL-3.0-or-later - See [LICENSE](LICENSE) for details.

[okdcamgi]: https://github.com/elmiko/okd-camgi
[mustgather]: https://github.com/openshift/must-gather
