# Frontend UI Guide

## Frontend Stack

- React 19.
- Vite.
- Tailwind CSS.
- Static data loading from generated files.

Build from `frontend/`:

```bash
cd frontend
npm run build
```

Do not run `npm run build` from repo root. There is no root `package.json`.

## Routing

Routing is hash-based and implemented in `frontend/src/hooks/useHashRouter.js`.

Examples:

```text
#dashboard
#workloads-pods
#networking-services
#administration-cluster-settings
#security-clusterrolebindings
```

Sidebar child IDs must match `App.jsx` route cases exactly.

## Main UI Shell

`App.jsx` renders:

```text
Header
Hero
Sidebar + main content
KeyboardShortcutsHelp
```

Routes that need internal scroll panes must be included in `fullHeightSection`. This makes `<main>` use `overflow-hidden` and lets the page component own scrolling.

Current full-height families include:

- Cluster Health
- Nodes
- Namespaces
- Compute children
- Workloads and Workload children
- Events
- Security children
- Administration children
- Networking and Networking children
- Storage and Storage children
- Fusion children

## Resource Page Families

### Workloads

File: `frontend/src/components/Workloads.jsx`

Used for:

- Workloads
- Networking
- IBM Spectrum Fusion workload views
- Cluster Operators

This component has mature independent scrolling and chrome collapse behavior. When diagnosing layout problems, compare new split-pane pages against this file.

### Storage

File: `frontend/src/components/Storage.jsx`

Storage has a custom shell because its resource categories and details differ.

### Generic ResourceSplitView

File: `frontend/src/App.jsx`

Used for:

- Cluster Health
- Nodes
- Namespaces
- Compute child sections
- Security child sections
- Administration child sections
- Platform -> Virtualization

Important layout rules:

- Root must be `h-full min-h-0 overflow-hidden`.
- Grid must have a bounded height and `min-h-0`.
- Left and right panes must be `pane-scrollbar` scroll containers.
- Security and Administration are sensitive to this because their lists can be long and their detail YAML can be very tall.

Current generic split view uses a hard desktop pane bound:

```text
xl:h-[calc(100vh-17rem)]
xl:max-h-[calc(100vh-17rem)]
```

This is intentionally explicit. Do not remove it unless replacing with a verified equivalent.

## Cache Busting

Generated site mode links frontend assets with query strings:

```html
<link rel="stylesheet" href="assets/index.css?v=<bundle-size-token>">
<script src="assets/index.js?v=<bundle-size-token>"></script>
```

This exists because browser tabs were showing stale `assets/index.js` after regenerating reports. If users report a UI fix does not appear, first confirm the generated `index.html` has versioned asset URLs and ask them to reopen the file.

## Light Theme

Light mode is implemented by toggling `theme-light` on `document.documentElement`.

Most light-mode overrides live in `frontend/src/index.css` and target Tailwind class names. Be careful: these selectors are broad and can affect many components.

## Scrollbar and Scroll Containment

`pane-scrollbar` is defined in `frontend/src/index.css`.

It sets:

- stable scrollbar gutter
- thin scrollbar styling
- `overscroll-behavior: contain`

The containment is important so trackpad or wheel scroll does not bubble into the page when an internal pane hits an edge.

## Detail Tabs

Resource detail tabs are in `ResourceDetailsPanel` and supporting components:

- YAML
- Related
- Logs, when present
- Analysis
- Errors
- Warnings
- Metadata

Default tab behavior should generally favor YAML for resource detail pages, because users inspect YAML and logs heavily.

## Platform Virtualization

`Platform -> Virtualization` is rendered in `PlatformView` in `frontend/src/App.jsx`.

It flattens every collection under `data.platform.virtualization` into one `ResourceSplitView`. The backend writes the underlying collections in `src/html_v2.rs`, and detail/raw YAML files are lazy-loaded like other generic resources.

If adding another platform-specific resource view, prefer this pattern unless the section needs custom topology or storage-style behavior.

## Resource Cards

Generic resource cards are intentionally compact:

- Name and status inline.
- Kind hidden for `CustomResourceDefinition` cards where it adds noise.
- Error/warning details only expand the card when present.

This supports high-density left columns. Avoid large summary cards for CRDs and generic admin resources.

## Report Bug Link

The header includes a report bug link:

```text
https://github.com/fumbles/openshift-must-gather-analyzer/issues/new/choose
```

Keep it next to the Light/Dark button.

## Frontend Verification Tips

After UI changes:

```bash
cd frontend
npm run build
cd ..
./analyze-mg must-gather-20260527-020313
```

Then verify:

- Workloads child navigation opens the correct child view.
- Networking child navigation opens the correct child view.
- Platform -> Virtualization shows HyperConverged/KubeVirt resources for the Palo Alto fixture when available.
- Security and Administration left panes scroll independently.
- YAML tab is default when selecting resources.
- Light and dark mode both show selected resources clearly.
- Generated `index.html` references versioned assets.
