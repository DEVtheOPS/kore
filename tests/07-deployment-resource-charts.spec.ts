import { expect, test } from '@playwright/test';
import { clusters, deploymentDetails, deploymentResourceUsage, deployments } from './screenshots/mock-data';

test('deployment charts keep requests and limits visible while live usage is unavailable', async ({ page }) => {
  const [cluster] = clusters;
  const [deployment] = deployments;
  const [details] = deploymentDetails;
  const report = deploymentResourceUsage['api-server'];
  const missingMetrics = 'Metrics API not available (pod usage requires metrics-server to be installed)';

  await page.clock.install();
  await page.addInitScript(({ cluster, deployment, details, report, missingMetrics }) => {
    let usageChecks = 0;

    Object.assign(window, {
      __TAURI_INTERNALS__: {
        async invoke(command: string, args?: Record<string, unknown>) {
          switch (command) {
            case 'db_list_clusters':
              return [cluster];
            case 'db_get_cluster':
              return cluster;
            case 'cluster_list_namespaces':
              return [deployment.namespace];
            case 'cluster_list_deployments':
              return [deployment];
            case 'cluster_get_deployment_details':
              return details;
            case 'cluster_get_deployment_pods':
            case 'cluster_get_deployment_replicasets':
            case 'cluster_get_deployment_events':
              return [];
            case 'cluster_get_deployment_resource_usage':
              if (args?.clusterId !== cluster.id || args?.namespace !== deployment.namespace ||
                  args?.deploymentName !== deployment.name) {
                throw new Error('Unexpected deployment chart target');
              }
              if (usageChecks++ > 0) return report;
              return {
                pods: report.pods.map((pod) => ({ ...pod, cpu_millicores: null, memory_bytes: null, window: null })),
                usage_error: missingMetrics,
              };
            default:
              return null;
          }
        },
      },
    });
  }, { cluster, deployment, details, report, missingMetrics });

  await page.goto(`/cluster/${cluster.id}/deployments`);
  await page.getByRole('row').filter({ hasText: deployment.name }).click();

  const drawer = page.getByRole('dialog', { name: `Deployment: ${deployment.name}` });
  const unavailable = drawer.getByText(missingMetrics, { exact: false });
  const totals = drawer.getByRole('list', { name: 'Resource totals' });
  const usage = totals.getByRole('listitem').filter({ hasText: 'Usage' });
  const requests = totals.getByRole('listitem').filter({ hasText: 'Requests' });
  const limits = totals.getByRole('listitem').filter({ hasText: 'Limits' });
  await expect(unavailable).toBeVisible();
  await expect(drawer.locator('canvas')).toBeVisible();
  await expect(usage).toHaveText('Usage Unavailable');
  await expect(requests).toHaveText('Requests 150m');
  await expect(limits).toHaveText('Limits 700m');

  await drawer.getByRole('button', { name: 'Memory', exact: true }).click();
  await expect(requests).toHaveText('Requests 320Mi');
  await expect(limits).toHaveText('Limits 640Mi');

  await page.clock.fastForward(15_000);
  await expect(unavailable).toBeHidden();
  await expect(usage).toHaveText('Usage 180Mi');
  await drawer.getByRole('button', { name: 'CPU', exact: true }).click();
  await expect(usage).toHaveText('Usage 120m');
});
