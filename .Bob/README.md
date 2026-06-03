# Bob Maintainer Guide

This folder is for AI maintainers working on this repository. Read this file first, then use the more focused files as needed.

## Project Summary

`must-gather-analyzer` builds a static analysis site for an unpacked OpenShift must-gather directory. The binary is named `mga`.

At a high level:

1. Rust parses the must-gather filesystem into strongly typed resource structs.
2. Rust runs analyzers and serializes summary/detail data.
3. A React frontend is built with Vite and embedded into the Rust binary.
4. `mga <must-gather-path>` writes a multi-file static site, usually beside the must-gather directory.

Primary user-facing helper:

```bash
./analyze-mg <must-gather-path> [output-dir]
```

Primary binary:

```bash
mga <must-gather-path> [output-dir]
mga --single-file <must-gather-path> output.html
```

## Important Files

- `src/main.rs`: CLI options, tar handling, default output directory logic.
- `src/mustgather.rs`: must-gather root discovery and resource loading.
- `src/manifest.rs`: YAML manifest reading and normalization.
- `src/resources/`: resource models and shared resource traits.
- `src/analyzers/`: health analysis implementations.
- `src/html_v2.rs`: React site data serialization and static site generation.
- `frontend/src/App.jsx`: main React routing and generic split-view pages.
- `frontend/src/components/Workloads.jsx`: Workloads, Networking, Platform-style resource UI.
- `frontend/src/components/Storage.jsx`: Storage-specific resource UI.
- `frontend/src/index.css`: Tailwind entry plus global theme and scrollbar rules.
- `build.rs`: compiles the frontend before Rust builds unless `CAMGI_SKIP_FRONTEND_BUILD=1`.
- `analyze-mg`: helper script for local and bundled execution.
- `build-dist`: single-platform distribution bundle.
- `build-release-artifacts`: macOS and Linux release bundles.

## Current Platform Resource Support

`Platform -> Virtualization` is a real resource view, not only a detection badge. The parser loads OpenShift Virtualization resources from namespaced list-style must-gather files such as:

```text
namespaces/<namespace>/hco.kubevirt.io/hyperconvergeds.yaml
namespaces/<namespace>/kubevirt.io/kubevirts.yaml
namespaces/<namespace>/kubevirt.io/virtualmachines.yaml
namespaces/<namespace>/kubevirt.io/virtualmachineinstances.yaml
namespaces/<namespace>/cdi.kubevirt.io/datavolumes.yaml
```

These are stored as `GenericResource` collections in `MustGather::virtualization`, serialized by `src/html_v2.rs`, and rendered through `PlatformView` in `frontend/src/App.jsx` using the shared `ResourceSplitView`.

## Must-Know Build Commands

Run frontend build from `frontend/`, not from repo root:

```bash
cd frontend
npm run build
```

Run Rust tests from repo root:

```bash
cargo test
```

Generate a local static site:

```bash
./analyze-mg must-gather-20260527-020313
```

Build release binary:

```bash
cargo build --release
```

Build release artifacts:

```bash
./build-release-artifacts --new-version
```

`--new-version` increments the patch segment, for example `0.0.306` to `0.0.307`.

## Verification Baseline

For most code changes, use:

```bash
cd frontend && npm run build
cargo test
git diff --check
```

For UI or generated-site changes, also regenerate a report:

```bash
rm -rf must-gather-analyze.20260527-020313
./analyze-mg must-gather-20260527-020313
```

Ask the user to reopen or hard-refresh the generated `index.html` when checking browser behavior. The generated HTML now includes cache-busting asset query strings, but old tabs can still hold old runtime state.

## AI Maintainer Rules

- Do not revert unrelated user changes. This repo often has release artifacts and version bumps in progress.
- Keep edits scoped. Avoid refactors unless the task requires them.
- Prefer existing patterns over new abstractions.
- Use `rg` for search.
- Use `apply_patch` for manual edits.
- For frontend changes, remember that the Rust binary embeds `frontend/dist/assets/index.js` and `index.css` at compile time.
- For generated site changes, check both frontend build and Rust generation path.
- Be cautious with `cargo fmt`: it may reformat unrelated Rust files. If formatting drift exists, avoid carrying unrelated churn unless the user asked for broad cleanup.

## More Detail

- Architecture and data flow: `.Bob/ARCHITECTURE.md`
- Development workflows: `.Bob/DEVELOPMENT_WORKFLOWS.md`
- Frontend layout rules: `.Bob/FRONTEND_UI_GUIDE.md`
- Must-gather parsing notes: `.Bob/MUST_GATHER_PARSING.md`
- Troubleshooting and known pitfalls: `.Bob/TROUBLESHOOTING.md`
