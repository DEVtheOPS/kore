import { expect, test, type Page } from '@playwright/test';
import { clusterMetrics, clusters, warningEvents } from './screenshots/mock-data';

const [a, b] = clusters;
const route = (id: string) => `/cluster/${id}`;
const metric = 'cluster_get_metrics';
const warning = 'cluster_get_events';
const metadata = 'db_get_cluster';
const namespaces = 'cluster_list_namespaces';
type Command = { command: string; id: string };
type Mock = { calls: Command[]; release: (index: number, value: unknown) => void };

declare global {
  interface Window {
    dashboardMock: Mock;
  }
}

async function setup(page: Page, held: Command[], metricsError?: string) {
  await page.clock.install();
  await page.addInitScript(({ a, b, held, clusterMetrics, event, metricsError }) => {
    const calls: Command[] = [];
    const releases = new Map<number, (value: unknown) => void>();
    const mock: Mock = {
      calls,
      release(index, value) {
        const resolve = releases.get(index);
        if (!resolve) throw new Error(`Request ${index} is not pending`);
        releases.delete(index);
        resolve(value);
      },
    };

    Object.assign(window, {
      dashboardMock: mock,
      __TAURI_INTERNALS__: {
        async invoke(command: string, args?: { id?: string; clusterId?: string }) {
          const id = args?.clusterId ?? args?.id ?? '';
          const index = calls.length;
          calls.push({ command, id });
          const hold = held.findIndex((entry) => entry.command === command && entry.id === id);
          if (hold !== -1) {
            held.splice(hold, 1);
            return new Promise((resolve) => releases.set(index, resolve));
          }
          switch (command) {
            case 'db_list_clusters':
              return [a, b];
            case 'db_get_cluster':
              return [a, b].find((cluster) => cluster.id === id);
            case 'cluster_list_namespaces':
              return [`${id}-zeta`, `${id}-alpha`];
            case 'cluster_get_metrics':
              if (metricsError) throw metricsError;
              return id === a.id ? clusterMetrics : { ...clusterMetrics, cpu: { ...clusterMetrics.cpu, capacity: 40 } };
            case 'cluster_get_events':
              return [{ ...event, message: `Warning for ${id}` }];
            default:
              return null;
          }
        },
      },
    });
  }, { a, b, held, clusterMetrics, event: warningEvents[0], metricsError });
  await page.goto(route(a.id));
}

function observed(page: Page) {
  return page.evaluate(() => window.dashboardMock.calls);
}

async function waitForRequest(page: Page, command: string, id: string) {
  await expect.poll(async () => (await observed(page)).some((call) => call.command === command && call.id === id)).toBe(true);
}

async function release(page: Page, command: string, id: string, value: unknown) {
  await waitForRequest(page, command, id);
  await page.evaluate(({ command, id, value }) => {
    const mock = window.dashboardMock;
    mock.release(mock.calls.findIndex((call) => call.command === command && call.id === id), value);
  }, { command, id, value });
}

async function navigate(page: Page, href: string) {
  await page.evaluate((href) => {
    const link = Object.assign(document.createElement('a'), { href });
    document.body.append(link);
    link.click();
    link.remove();
  }, href);
  await expect(page).toHaveURL(href);
}

async function expectDashboard(page: Page, cluster: typeof a, cpuCapacity: string) {
  await expect(page.locator('aside.w-64').getByText(cluster.name, { exact: true })).toBeVisible();
  await expect(page.getByText(cluster.context_name, { exact: true })).toBeVisible();
  await expect(page.getByText(cpuCapacity, { exact: true })).toBeVisible();
  await expect(page.getByText(`Warning for ${cluster.id}`, { exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: 'View Nodes' })).toHaveAttribute('href', `${route(cluster.id)}/nodes`);
}

async function expectNamespaces(page: Page, id: string) {
  const select = page.getByRole('button', { name: 'Namespace' });
  await expect(select).toHaveText('all');
  await select.click();
  await expect(page.locator('#namespace-select + div button')).toHaveText(['all', `${id}-alpha`, `${id}-zeta`]);
}

test('late A metadata cannot change B sidebar, dashboard, actions, or IPC', async ({ page }) => {
  await setup(page, [{ command: metadata, id: a.id }]);
  await waitForRequest(page, metadata, a.id);
  await navigate(page, route(b.id));
  await expectDashboard(page, b, '40.00');
  await release(page, metadata, a.id, a);
  await page.clock.runFor(6_000);
  await expectDashboard(page, b, '40.00');
  await expectNamespaces(page, b.id);
  const calls = (await observed(page)).filter(({ command }) => [metric, warning, namespaces].includes(command));
  expect(calls.map(({ id }) => id)).toEqual(Array(calls.length).fill(b.id));
  expect(calls.filter(({ command }) => command === metric)).toHaveLength(2);
});

test('late A namespaces cannot reach B while B metadata is pending', async ({ page }) => {
  await setup(page, [{ command: namespaces, id: a.id }, { command: metadata, id: b.id }]);
  await waitForRequest(page, namespaces, a.id);
  await navigate(page, route(b.id));
  await release(page, namespaces, a.id, ['stale-a']);
  await expect(page.getByText('Loading cluster...', { exact: true })).toBeVisible();
  await expect(page.locator('aside.w-64')).toHaveCount(0);
  expect((await observed(page)).filter(({ command }) => [metric, warning].includes(command))).toEqual([]);
  await release(page, metadata, b.id, b);
  await expectDashboard(page, b, '40.00');
  await expectNamespaces(page, b.id);
});

test('A to B to A ignores an older A namespace response and resets dashboard history', async ({ page }) => {
  await setup(page, [{ command: namespaces, id: a.id }, { command: metric, id: a.id }]);
  await waitForRequest(page, namespaces, a.id);
  await navigate(page, route(b.id));
  await expectDashboard(page, b, '40.00');
  await page.clock.runFor(6_000);
  await navigate(page, route(a.id));
  await expect(page.getByText('Loading metrics...', { exact: true })).toBeVisible();
  await expect(page.locator('main canvas')).toHaveCount(0);
  await release(page, namespaces, a.id, ['stale-first-a']);
  await expectNamespaces(page, a.id);
  await release(page, metric, a.id, clusterMetrics);
  await expectDashboard(page, a, '20.00');
  const requests = (await observed(page)).filter(({ command }) => [metric, warning].includes(command));
  expect(requests.map(({ id }) => id)).toEqual([b.id, b.id, b.id, b.id, a.id, a.id]);
});

test('a late A dashboard sample cannot appear or restart polling on B', async ({ page }) => {
  await setup(page, [{ command: metric, id: a.id }, { command: warning, id: a.id }]);
  await waitForRequest(page, warning, a.id);
  await navigate(page, route(b.id));
  await expectDashboard(page, b, '40.00');
  const cpuHistory = page.locator('main canvas').nth(1);
  const initialHistory = await cpuHistory.evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL());
  await release(page, metric, a.id, clusterMetrics);
  await release(page, warning, a.id, [{ ...warningEvents[0], message: 'Late A warning' }]);
  await page.clock.runFor(100);
  await expectDashboard(page, b, '40.00');
  await expect(page.getByText('Late A warning', { exact: true })).toHaveCount(0);
  await expect(page.getByText('20.00', { exact: true })).toHaveCount(0);
  expect(await cpuHistory.evaluate((canvas: HTMLCanvasElement) => canvas.toDataURL())).toBe(initialHistory);
  await page.clock.runFor(6_000);
  const requests = (await observed(page)).filter(({ command }) => [metric, warning].includes(command));
  expect(requests.map(({ id }) => id)).toEqual([a.id, a.id, b.id, b.id, b.id, b.id]);
});

test('denied metrics show the backend error without hiding warnings', async ({ page }) => {
  const denial = 'Forbidden: metrics.k8s.io is denied';
  await setup(page, [], denial);
  await expect(page.getByText(`Failed to load cluster metrics: ${denial}`, { exact: true })).toBeVisible();
  await expect(page.getByText(`Warning for ${a.id}`, { exact: true })).toBeVisible();
  await expect(page.getByText('Loading metrics...', { exact: true })).toHaveCount(0);
});

test('a slow poll does not overlap or continue after the dashboard unmounts', async ({ page }) => {
  await setup(page, [{ command: metric, id: a.id }]);
  const polls = async () => (await observed(page)).filter(({ command }) => [metric, warning].includes(command));
  await expect.poll(async () => (await polls()).filter(({ command }) => command === metric)).toHaveLength(1);
  const initial = await polls();
  await page.clock.runFor(12_000);
  expect(await polls()).toEqual(initial);
  await navigate(page, '/');
  await release(page, metric, a.id, clusterMetrics);
  await page.clock.runFor(12_000);
  expect(await polls()).toEqual(initial);
});
