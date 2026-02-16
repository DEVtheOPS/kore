# Kore Sidebar Sections - Full Feature Implementation Plan

Date: February 15, 2026
Status: Draft for review (no implementation started)

## 1. Goal
Implement complete, user-meaningful functionality for every sidebar section in Kore (left icon sidebar + cluster resource sidebar), aligned with Kubernetes operator workflows and existing Kore architecture (Tauri v2 + Svelte 5 runes).

## 2. What Was Audited
- Sidebar definitions:
  - `src/lib/components/IconSidebar.svelte`
  - `src/lib/components/ResourceSidebar.svelte`
- Route pages for all sidebar destinations under:
  - `src/routes/+page.svelte`
  - `src/routes/settings/+page.svelte`
  - `src/routes/cluster/[id]/**/+page.svelte`
- Shared list/detail components:
  - `src/lib/components/WorkloadList.svelte`
  - `src/lib/components/PodDetailDrawer.svelte`
  - `src/lib/components/DeploymentDetailDrawer.svelte`
- Backend command surface:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/k8s/*`

## 3. Sidebar Inventory and Current State

### 3.1 Icon Sidebar
1. Overview (`/`): Implemented (cluster inventory table with open, pin, settings, delete).
2. Add Cluster (`+` button): Implemented via import modal.
3. Bookmarked clusters: Implemented with drag reorder + context menu.
4. App Settings (`/settings`): Implemented (theme settings currently).

### 3.2 Cluster Sidebar
1. Cluster Settings (`/cluster/[id]/settings`): Implemented (name/icon/description/tags/delete).
2. Namespace selector: Implemented via `activeClusterStore` and `cluster_list_namespaces`.
3. Dashboard (`/cluster/[id]/dashboard`): Implemented (metrics + warning events).
4. Nodes (`/cluster/[id]/nodes`): Placeholder.

#### Workloads group
- Overview (`/workloads`): Placeholder.
- Pods (`/pods`): Implemented (watch, detail drawer, logs, delete).
- Deployments (`/deployments`): Implemented list + details + delete; edit/delete in drawer still TODO.
- StatefulSets, DaemonSets, ReplicaSets, Jobs, CronJobs: Implemented list/delete via shared `WorkloadList`.

#### Configuration group
- ConfigMaps, Secrets, ResourceQuotas, LimitRanges, HPA, PDB: Implemented list/delete via shared `WorkloadList`.

#### Network group
- Services, Endpoints, Ingresses, NetworkPolicies: Implemented list/delete via shared `WorkloadList`.

#### Storage group
- PVC, PV, StorageClasses: Implemented list/delete via shared `WorkloadList`.

#### Access Control group
- ServiceAccounts, Roles, ClusterRoles: Implemented list/delete via shared `WorkloadList`.
- RoleBindings (`/role-bindings`): Placeholder.
- ClusterRoleBindings (`/cluster-role-bindings`): Placeholder.

#### Standalone
- Namespaces (`/namespaces`): Placeholder.
- Events (`/events`): Placeholder.

#### Helm group
- Releases (`/helm/releases`): Placeholder.
- Charts (`/helm/charts`): Placeholder.

#### Custom Resources group
- CRDs (`/crd`): Placeholder.

## 4. User Jobs To Be Done Per Section (Target UX)

### 4.1 Global/Core
- Discover clusters quickly, pin favorites, switch clusters and namespace safely.
- Understand cluster health at a glance.
- Navigate resources consistently with filter/search/sort and bulk actions.

### 4.2 Workloads
- Inspect runtime state and rollout health.
- Perform operational actions: scale, restart, delete, view events, view logs.
- Open detailed YAML/spec for troubleshooting.

### 4.3 Config and Policy
- Inspect and manage config artifacts (ConfigMap/Secret/etc.).
- Validate policy objects (quota, limits, HPA, PDB) and detect drift.

### 4.4 Network
- Debug service routing and exposure (Service/Endpoints/Ingress).
- Inspect and maintain network segmentation (NetworkPolicy).

### 4.5 Storage
- Track claim/volume binding, capacity, reclaim behavior.
- Maintain storage classes and clean up unused objects.

### 4.6 Access Control
- Audit identities/roles/bindings.
- Add/remove/adjust role bindings and cluster role bindings safely.

### 4.7 Namespaces and Events
- Manage namespace lifecycle.
- Investigate incidents with event timeline and filters.

### 4.8 Helm
- Manage releases lifecycle (install/upgrade/rollback/uninstall).
- Explore available charts and chart metadata.

### 4.9 CRDs
- Discover installed CRDs.
- Inspect CRD schemas/versions and browse instances for selected CRD kinds.

## 5. Gap Analysis

1. Multiple sidebar routes are placeholders with no data/actions: Nodes, Workloads Overview, Namespaces, Events, RoleBindings, ClusterRoleBindings, Helm Charts/Releases, CRDs.
2. Existing implemented sections are mostly list+delete; missing common operator actions (create/edit/apply YAML, scale/restart/rollout, binding management).
3. Drawer actions are incomplete (`TODO` in deployment/pod edit paths).
4. Backend command coverage is absent for several placeholder pages (not registered in `src-tauri/src/lib.rs`).
5. UX consistency gap: no unified "YAML view/edit/apply" and no standardized details drawer for all resource types.

## 6. Implementation Plan (Phased)

## Phase 0 - Foundation and Consistency
Scope:
- Establish shared resource action framework used by all pages.

Deliverables:
1. Build shared "Resource Details Drawer" pattern with tabs:
   - Overview
   - YAML (read-only first)
   - Events (resource-scoped when possible)
2. Introduce shared action contract for DataTable row actions:
   - View details
   - Delete
   - Open YAML
3. Add consistent empty/loading/error states across all resource pages.
4. Add toast/notification conventions for success/failure paths.

Backend:
- Add generic read command(s) for manifest retrieval where needed (or per-resource detail commands).

Acceptance criteria:
- All non-placeholder pages use consistent action semantics and error handling.

## Phase 1 - Complete Placeholder Sections (MVP)
Scope:
- Replace all placeholders with list + details + delete (where deletion is valid).

### 1A. Nodes
Frontend:
- Build `nodes/+page.svelte` using `DataTable`.
- Columns: name, status/conditions, roles, version, age, internal IP, pods count (if available).
- Row details drawer with labels/taints/capacity/allocatable.

Backend:
- Add commands: `cluster_list_nodes`, `cluster_get_node_details`.

### 1B. Workloads Overview
Frontend:
- Build aggregated summary page with cards + grouped tables:
  - Deployments, StatefulSets, DaemonSets, Jobs, CronJobs health snapshot.
- Quick drill-down links into each section.

Backend:
- Reuse existing list commands; optional aggregate endpoint for efficiency.

### 1C. Namespaces
Frontend:
- Table with namespace status, age, labels.
- Actions: view details, delete namespace.
- Optional: create namespace (Phase 2 if needed).

Backend:
- Add commands: `cluster_get_namespaces_full` (or expand existing), `cluster_delete_namespace`.

### 1D. Events
Frontend:
- Table with type, reason, object, namespace, message, last seen, count.
- Filters: namespace, type, reason, object search.
- Auto-refresh/watch toggle.

Backend:
- Reuse `cluster_get_events`; add optional streaming/watch command for real-time mode.

### 1E. RoleBindings and ClusterRoleBindings
Frontend:
- Standard list pages with subject/roleRef columns.
- Details drawer showing subjects, roleRef, metadata.
- Actions: delete.

Backend:
- Add commands:
  - `cluster_list_role_bindings`, `cluster_delete_role_binding`
  - `cluster_list_cluster_role_bindings`, `cluster_delete_cluster_role_binding`

### 1F. Helm Releases and Charts
Frontend:
- Releases page: table + details drawer (status, chart version, app version, namespace, updated).
- Charts page: searchable chart catalog (initially from configured repos).

Backend:
- Add Helm integration command layer (likely wrapping `helm` binary or helm crate):
  - `cluster_list_helm_releases`
  - `cluster_list_helm_charts`
- Include capability detection and graceful fallback when Helm unavailable.

### 1G. CRDs
Frontend:
- CRD table with group, kind, versions, scope, age.
- Details drawer with versions and schema summary.
- Add "View Instances" action (navigates/filter to CR instances view).

Backend:
- Add commands: `cluster_list_crds`, `cluster_get_crd_details`.
- Plan for CR instance listing command shape (Phase 2).

Acceptance criteria for Phase 1:
- No sidebar page shows "Placeholder" or "Coming soon".
- Every sidebar destination has real data load, refresh, and at least one meaningful action.

## Phase 2 - Operator Actions Beyond Delete
Scope:
- Add high-value mutations users expect from a Kubernetes IDE.

Deliverables:
1. YAML workflow for managed resources:
   - View YAML
   - Edit YAML
   - Apply patch/update
2. Workload actions:
   - Deployments/StatefulSets/DaemonSets: scale, restart rollout
   - Jobs: rerun (create from template) where practical
3. Namespace create flow.
4. Access control create/edit for RoleBinding/ClusterRoleBinding.
5. Helm release lifecycle actions:
   - Install
   - Upgrade
   - Rollback
   - Uninstall

Backend:
- Add update/apply commands per resource category with validation and clear error messages.

Acceptance criteria:
- Users can complete common day-2 operations without leaving Kore for kubectl in the primary sections.

## Phase 3 - Advanced UX and Reliability
Scope:
- Improve performance, observability, and safety.

Deliverables:
1. Incremental refresh/watch support for major lists (where stable and efficient).
2. RBAC-aware UI states (disable/hide forbidden actions; surface permission errors clearly).
3. Bulk operations for additional resources.
4. Audit trail style activity panel for recent user actions in session.
5. Test hardening and regression coverage.

Acceptance criteria:
- Measurable reduction in stale data windows and user-facing action failures.

## 7. Technical Work Breakdown

### 7.1 Frontend
1. Create generic page scaffolding util for new resource pages.
2. Build missing pages:
   - `src/routes/cluster/[id]/nodes/+page.svelte`
   - `src/routes/cluster/[id]/workloads/+page.svelte`
   - `src/routes/cluster/[id]/namespaces/+page.svelte`
   - `src/routes/cluster/[id]/events/+page.svelte`
   - `src/routes/cluster/[id]/role-bindings/+page.svelte`
   - `src/routes/cluster/[id]/cluster-role-bindings/+page.svelte`
   - `src/routes/cluster/[id]/helm/charts/+page.svelte`
   - `src/routes/cluster/[id]/helm/releases/+page.svelte`
   - `src/routes/cluster/[id]/crd/+page.svelte`
3. Add reusable drawers/components:
   - Resource metadata panel
   - YAML viewer/editor modal
   - Event list widget
4. Upgrade existing pages to consistent action model:
   - Pods, Deployments, all `WorkloadList` consumers.

### 7.2 Backend (Rust/Tauri)
1. Add command APIs for missing resource types:
   - Nodes, namespaces (full), bindings, CRDs, Helm.
2. Add detail/read commands to support drawers.
3. Add optional mutate commands for Phase 2 (scale/restart/edit/apply).
4. Register commands in `src-tauri/src/lib.rs` and expose typed response structs.
5. Add validation and error normalization (user-safe messages).

### 7.3 Stores and State
1. Extend state handling for per-page filters and persisted table state keys.
2. Keep namespace-scoped behavior consistent (cluster-scoped resources ignore namespace filter cleanly).
3. Add action feedback hooks (toasts, pending states, retry).

## 8. Testing and Quality Gates

Frontend:
- Component/unit tests for new pages and action menus.
- Behavior tests for search/filter/bulk actions and drawer state.

Backend:
- Unit tests for mapping and parsing for new resource structs.
- Command-level tests (mocked where appropriate).

Integration:
- Smoke pass per sidebar route: load, refresh, row action, error path.

Manual validation checklist:
1. Every sidebar item navigates to a functional page.
2. Namespace switch updates namespaced pages correctly.
3. Cluster-scoped resources behave correctly regardless of namespace selection.
4. Delete and mutation confirmations work and recover gracefully on errors.

## 9. Risks and Mitigations

1. Helm portability risk:
- Mitigation: detect Helm availability; provide clear fallback UI state.

2. RBAC variance across clusters:
- Mitigation: treat permission errors as expected states with explicit messaging.

3. Real-time watch complexity:
- Mitigation: start with polling + manual refresh for new sections, then add watch incrementally.

4. YAML mutation safety:
- Mitigation: add confirm step, server-side validation, and precise diff preview before apply (Phase 2).

## 10. Proposed Execution Order

1. Foundation consistency refactor (Phase 0).
2. Placeholder replacement for core ops pages first:
   - Nodes, Events, Namespaces.
3. Access Control completion:
   - RoleBindings, ClusterRoleBindings.
4. CRD browser.
5. Helm pages.
6. Mutation workflows (edit/apply/scale/restart).

## 11. Definition of Done
A sidebar implementation is done when:
1. Route has production UI (not placeholder).
2. Data loads from backend with loading/error/empty states.
3. At least one meaningful user action is available and tested.
4. Action results are reflected in UI without manual app restart.
5. Page follows Kore theme variables and existing component patterns.

## 12. Requested Review
Please review and confirm:
1. Phase ordering.
2. Whether Helm should stay in Phase 1 (MVP) or move to Phase 2.
3. Whether YAML edit/apply should be global in Phase 2 or limited to specific resources first.

After your sign-off, I will implement in the approved sequence.
