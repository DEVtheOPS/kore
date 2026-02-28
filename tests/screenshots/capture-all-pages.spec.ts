/**
 * Screenshot capture for all app pages + key panels.
 *
 * Uses a comprehensive Tauri IPC mock so every page renders with realistic
 * fake data – no real cluster connection needed.
 *
 * Run:  pnpm screenshots
 * Out:  screenshots/  (project root)
 */

import { test, type Page } from '@playwright/test'
import path from 'path'
import fs from 'fs'
import {
  CLUSTER_ID,
  clusters,
  namespaces,
  nodes,
  pods,
  podEvents,
  deployments,
  clusterMetrics,
  warningEvents,
  clusterEvents,
  services,
  configMaps,
  secrets,
  namespacesDetailed,
  statefulsets,
  daemonsets,
  replicasets,
  jobs,
  cronjobs,
  ingresses,
  endpoints,
  networkPolicies,
  pvcs,
  pvs,
  storageClasses,
  roleBindings,
  roles,
  clusterRoles,
  clusterRoleBindings,
  serviceAccounts,
  hpas,
  resourceQuotas,
  limitRanges,
  pdbs,
  helmReleases,
  helmCharts,
  crds,
  resourceYaml,
  helmAvailability,
} from './mock-data'

// ─── Constants ────────────────────────────────────────────────────────────────

const OUT_DIR = path.resolve('screenshots')
const CLUSTER_BASE = `/cluster/${CLUSTER_ID}`
const WAIT_TIMEOUT = 8_000

// Bundle all mock responses into a single JSON-serialisable object that can be
// passed into the browser context via addInitScript.
const MOCK_PAYLOAD = {
  clusters,
  namespaces,
  nodes,
  pods,
  podEvents,
  deployments,
  clusterMetrics,
  warningEvents,
  clusterEvents,
  services,
  configMaps,
  secrets,
  namespacesDetailed,
  statefulsets,
  daemonsets,
  replicasets,
  jobs,
  cronjobs,
  ingresses,
  endpoints,
  networkPolicies,
  pvcs,
  pvs,
  storageClasses,
  roleBindings,
  roles,
  clusterRoles,
  clusterRoleBindings,
  serviceAccounts,
  hpas,
  resourceQuotas,
  limitRanges,
  pdbs,
  helmReleases,
  helmCharts,
  crds,
  resourceYaml,
  helmAvailability,
  clusterId: CLUSTER_ID,
}

// ─── Tauri Mock injection ─────────────────────────────────────────────────────

/**
 * Injects window.__TAURI_INTERNALS__ before any app scripts load so that
 * @tauri-apps/api invoke() and listen() calls return fake data.
 */
async function injectTauriMock(page: Page) {
  await page.addInitScript((data: typeof MOCK_PAYLOAD) => {
    const clusterId = data.clusterId

    // Core invoke handler – maps Tauri command names to mock responses
    async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
      switch (cmd) {
        // ── Cluster DB ──────────────────────────────────────────────────────
        case 'db_list_clusters':
          return data.clusters
        case 'db_get_cluster':
          return data.clusters.find((c) => c.id === (args as { id: string })?.id) ?? data.clusters[0]
        case 'db_update_cluster':
        case 'db_delete_cluster':
        case 'db_update_last_accessed':
          return null

        // ── Namespaces ──────────────────────────────────────────────────────
        case 'cluster_list_namespaces':
          return data.namespaces.filter((n) => n !== 'all')

        // ── Cluster metrics & events ────────────────────────────────────────
        case 'cluster_get_metrics':
          return data.clusterMetrics
        case 'cluster_get_events':
          return data.warningEvents

        // ── Nodes ───────────────────────────────────────────────────────────
        case 'cluster_list_nodes':
          return data.nodes

        // ── Pods ────────────────────────────────────────────────────────────
        case 'list_pods':
          return data.pods
        case 'start_pod_watch':
          // Simulate an initial pod list via a synthetic event shortly after
          return null
        case 'get_pod_events':
          return data.podEvents
        case 'delete_pod':
          return null

        // ── Deployments ─────────────────────────────────────────────────────
        case 'cluster_list_deployments':
          return data.deployments
        case 'cluster_delete_deployment':
        case 'cluster_scale_workload':
        case 'cluster_restart_workload':
          return null

        // ── StatefulSets ────────────────────────────────────────────────────
        case 'cluster_list_statefulsets':
          return data.statefulsets
        case 'cluster_delete_statefulset':
          return null

        // ── DaemonSets ──────────────────────────────────────────────────────
        case 'cluster_list_daemonsets':
          return data.daemonsets
        case 'cluster_delete_daemonset':
          return null

        // ── ReplicaSets ─────────────────────────────────────────────────────
        case 'cluster_list_replicasets':
          return data.replicasets
        case 'cluster_delete_replicaset':
          return null

        // ── Jobs ─────────────────────────────────────────────────────────────
        case 'cluster_list_jobs':
          return data.jobs
        case 'cluster_delete_job':
          return null

        // ── CronJobs ─────────────────────────────────────────────────────────
        case 'cluster_list_cronjobs':
          return data.cronjobs
        case 'cluster_delete_cronjob':
          return null

        // ── Services ─────────────────────────────────────────────────────────
        case 'cluster_list_services':
          return data.services
        case 'cluster_delete_service':
          return null

        // ── Endpoints ────────────────────────────────────────────────────────
        case 'cluster_list_endpoints':
          return data.endpoints

        // ── Ingresses ────────────────────────────────────────────────────────
        case 'cluster_list_ingresses':
          return data.ingresses
        case 'cluster_delete_ingress':
          return null

        // ── Network Policies ─────────────────────────────────────────────────
        case 'cluster_list_network_policies':
          return data.networkPolicies
        case 'cluster_delete_network_policy':
          return null

        // ── ConfigMaps ────────────────────────────────────────────────────────
        case 'cluster_list_config_maps':
          return data.configMaps
        case 'cluster_delete_config_map':
          return null

        // ── Secrets ───────────────────────────────────────────────────────────
        case 'cluster_list_secrets':
          return data.secrets
        case 'cluster_delete_secret':
          return null

        // ── Namespaces detailed ───────────────────────────────────────────────
        case 'cluster_list_namespaces_detailed':
          return data.namespacesDetailed
        case 'cluster_delete_namespace':
          return null

        // ── PVCs ──────────────────────────────────────────────────────────────
        case 'cluster_list_pvcs':
          return data.pvcs
        case 'cluster_delete_pvc':
          return null

        // ── PVs ───────────────────────────────────────────────────────────────
        case 'cluster_list_pvs':
          return data.pvs
        case 'cluster_delete_pv':
          return null

        // ── Storage Classes ───────────────────────────────────────────────────
        case 'cluster_list_storage_classes':
          return data.storageClasses
        case 'cluster_delete_storage_class':
          return null

        // ── RBAC ──────────────────────────────────────────────────────────────
        case 'cluster_list_role_bindings':
          return data.roleBindings
        case 'cluster_list_roles':
          return data.roles
        case 'cluster_list_cluster_roles':
          return data.clusterRoles
        case 'cluster_list_cluster_role_bindings':
          return data.clusterRoleBindings
        case 'cluster_list_service_accounts':
          return data.serviceAccounts

        // ── HPA ───────────────────────────────────────────────────────────────
        case 'cluster_list_hpas':
          return data.hpas
        case 'cluster_delete_hpa':
          return null

        // ── Resource Quotas ───────────────────────────────────────────────────
        case 'cluster_list_resource_quotas':
          return data.resourceQuotas
        case 'cluster_delete_resource_quota':
          return null

        // ── Limit Ranges ──────────────────────────────────────────────────────
        case 'cluster_list_limit_ranges':
          return data.limitRanges
        case 'cluster_delete_limit_range':
          return null

        // ── PDBs ──────────────────────────────────────────────────────────────
        case 'cluster_list_pdbs':
          return data.pdbs
        case 'cluster_delete_pdb':
          return null

        // ── Events ────────────────────────────────────────────────────────────
        case 'cluster_list_events':
          return data.clusterEvents

        // ── Helm ──────────────────────────────────────────────────────────────
        case 'cluster_check_helm_available':
          return data.helmAvailability
        case 'cluster_list_helm_releases':
          return data.helmReleases
        case 'cluster_list_helm_charts':
          return data.helmCharts

        // ── CRDs ──────────────────────────────────────────────────────────────
        case 'cluster_list_crds':
          return data.crds
        case 'cluster_delete_crd':
          return null

        // ── YAML ──────────────────────────────────────────────────────────────
        case 'cluster_get_resource_yaml':
          return data.resourceYaml
        case 'cluster_apply_resource_yaml':
          return null

        // ── Import / misc ─────────────────────────────────────────────────────
        case 'import_discover_file':
        case 'import_discover_folder':
          return []
        case 'import_add_cluster':
        case 'process_icon_file':
          return null
        case 'stream_container_logs':
        case 'stop_stream_logs':
          return null

        default:
          console.warn(`[Tauri Mock] Unhandled command: "${cmd}"`, args)
          return null
      }
    }

    // Minimal event system – stores listeners so we can fire synthetic events
    const listeners: Record<string, ((payload: unknown) => void)[]> = {}

    function mockListen(
      event: string,
      handler: (e: { payload: unknown }) => void,
    ): Promise<() => void> {
      if (!listeners[event]) listeners[event] = []
      listeners[event].push((payload) => handler({ payload }))

      // For pod_event: immediately fire a synthetic "Restarted" payload so the
      // pods page renders populated rows without needing a real watch stream.
      if (event === 'pod_event') {
        setTimeout(() => {
          handler({ payload: { type: 'Restarted', payload: data.pods } })
        }, 50)
      }

      return Promise.resolve(() => {
        const arr = listeners[event] ?? []
        const idx = arr.findIndex((h) => h === handler)
        if (idx !== -1) arr.splice(idx, 1)
      })
    }

    // Install mock – Tauri v2 reads window.__TAURI_INTERNALS__.invoke / .listen
    ;(window as unknown as Record<string, unknown>)['__TAURI_INTERNALS__'] = {
      invoke: (cmd: string, args?: Record<string, unknown>) =>
        mockInvoke(cmd, args).catch((err) => {
          console.error(`[Tauri Mock] invoke error for "${cmd}":`, err)
          return null
        }),
      listen: mockListen,
      emit: () => Promise.resolve(),
      transformCallback: () => 0,
      metadata: { currentWindow: { label: 'main' } },
    }
  }, MOCK_PAYLOAD)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

test.beforeAll(() => {
  if (!fs.existsSync(OUT_DIR)) {
    fs.mkdirSync(OUT_DIR, { recursive: true })
  }
})

async function capture(page: Page, name: string, fullPage = true) {
  await page.screenshot({
    path: path.join(OUT_DIR, `${name}.png`),
    fullPage,
  })
}

/**
 * Navigate to a URL and wait for the page to settle before screenshotting.
 * – waits for domcontentloaded
 * – waits for any visible loading spinner to disappear (if present)
 * – gives Svelte reactivity a short tick to flush
 */
async function goAndWait(page: Page, url: string) {
  await page.goto(url)
  await page.waitForLoadState('domcontentloaded')

  // Dismiss loading skeletons / spinners if they appear
  try {
    await page
      .locator('[data-loading="true"], .loading, [aria-busy="true"]')
      .waitFor({ state: 'hidden', timeout: 3_000 })
  } catch {
    // No loading indicator – that's fine
  }

  // Allow Svelte reactive state to flush and Chart.js to render
  await page.waitForTimeout(600)
}

// ─── Shared setup ────────────────────────────────────────────────────────────

test.beforeEach(async ({ page }) => {
  await injectTauriMock(page)
})

// ─── Global pages ────────────────────────────────────────────────────────────

test('01-home-cluster-list', async ({ page }) => {
  await goAndWait(page, '/')
  await capture(page, '01-home-cluster-list')
})

test('02-settings', async ({ page }) => {
  await goAndWait(page, '/settings')
  await capture(page, '02-settings')
})

// ─── Cluster pages ────────────────────────────────────────────────────────────

test('03-cluster-overview', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}`)
  await capture(page, '03-cluster-overview')
})

test('04-cluster-dashboard', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/dashboard`)
  await capture(page, '04-cluster-dashboard')
})

test('05-cluster-nodes', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/nodes`)
  await capture(page, '05-cluster-nodes')
})

test('06-cluster-workloads', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/workloads`)
  await capture(page, '06-cluster-workloads')
})

test('07-cluster-pods', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/pods`)
  // Pods page uses a watch stream; give it a moment to receive the synthetic event
  await page.waitForTimeout(300)
  await capture(page, '07-cluster-pods')
})

test('08-cluster-deployments', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/deployments`)
  await capture(page, '08-cluster-deployments')
})

test('09-cluster-statefulsets', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/statefulsets`)
  await capture(page, '09-cluster-statefulsets')
})

test('10-cluster-daemonsets', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/daemonsets`)
  await capture(page, '10-cluster-daemonsets')
})

test('11-cluster-replicasets', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/replicasets`)
  await capture(page, '11-cluster-replicasets')
})

test('12-cluster-jobs', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/jobs`)
  await capture(page, '12-cluster-jobs')
})

test('13-cluster-cronjobs', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/cronjobs`)
  await capture(page, '13-cluster-cronjobs')
})

test('14-cluster-namespaces', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/namespaces`)
  await capture(page, '14-cluster-namespaces')
})

test('15-cluster-services', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/services`)
  await capture(page, '15-cluster-services')
})

test('16-cluster-endpoints', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/endpoints`)
  await capture(page, '16-cluster-endpoints')
})

test('17-cluster-ingresses', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/ingresses`)
  await capture(page, '17-cluster-ingresses')
})

test('18-cluster-network-policies', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/network-policies`)
  await capture(page, '18-cluster-network-policies')
})

test('19-cluster-config-maps', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/config-maps`)
  await capture(page, '19-cluster-config-maps')
})

test('20-cluster-secrets', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/secrets`)
  await capture(page, '20-cluster-secrets')
})

test('21-cluster-pvc', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/pvc`)
  await capture(page, '21-cluster-pvc')
})

test('22-cluster-pv', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/pv`)
  await capture(page, '22-cluster-pv')
})

test('23-cluster-storage-classes', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/storage-classes`)
  await capture(page, '23-cluster-storage-classes')
})

test('24-cluster-hpa', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/hpa`)
  await capture(page, '24-cluster-hpa')
})

test('25-cluster-resource-quotas', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/resource-quotas`)
  await capture(page, '25-cluster-resource-quotas')
})

test('26-cluster-limit-ranges', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/limit-ranges`)
  await capture(page, '26-cluster-limit-ranges')
})

test('27-cluster-pdb', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/pdb`)
  await capture(page, '27-cluster-pdb')
})

test('28-cluster-roles', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/roles`)
  await capture(page, '28-cluster-roles')
})

test('29-cluster-role-bindings', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/role-bindings`)
  await capture(page, '29-cluster-role-bindings')
})

test('30-cluster-cluster-roles', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/cluster-roles`)
  await capture(page, '30-cluster-cluster-roles')
})

test('31-cluster-cluster-role-bindings', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/cluster-role-bindings`)
  await capture(page, '31-cluster-cluster-role-bindings')
})

test('32-cluster-service-accounts', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/service-accounts`)
  await capture(page, '32-cluster-service-accounts')
})

test('33-cluster-helm-releases', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/helm/releases`)
  await capture(page, '33-cluster-helm-releases')
})

test('34-cluster-helm-charts', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/helm/charts`)
  await capture(page, '34-cluster-helm-charts')
})

test('35-cluster-crd', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/crd`)
  await capture(page, '35-cluster-crd')
})

test('36-cluster-events', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/events`)
  await capture(page, '36-cluster-events')
})

test('37-cluster-settings', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/settings`)
  await capture(page, '37-cluster-settings')
})

// ─── Side panels ─────────────────────────────────────────────────────────────

test('38-panel-pod-detail', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/pods`)
  await page.waitForTimeout(300) // let watch event populate rows

  // Click first table row to open the pod detail drawer
  const firstRow = page.locator('table tbody tr, [role="row"]:not([role="columnheader"])').first()
  if (await firstRow.isVisible({ timeout: WAIT_TIMEOUT })) {
    await firstRow.click()
    await page.waitForTimeout(600) // wait for drawer animation
  }

  await capture(page, '38-panel-pod-detail', false)
})

test('39-panel-deployment-detail', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/deployments`)

  const firstRow = page.locator('table tbody tr, [role="row"]:not([role="columnheader"])').first()
  if (await firstRow.isVisible({ timeout: WAIT_TIMEOUT })) {
    await firstRow.click()
    await page.waitForTimeout(600)
  }

  await capture(page, '39-panel-deployment-detail', false)
})

test('40-panel-yaml-editor', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/deployments`)

  // Right-click (or find an "Edit YAML" / kebab menu) on the first row.
  // Most resource pages expose a context menu with an Edit option.
  const firstRow = page.locator('table tbody tr, [role="row"]:not([role="columnheader"])').first()
  if (await firstRow.isVisible({ timeout: WAIT_TIMEOUT })) {
    await firstRow.click({ button: 'right' })
    await page.waitForTimeout(300)

    // Try to find and click an "Edit" / "Edit YAML" menu item
    const editItem = page
      .getByRole('menuitem')
      .filter({ hasText: /edit yaml|edit|yaml/i })
      .first()
    if (await editItem.isVisible({ timeout: 2_000 })) {
      await editItem.click()
      await page.waitForTimeout(600)
    }
  }

  await capture(page, '40-panel-yaml-editor', false)
})

test('41-panel-logs-bottom-drawer', async ({ page }) => {
  await goAndWait(page, `${CLUSTER_BASE}/pods`)
  await page.waitForTimeout(300)

  // Look for a "Logs" or "View Logs" button in the row actions or context menu
  const firstRow = page.locator('table tbody tr, [role="row"]:not([role="columnheader"])').first()
  if (await firstRow.isVisible({ timeout: WAIT_TIMEOUT })) {
    // Try right-click context menu first
    await firstRow.click({ button: 'right' })
    await page.waitForTimeout(300)

    const logsItem = page
      .getByRole('menuitem')
      .filter({ hasText: /logs|view logs/i })
      .first()
    if (await logsItem.isVisible({ timeout: 2_000 })) {
      await logsItem.click()
    } else {
      // Dismiss context menu, try clicking row to open detail then find "View Logs"
      await page.keyboard.press('Escape')
      await firstRow.click()
      await page.waitForTimeout(400)
      const logsBtn = page.getByRole('button').filter({ hasText: /view logs|logs/i }).first()
      if (await logsBtn.isVisible({ timeout: 2_000 })) {
        await logsBtn.click()
      }
    }

    await page.waitForTimeout(600)
  }

  await capture(page, '41-panel-logs-bottom-drawer', false)
})
