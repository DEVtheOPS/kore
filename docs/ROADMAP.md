# Roadmap

Open work items, roughly in priority order. Use GitHub Issues for discussion; this
file is the lightweight at-a-glance list.

## Features

- **Helm lifecycle actions** — releases and the chart catalog are read-only today
  (via the `helm` CLI with capability detection). Add install / upgrade / rollback /
  uninstall and a values viewer.
- **Pod exec / interactive shell** — an interactive terminal tab in the bottom drawer
  that execs into a pod container (kube exec over websocket).
- **Port forwarding** — start / stop / list active forwards for Services and Pods,
  surfaced from the Network section.
- **Custom Resources browser** — CRD list / YAML edit / delete exist. Add browsing and
  editing of CR *instances* and a schema / versions view.
- **Remaining Config kinds** — PriorityClasses, RuntimeClasses, Leases pages, plus
  secret value masking in a dedicated Secret detail view.
- **Network extras** — service / endpoint-specific columns and status visuals.

## Improvements

- **Resource-specific columns in `WorkloadList`** — it currently shows the same
  name / namespace / status / images / age columns for every kind. Add per-kind
  columns (Service ClusterIP/ports, PVC capacity, RoleBinding subjects, CRD
  group/versions, HPA min/max/current, ...).

## Chores

- **Remove the `~/.rustylens` migration** after a few releases
  (`config::migrate_legacy_app_dir` and `ClusterManager::migrate_config_paths`).
