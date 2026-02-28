---
title: Storage
description: Manage PersistentVolumeClaims, PersistentVolumes, StorageClasses, ConfigMaps, and Secrets.
---

## ConfigMaps

ConfigMaps hold non-sensitive configuration data as key-value pairs or files. The list view shows name, namespace, and creation age. Click a row to view all keys and values in the detail panel, or right-click to edit the full YAML.

## Secrets

The Secrets page lists Secrets by type (`Opaque`, `kubernetes.io/tls`, `kubernetes.io/service-account-token`, etc.). Secret values are **not** displayed in the UI — only keys are shown. You can edit a Secret's YAML (values are base64-encoded in the raw YAML).

:::caution
Take care when editing Secrets. Kore applies changes immediately using `kubectl apply` semantics.
:::

## PersistentVolumeClaims (PVCs)

PVCs are namespace-scoped requests for storage. The list shows:

| Column | Description |
|--------|-------------|
| Name | PVC name |
| Namespace | Owning namespace |
| Status | `Bound`, `Pending`, `Lost` |
| Volume | Bound PersistentVolume name |
| Capacity | Requested/provisioned size |
| Access Modes | `RWO`, `ROX`, `RWX` |
| Storage Class | Provisioner class used |
| Age | Time since creation |

## PersistentVolumes (PVs)

PVs are cluster-scoped storage resources. The list view shows capacity, access modes, reclaim policy, and the bound claim (if any).

## Storage Classes

Storage Classes define available storage tiers. The list shows the provisioner, reclaim policy, volume binding mode, and whether the class is the cluster default.

The default Storage Class is highlighted — it is used when a PVC does not specify a `storageClassName`.
