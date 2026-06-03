# Troubleshooting and Known Pitfalls

## Frontend Build Fails from Repo Root

Symptom:

```text
npm error enoent Could not read package.json
```

Cause: `package.json` is under `frontend/`.

Fix:

```bash
cd frontend
npm run build
```

## Browser Still Shows Old UI After Regenerating

Generated site mode now uses cache-busting asset query strings, but an already-open tab can still hold old runtime state.

Ask the user to reopen the regenerated `index.html`, or hard refresh.

Check generated HTML:

```bash
sed -n '1,24p' must-gather-analyze.20260527-020313/index.html
```

Expected:

```html
<link rel="stylesheet" href="assets/index.css?v=...">
<script src="assets/index.js?v=..."></script>
```

## Security or Administration Left Column Does Not Scroll

Those sections use `ResourceSplitView` in `frontend/src/App.jsx`, not the Workloads component.

Check:

- Route ID starts with `security-` or `administration-`.
- Route is included in `fullHeightSection`.
- `ResourceSplitView` root has `h-full min-h-0 overflow-hidden`.
- Its grid has `min-h-0`, `overflow-hidden`, and a hard desktop height bound.
- Left pane has `pane-scrollbar`, `h-full`, `min-h-0`, and `overflow-y-scroll`.
- `frontend/src/index.css` has `overscroll-behavior: contain` for `.pane-scrollbar`.

After changes:

```bash
cd frontend && npm run build
cd ..
./analyze-mg must-gather-20260527-020313
```

Then reopen the generated site.

## Must-Gather Root Panic or Not Found

Past panic:

```text
called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound }
```

Cause: root discovery used `canonicalize().unwrap()` and assumed wrapper directories had only one child. Some customer archives contain empty image directories and valid sibling roots.

Fix exists in `find_must_gather_root`. Do not replace it with a single-child recursion.

## Weird Flattened Must-Gather

If a directory contains hundreds of unrelated names at one level, it may be flattened or corrupt.

Check for a complete nested root:

```bash
find . -type f -name version -print
find . -type d -name namespaces -print
find . -type d -name cluster-scoped-resources -print
```

If no single directory contains all required root markers, the analyzer may produce a misleading partial report. Ask for a fresh must-gather.

## Cargo Build Rebuilds Frontend

`build.rs` runs `npm run build` automatically. This can make Rust builds slower.

Linux release container builds set:

```bash
CAMGI_SKIP_FRONTEND_BUILD=1
```

Only use that when `frontend/dist/assets/index.js` and `index.css` already exist.

## Cargo Fmt Can Touch Unrelated Files

Some files may have formatting drift. `cargo fmt` can reformat unrelated modules. If the task is narrow, avoid carrying unrelated formatter-only diffs unless the user wants cleanup.

Use:

```bash
git diff --check
```

to catch whitespace errors without reformatting everything.

## Release Artifact Changes Are Large

Release builds write binary tarballs and platform directories under `releases/<version>/`.

Do not delete or revert release files unless the user asked to rebuild or clean releases.

The current script removes the whole `releases/` directory before building a new version.

## Output Site Folder Location

The default site output goes beside the must-gather directory.

If the user reports the report folder appears in the wrong place, check both:

- `default_site_output_dir` in `src/main.rs`
- `default_output_dir` in `analyze-mg`

They should stay behaviorally aligned.

## Platform Label Says None

The hero should display `Platform: <value>` when a platform value is available. `Platform: None` is clearer than an unlabeled `None`.

This value comes from `infrastructures.config.openshift.io/cluster.yaml`:

```text
status.platformStatus.type
```

If the file is absent or unparsable, the Rust side returns `Unknown`.

## Multiple Containers in Pods

Pods can include multiple containers and current logs. The UI should let users select containers in the Logs tab. Do not assume one pod equals one log.

## Generated Output Can Be Large

A normal generated report can be tens of megabytes and thousands of files. Example local report:

```text
81M  must-gather-analyze.20260527-020313
```

This is expected.
