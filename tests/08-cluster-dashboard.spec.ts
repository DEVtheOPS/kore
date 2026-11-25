import { expect, test } from '@playwright/test';
import { clusterMetrics, clusters, namespaces, nodes, nodeUsage, warningEvents } from './screenshots/mock-data';

const [cluster] = clusters;
const dashboard = `/cluster/${cluster.id}`;

test.beforeEach(async ({ page }) => {
  await page.addInitScript(({ cluster, clusterMetrics, namespaces, nodes, nodeUsage, warningEvents }) => {
    Object.assign(window, {
      __TAURI_INTERNALS__: {
        async invoke(command: string, args?: Record<string, unknown>) {
          if (command.startsWith('cluster_') && args?.clusterId !== cluster.id) {
            throw new Error('Unexpected cluster target');
          }

          switch (command) {
            case 'db_list_clusters':
              return [cluster];
            case 'db_get_cluster':
              return args?.id === cluster.id ? cluster : null;
            case 'cluster_list_namespaces':
              return namespaces.filter((namespace) => namespace !== 'all');
            case 'cluster_get_metrics':
              return clusterMetrics;
            case 'cluster_get_events':
              return warningEvents;
            case 'cluster_list_nodes':
              return nodes;
            case 'cluster_get_node_usage':
              return nodeUsage;
            default:
              return null;
          }
        },
      },
    });
  }, { cluster, clusterMetrics, namespaces, nodes, nodeUsage, warningEvents });
});

test('canonical dashboard shows metrics and warnings without the old overview summary', async ({ page }) => {
  const pageErrors: string[] = [];
  page.on('pageerror', (error) => pageErrors.push(error.message));
  await page.clock.install();
  await page.goto(dashboard);

  const main = page.getByRole('main');
  await expect(main.getByRole('heading', { name: 'Cluster Dashboard', exact: true })).toHaveCount(1);
  await expect(main.getByText(cluster.context_name, { exact: true })).toBeVisible();
  for (const name of ['CPU', 'Memory', 'Pods']) {
    await expect(main.getByRole('heading', { name, exact: true })).toHaveCount(1);
  }
  await expect(main.getByText('Capacity:', { exact: true })).toHaveCount(3);
  await expect(main.getByText('20.00', { exact: true })).toBeVisible();
  await expect(main.getByText('80.0 GiB', { exact: true })).toBeVisible();
  await expect(main.getByText('330', { exact: true })).toBeVisible();
  await expect(main.locator('canvas')).toHaveCount(5);
  await expect(main.getByText('Current requested/used resources', { exact: true })).toHaveCount(0);
  await expect(main.getByRole('heading', { name: `Warnings (${warningEvents.length})` })).toBeVisible();
  await expect(main.getByRole('row').filter({ hasText: warningEvents[0].message })).toBeVisible();

  const cpuHistory = main.locator('canvas').nth(1);
  const initialHistory = await cpuHistory.evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL());
  await page.clock.fastForward(5_000);
  await expect.poll(() => cpuHistory.evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL())).not.toBe(initialHistory);
  expect(pageErrors).toEqual([]);
});

test('Nodes shortcut and sidebar dashboard link use cluster-scoped canonical routes', async ({ page }) => {
  await page.goto(dashboard);
  await page.getByRole('link', { name: 'View Nodes', exact: true }).click();
  await expect(page).toHaveURL(`${dashboard}/nodes`);
  await expect(page.getByRole('row').filter({ hasText: nodes[0].name })).toBeVisible();

  const dashboardLink = page.getByRole('navigation').getByRole('link', { name: 'Cluster Dashboard', exact: true });
  await expect(dashboardLink).toHaveAttribute('href', dashboard);
  await expect(page.getByRole('navigation').getByRole('link', { name: 'Overview', exact: true }))
    .toHaveAttribute('href', `${dashboard}/workloads`);
  await dashboardLink.click();
  await expect(page).toHaveURL(dashboard);
  await expect(page.getByRole('heading', { name: `Warnings (${warningEvents.length})` })).toBeVisible();
});

test('a direct legacy dashboard link redirects to the canonical dashboard', async ({ page }) => {
  await page.goto(`${dashboard}/dashboard?source=bookmark`);
  await expect(page).toHaveURL(`${dashboard}?source=bookmark`);
  await expect(page.getByRole('heading', { name: 'Cluster Dashboard', exact: true })).toBeVisible();
});
