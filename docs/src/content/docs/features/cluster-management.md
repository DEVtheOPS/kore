---
title: Cluster Management
description: Import, organise, and switch between multiple Kubernetes clusters.
---

Kore is designed for multi-cluster workflows. You can connect as many clusters as you like, organise them with custom names, icons, and tags, and switch between them instantly.

## Importing clusters

Click **+** in the icon sidebar to open the import dialog. Kore supports two discovery modes:

| Mode | How it works |
|------|-------------|
| **From file** | Browse to a single `kubeconfig` file |
| **From folder** | Scan a directory — Kore finds all valid configs inside |

After discovery, Kore lists every context found. Select the ones you want and click **Import**. The original `kubeconfig` files are never modified.

### Supported authentication methods

All standard `kubeconfig` credential types are supported:

- Client certificate + key
- Bearer token
- OIDC / exec-based auth (via `kubeconfig` `exec` entries)
- In-cluster service account (when running inside a pod)

---

## The Clusters overview

The **Clusters** page (`/`) is the home screen. It shows all your imported clusters in a searchable, sortable table with columns for name, context, last accessed, and tags.

### Actions per cluster

| Action | How |
|--------|-----|
| **Open** | Click the cluster row |
| **Settings** | Click the ⚙️ icon on the row |
| **Delete** | Click ✕ on the row (removes from Kore only — cluster is unaffected) |
| **Bookmark** | Click the bookmark icon to pin to the icon sidebar |

---

## Cluster settings

Open a cluster's settings page from the resource sidebar or the cluster row. Here you can:

- **Rename** the cluster (display name only — context name is unchanged)
- **Change the icon** — upload a custom PNG or SVG (stored locally, never uploaded)
- **Add tags** — arbitrary labels for filtering (`production`, `gke`, `eu-west`, …)
- **Add a description** — free-text note visible on the Clusters overview

---

## Bookmarks

Bookmarked clusters appear as icons in the icon sidebar for immediate access. The order is draggable — rearrange by holding and dragging.

---

## Namespace filtering

Every resource view respects the **namespace selector** at the top of the resource sidebar. Choose:

- **all** — show resources across all namespaces
- Any individual namespace — filters every list view simultaneously

Kore remembers your last-used namespace per cluster between sessions.

---

## Live kubeconfig sync

Kore watches your `kubeconfig` files for changes. If you add a new context with `kubectl config set-context` or any other tool, the Clusters list refreshes automatically — no restart needed.
