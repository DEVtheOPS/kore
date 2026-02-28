---
title: Networking
description: Inspect Services, Endpoints, Ingresses, and Network Policies.
---

## Services

The Services page lists all Services in the active namespace. The list shows:

| Column | Description |
|--------|-------------|
| Name | Service name |
| Namespace | Owning namespace |
| Type | `ClusterIP`, `NodePort`, `LoadBalancer`, `ExternalName` |
| Cluster IP | Internal cluster IP (or `None` for headless) |
| External IP | External IP or hostname (LoadBalancer only) |
| Ports | Port mappings (e.g., `80/TCP`, `443:30443/TCP`) |
| Age | Time since creation |

Click a Service row to see the full selector, session affinity settings, and related Endpoints.

## Endpoints

The Endpoints page lists the resolved IP:port pairs behind each Service. This is useful for debugging when a Service has no healthy endpoints (e.g., no pods match the selector).

## Ingresses

The Ingresses page shows all Ingress resources with their host rules, paths, TLS settings, and the Ingress class used. Click a row to see the full rule list with path types and backend service mappings.

## Network Policies

Network Policies control traffic flow between pods. The list shows the policy name, namespace, pod selector, and policy types (`Ingress`, `Egress`, or both).

Click a row to view the full ingress and egress rules — from/to selectors, namespace selectors, IP blocks, and ports.

:::note
Network Policies are only enforced if your cluster uses a CNI plugin that supports them (e.g., Calico, Cilium, WeaveNet). Kore shows the policies regardless of enforcement.
:::
