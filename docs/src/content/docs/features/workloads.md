---
title: Workloads
description: Manage pods, deployments, StatefulSets, DaemonSets, Jobs, and CronJobs.
---

The **Workloads** section covers all the resource types that run your applications.

## Pods

The Pods page provides a live view of all pods in the selected namespace.

### Real-time watch stream

Kore opens a Kubernetes watch stream when you navigate to the Pods page. Changes — new pods, status transitions, deletions — appear immediately without polling.

### Pod detail panel

Click any pod row to open the detail panel on the right side:

- **Overview** — node, IP addresses, QoS class, controlled-by, service account
- **Containers** — each container's image, ports, environment variables, resource requests/limits, and probe configuration
- **Conditions** — `PodScheduled`, `Initialized`, `ContainersReady`, `Ready` with status and messages
- **Volumes** — mounted volumes with type and source
- **Labels & Annotations** — full key/value list
- **Events** — recent events for this specific pod

### Viewing logs

From the pod detail panel, click **View Logs** to open the log tab in the bottom drawer. Kore streams output from the container in real time. Multiple log tabs can be open simultaneously — each runs as an independent stream.

### Pod actions

Right-click any row (or use the row action menu) to:

| Action | Description |
|--------|-------------|
| **Delete** | Delete the pod immediately (the owning controller will reschedule it) |
| **View Logs** | Open a live log stream |
| **Edit YAML** | Open the full pod YAML in the in-app editor |
| **Copy name** | Copy the pod name to clipboard |

---

## Deployments

The Deployments page shows all Deployments in the active namespace with replica counts and availability status.

### Deployment detail panel

Clicking a Deployment row opens its detail panel showing the template spec, strategy, conditions, and related ReplicaSets.

### Deployment actions

| Action | How |
|--------|-----|
| **Scale** | Right-click → Scale, enter desired replica count |
| **Restart** | Right-click → Restart (triggers a rolling restart via annotation) |
| **Edit YAML** | Right-click → Edit YAML — make changes and apply |
| **Delete** | Right-click → Delete |

---

## StatefulSets

StatefulSets are managed with the same interface as Deployments: list view, detail panel, scale, restart, and YAML editing.

Ordered pod identity and stable network identities are visible in the pod list — pods are named `<statefulset>-0`, `<statefulset>-1`, etc.

---

## DaemonSets

The DaemonSet list shows `desired`, `current`, `ready`, `available`, and `misscheduled` counts. All standard actions (edit, delete, restart) are available.

---

## ReplicaSets

ReplicaSets are usually managed through Deployments, but Kore exposes them directly for inspection. You can see which Deployment owns each ReplicaSet and the full pod template.

---

## Jobs

The Jobs list shows each Job's completion status (`0/1 Complete`, `1/1 Complete`, etc.), duration, and creation time. Completed jobs are shown alongside active ones.

---

## CronJobs

The CronJobs page shows the schedule expression, last scheduled time, and active job count. From the detail panel you can inspect the Job template and trigger a manual run (via `kubectl create job` equivalent).

---

## Workloads overview

Navigate to **Workloads** in the sidebar for a cross-type summary view. All Deployments, StatefulSets, DaemonSets, ReplicaSets, Jobs, and CronJobs are shown together, grouped by type, making it easy to see cluster health at a glance.
