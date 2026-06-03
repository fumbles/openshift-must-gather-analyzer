# Development Workflows

## Local Development

Install dependencies once:

```bash
cd frontend
npm install
```

Build frontend:

```bash
cd frontend
npm run build
```

Run Rust tests:

```bash
cargo test
```

Run release build:

```bash
cargo build --release
```

Generate a report from the common local fixture:

```bash
rm -rf must-gather-analyze.20260527-020313
./analyze-mg must-gather-20260527-020313
```

Generate from the Palo Alto fixture, when available:

```bash
./analyze-mg must-gather.local.palo-alto.5945424480890452937 output-script-test
```

## Validation Matrix

Use this matrix based on the change type.

### Rust Parser or Analyzer Change

```bash
cargo test
git diff --check
./analyze-mg must-gather-20260527-020313
```

### Frontend UI Change

```bash
cd frontend
npm run build
cd ..
cargo test
rm -rf must-gather-analyze.20260527-020313
./analyze-mg must-gather-20260527-020313
```

Then inspect the generated site. If the user is checking in a browser, ask them to reopen the generated `index.html` after regeneration.

### Generated Site or Asset Embedding Change

```bash
cd frontend
npm run build
cd ..
cargo test
./analyze-mg must-gather-20260527-020313
sed -n '1,24p' must-gather-analyze.20260527-020313/index.html
```

Confirm asset links include cache-busting query strings.

### Release Script Change

```bash
./build-release-artifacts --skip-linux
```

For full release verification, Linux requires `podman`:

```bash
./build-release-artifacts
```

## Adding a New Resource Kind

1. Add a parser file in `src/resources/<kind>.rs`.
2. Export it in `src/resources/mod.rs`.
3. Add fields to `MustGather` if it is a first-class collection.
4. Load it in `MustGather::from_path` using `build_manifest_path` or custom traversal.
5. Add it to the relevant data structure in `src/html_v2.rs`.
6. Serialize it through `serialize_resources`.
7. Add it to `write_site_details` and `write_resource_collection_raw` if detail/raw loading is needed.
8. Add frontend navigation in `Sidebar.jsx` and routing in `App.jsx`.
9. Add analyzer support if there are meaningful health checks.
10. Add tests for parser behavior and path discovery when practical.

Use `GenericResource` when the UI only needs name, kind, namespace, labels, annotations, raw YAML, and simple health status.

## Adding an Analyzer

1. Create a new analyzer file in `src/analyzers/`.
2. Implement `HealthAnalyzer`.
3. Return supported kinds from `supported_kinds`.
4. Register the analyzer in `AnalyzerRegistry::new`.
5. Add focused tests in `src/analyzers/tests.rs` or a local test module.
6. Ensure frontend detail tabs show analysis, errors, and warnings as expected.

## Adding a Frontend Section

1. Add data to `src/html_v2.rs`.
2. Add sidebar entries in `frontend/src/components/Sidebar.jsx`.
3. Add route mapping in `frontend/src/App.jsx`.
4. Decide which UI shell to use:
   - Workloads-style shell for Kubernetes workload-like collections.
   - Storage shell for storage resource views.
   - `ResourceSplitView` for simple list/detail collections.
5. Add the route to `fullHeightSection` if the page has independent scroll panes.
6. Build frontend and regenerate a report.

## Output Directory Behavior

Default site mode writes beside the input must-gather:

```text
must-gather-20260527-020313
must-gather-analyze.20260527-020313/
```

This logic exists in both:

- `src/main.rs`
- `analyze-mg`

If changing output naming, update both and keep tests aligned.

## Version and Release Notes

Current package version comes from `Cargo.toml`.

`./build-release-artifacts --new-version` increments the patch segment. It does not increment the middle semver segment.

Release artifacts are written under:

```text
releases/<version>/
```

Do not delete or overwrite release artifacts unless the user asks or a release build command is explicitly being run.

## Dirty Worktree Guidance

This repo is often dirty during active work. Before editing:

```bash
git status --short
```

Treat unrelated changes as user-owned. Do not revert them. If a file you need to edit already has unrelated changes, read it carefully and patch around them.
