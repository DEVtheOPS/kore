---
title: Supported Resources
description: Full list of Kubernetes resource types supported by Kore.
sidebar:
  order: 1
---

Kore supports 35+ Kubernetes resource types across all major API groups. All resources support list, detail view, YAML view, YAML edit, and delete. Workload types additionally support scale and restart.

## Workloads

| Resource | API Group | Scope | Scale | Restart |
|----------|-----------|-------|-------|---------|
| Pod | `v1` | Namespaced | — | — |
| Deployment | `apps/v1` | Namespaced | ✓ | ✓ |
| StatefulSet | `apps/v1` | Namespaced | ✓ | ✓ |
| DaemonSet | `apps/v1` | Namespaced | — | ✓ |
| ReplicaSet | `apps/v1` | Namespaced | ✓ | — |
| Job | `batch/v1` | Namespaced | — | — |
| CronJob | `batch/v1` | Namespaced | — | — |

## Configuration

| Resource | API Group | Scope |
|----------|-----------|-------|
| ConfigMap | `v1` | Namespaced |
| Secret | `v1` | Namespaced |
| HorizontalPodAutoscaler | `autoscaling/v2` | Namespaced |
| ResourceQuota | `v1` | Namespaced |
| LimitRange | `v1` | Namespaced |
| PodDisruptionBudget | `policy/v1` | Namespaced |

## Networking

| Resource | API Group | Scope |
|----------|-----------|-------|
| Service | `v1` | Namespaced |
| Endpoints | `v1` | Namespaced |
| Ingress | `networking.k8s.io/v1` | Namespaced |
| NetworkPolicy | `networking.k8s.io/v1` | Namespaced |

## Storage

| Resource | API Group | Scope |
|----------|-----------|-------|
| PersistentVolumeClaim | `v1` | Namespaced |
| PersistentVolume | `v1` | Cluster |
| StorageClass | `storage.k8s.io/v1` | Cluster |

## Access Control

| Resource | API Group | Scope |
|----------|-----------|-------|
| ServiceAccount | `v1` | Namespaced |
| Role | `rbac.authorization.k8s.io/v1` | Namespaced |
| RoleBinding | `rbac.authorization.k8s.io/v1` | Namespaced |
| ClusterRole | `rbac.authorization.k8s.io/v1` | Cluster |
| ClusterRoleBinding | `rbac.authorization.k8s.io/v1` | Cluster |

## Cluster

| Resource | API Group | Scope |
|----------|-----------|-------|
| Node | `v1` | Cluster |
| Namespace | `v1` | Cluster |
| Event | `v1` | Namespaced |
| CustomResourceDefinition | `apiextensions.k8s.io/v1` | Cluster |

## Helm

| Resource | Notes |
|----------|-------|
| Helm Release | Requires `helm` CLI in `$PATH` |
| Helm Chart | Lists charts from configured repos |
