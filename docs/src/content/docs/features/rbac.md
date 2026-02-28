---
title: RBAC
description: Explore Roles, ClusterRoles, bindings, and service accounts.
---

Kore provides a full read/edit interface for Kubernetes RBAC resources.

## Roles and ClusterRoles

**Roles** are namespace-scoped permission sets. **ClusterRoles** apply cluster-wide (or as templates for RoleBindings across namespaces).

Both pages show the rule count and creation age. Click any row to see the full rule list — each entry shows the API groups, resources, and allowed verbs.

## RoleBindings and ClusterRoleBindings

Bindings connect subjects (users, groups, service accounts) to a Role or ClusterRole.

The list view shows:

| Column | Description |
|--------|-------------|
| Name | Binding name |
| Namespace | Namespace (RoleBindings only) |
| Role | The referenced Role or ClusterRole |
| Subjects | Comma-separated list of bound subjects |
| Age | Time since creation |

Click a binding row to see the full subject list with kind, name, and namespace for each entry.

## Service Accounts

Service accounts are the Kubernetes identity mechanism for workloads. The Service Accounts page lists all accounts across the selected namespace with their associated secret count.

## Editing RBAC resources

Right-click any RBAC resource row to open the in-app YAML editor. You can add or remove rules, modify subjects, and apply changes directly. Use care — incorrect RBAC configuration can lock workloads out of necessary API access.

## Custom Resource Definitions (CRDs)

The **CRDs** page lists all CustomResourceDefinitions installed in the cluster. Each entry shows the API group, scope (Cluster or Namespaced), and supported versions. CRDs are cluster-scoped so the namespace selector does not apply.
