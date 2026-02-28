---
title: Quick Start
description: Connect your first cluster and start exploring resources in under two minutes.
sidebar:
  order: 2
---

import { Steps } from '@astrojs/starlight/components'

This guide walks you through connecting your first cluster and navigating the core UI.

<Steps>

1. **Launch Kore**

   Open Kore from your Applications folder, Start menu, or by running the AppImage.

   The **Clusters** overview opens on first launch — it will be empty until you import a cluster.

2. **Import a cluster**

   Click the **+** button in the icon sidebar (left edge of the window) to open the import dialog.

   Kore reads your existing `kubeconfig` file — nothing is copied or stored in the cloud.

   - **From file** — browse to a specific `kubeconfig` (handy for per-cluster files)
   - **From folder** — scan a directory and Kore finds all valid configs automatically

   Select the contexts you want to add and click **Import**.

3. **Open a cluster**

   Your imported clusters appear in the **Clusters** list. Click any row to open it.

   Kore connects to the cluster and loads the namespace list. The resource sidebar expands on the left with all available resource types.

4. **Explore resources**

   Navigate using the resource sidebar:

   - **Workloads** → Pods, Deployments, StatefulSets, DaemonSets, Jobs, CronJobs
   - **Configuration** → ConfigMaps, Secrets, Resource Quotas, HPA
   - **Network** → Services, Ingresses, Endpoints, Network Policies
   - **Storage** → PVCs, PVs, Storage Classes
   - **Access Control** → RBAC roles, bindings, and service accounts
   - **Helm** → Installed releases and chart index
   - **Events** → Cluster-wide event stream

5. **Filter by namespace**

   Use the **namespace selector** at the top of the resource sidebar to filter all views to a specific namespace, or select **all** to see cluster-wide resources.

6. **View resource details**

   Click any table row to open the detail panel on the right. For pods, this shows container info, conditions, volumes, labels, annotations, and recent events. Click **View Logs** to tail the container output live.

</Steps>

---

## Tips

- **Bookmark clusters** — hover a cluster card and click the bookmark icon to pin it to the icon sidebar for one-click access.
- **Keyboard navigation** — press `Ctrl+K` / `Cmd+K` to open the command palette (coming soon).
- **Edit YAML** — right-click any resource row and choose **Edit YAML** to open the in-app editor and apply changes directly.
- **Real-time pods** — the Pods page uses a watch stream; new pods, deletions, and status changes appear automatically without refreshing.
