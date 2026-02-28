---
title: Helm
description: Browse Helm releases and chart repositories without the CLI.
---

Kore includes native Helm support, letting you inspect installed releases and chart repositories directly from the UI — no `helm` CLI required for read operations.

:::note
Helm must be installed on your system (`helm` in `$PATH`) for Kore to show the Helm section. If Helm is not detected, the sidebar items link to an installation guide.
:::

## Helm Releases

The **Releases** page lists every Helm release installed in the cluster (across all namespaces).

| Column | Description |
|--------|-------------|
| Name | Release name |
| Namespace | Namespace the release is deployed into |
| Chart | Chart name and version |
| App Version | Upstream application version |
| Status | `deployed`, `failed`, `pending-install`, `superseded` |
| Revision | Current revision number |
| Updated | Timestamp of the last upgrade |

### Release actions

Right-click a release row to:

- **View manifests** — see the rendered Kubernetes YAML for all resources in the release
- **View values** — inspect the `values.yaml` used for this release
- **Copy name** — copy the release name

## Helm Charts

The **Charts** page lists charts available in the locally configured Helm repositories.

| Column | Description |
|--------|-------------|
| Chart | Chart name |
| Repository | Source repository |
| Version | Latest chart version |
| App Version | Upstream application version |
| Description | Chart description |

### Adding repositories

Helm repository management is done via the `helm` CLI (`helm repo add`, `helm repo update`). Kore reflects the repositories already configured on your system — run `helm repo update` and re-open the Charts page to see the latest versions.

## Checking Helm availability

Kore detects Helm at startup by running `helm version`. The detected version is shown in the Helm section header. If `helm` is not in `$PATH`, the Releases and Charts pages display a prompt with installation instructions.
