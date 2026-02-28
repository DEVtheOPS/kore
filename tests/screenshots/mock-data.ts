/**
 * Comprehensive fake Kubernetes data for screenshot captures.
 * All data is synthetic – no real cluster info, credentials, or private details.
 */

export const CLUSTER_ID = 'demo-cluster-001'

const now = Date.now()
const d = (n: number) => now - n * 86_400_000
const h = (n: number) => now - n * 3_600_000
const m = (n: number) => now - n * 60_000

function age(n: number, unit: 'd' | 'h' | 'm'): string {
  return `${n}${unit}`
}

// ─── Clusters ─────────────────────────────────────────────────────────────────

export const clusters = [
  {
    id: 'demo-cluster-001',
    name: 'acme-production',
    context_name: 'gke_acme-corp_us-east1_production',
    icon: null,
    description: 'Primary production cluster – US East',
    tags: JSON.stringify(['production', 'gke', 'us-east']),
    created_at: d(90),
    last_accessed: h(2),
  },
  {
    id: 'demo-cluster-002',
    name: 'acme-staging',
    context_name: 'gke_acme-corp_eu-west1_staging',
    icon: null,
    description: 'Staging environment – EU West',
    tags: JSON.stringify(['staging', 'gke', 'eu-west']),
    created_at: d(60),
    last_accessed: h(24),
  },
  {
    id: 'demo-cluster-003',
    name: 'local-dev',
    context_name: 'minikube',
    icon: null,
    description: 'Local development cluster',
    tags: JSON.stringify(['development', 'minikube']),
    created_at: d(14),
    last_accessed: d(1),
  },
]

// ─── Namespaces ───────────────────────────────────────────────────────────────

export const namespaces = ['all', 'default', 'kube-system', 'monitoring', 'production', 'ingress-nginx']

// ─── Nodes ────────────────────────────────────────────────────────────────────

export const nodes = [
  {
    id: 'node-cp-1',
    name: 'gke-acme-prod-control-abc12',
    status: 'Ready',
    roles: 'control-plane',
    version: 'v1.28.4',
    age: age(47, 'd'),
    internal_ip: '10.0.0.1',
    os_image: 'Ubuntu 22.04.3 LTS',
    kernel_version: '5.15.0-1046-gke',
    container_runtime: 'containerd://1.7.2',
    taints: ['node-role.kubernetes.io/control-plane:NoSchedule'],
    capacity_cpu: '4',
    capacity_memory: '16Gi',
    capacity_pods: '110',
    allocatable_cpu: '3920m',
    allocatable_memory: '14Gi',
    allocatable_pods: '110',
    labels: { 'kubernetes.io/role': 'master', 'node.kubernetes.io/instance-type': 'e2-standard-4' },
    created_at: d(47),
  },
  {
    id: 'node-wk-1',
    name: 'gke-acme-prod-pool-1-def34',
    status: 'Ready',
    roles: 'worker',
    version: 'v1.28.4',
    age: age(47, 'd'),
    internal_ip: '10.0.0.2',
    os_image: 'Ubuntu 22.04.3 LTS',
    kernel_version: '5.15.0-1046-gke',
    container_runtime: 'containerd://1.7.2',
    taints: [],
    capacity_cpu: '8',
    capacity_memory: '32Gi',
    capacity_pods: '110',
    allocatable_cpu: '7910m',
    allocatable_memory: '30Gi',
    allocatable_pods: '110',
    labels: { 'node.kubernetes.io/instance-type': 'e2-standard-8' },
    created_at: d(47),
  },
  {
    id: 'node-wk-2',
    name: 'gke-acme-prod-pool-1-ghi56',
    status: 'Ready',
    roles: 'worker',
    version: 'v1.28.4',
    age: age(12, 'd'),
    internal_ip: '10.0.0.3',
    os_image: 'Ubuntu 22.04.3 LTS',
    kernel_version: '5.15.0-1046-gke',
    container_runtime: 'containerd://1.7.2',
    taints: [],
    capacity_cpu: '8',
    capacity_memory: '32Gi',
    capacity_pods: '110',
    allocatable_cpu: '7910m',
    allocatable_memory: '30Gi',
    allocatable_pods: '110',
    labels: { 'node.kubernetes.io/instance-type': 'e2-standard-8' },
    created_at: d(12),
  },
]

// ─── Pods ─────────────────────────────────────────────────────────────────────

export const pods = [
  {
    name: 'api-server-7d4b8c9f6-xkj2q',
    namespace: 'production',
    status: 'Running',
    age: age(2, 'd'),
    containers: 2,
    restarts: 0,
    node: 'gke-acme-prod-pool-1-def34',
    qos: 'Burstable',
    controlled_by: 'ReplicaSet/api-server-7d4b8c9f6',
    creation_timestamp: new Date(d(2)).toISOString(),
    labels: { app: 'api-server', version: 'v2.1.0', env: 'production' },
    annotations: { 'prometheus.io/scrape': 'true', 'prometheus.io/port': '9090' },
    pod_ip: '10.8.0.5',
    host_ip: '10.0.0.2',
    service_account: 'api-server-sa',
    priority_class: 'high-priority',
    container_details: [
      {
        name: 'api',
        image: 'acme/api-server:v2.1.0',
        ports: [{ name: 'http', containerPort: 8080, protocol: 'TCP' }],
        env_vars: [
          { name: 'LOG_LEVEL', value: 'info' },
          { name: 'DB_HOST', value_from: 'secretKeyRef' },
        ],
        resource_requests: { cpu: '100m', memory: '256Mi' },
        resource_limits: { cpu: '500m', memory: '512Mi' },
        liveness_probe: 'httpGet :8080/healthz',
        readiness_probe: 'httpGet :8080/ready',
        volume_mounts: [{ name: 'config', mountPath: '/etc/config' }],
        ready: true,
        restart_count: 0,
      },
      {
        name: 'sidecar-proxy',
        image: 'envoyproxy/envoy:v1.27.0',
        ports: [{ name: 'admin', containerPort: 9901, protocol: 'TCP' }],
        env_vars: [],
        resource_requests: { cpu: '50m', memory: '64Mi' },
        resource_limits: { cpu: '200m', memory: '128Mi' },
        liveness_probe: null,
        readiness_probe: null,
        volume_mounts: [],
        ready: true,
        restart_count: 0,
      },
    ],
    volumes: [
      { name: 'config', type: 'ConfigMap', source: 'api-server-config' },
      { name: 'credentials', type: 'Secret', source: 'api-credentials' },
    ],
    conditions: [
      { type: 'PodScheduled', status: 'True', message: '' },
      { type: 'Initialized', status: 'True', message: '' },
      { type: 'ContainersReady', status: 'True', message: '' },
      { type: 'Ready', status: 'True', message: '' },
    ],
  },
  {
    name: 'frontend-6b5c7d8e9-abc12',
    namespace: 'production',
    status: 'Running',
    age: age(5, 'd'),
    containers: 1,
    restarts: 1,
    node: 'gke-acme-prod-pool-1-ghi56',
    qos: 'Guaranteed',
    controlled_by: 'ReplicaSet/frontend-6b5c7d8e9',
    creation_timestamp: new Date(d(5)).toISOString(),
    labels: { app: 'frontend', version: 'v3.0.2', env: 'production' },
    annotations: {},
    pod_ip: '10.8.0.6',
    host_ip: '10.0.0.3',
    service_account: 'default',
    priority_class: '',
    container_details: [
      {
        name: 'nginx',
        image: 'acme/frontend:v3.0.2',
        ports: [{ name: 'http', containerPort: 80, protocol: 'TCP' }],
        env_vars: [{ name: 'NODE_ENV', value: 'production' }],
        resource_requests: { cpu: '50m', memory: '128Mi' },
        resource_limits: { cpu: '250m', memory: '256Mi' },
        liveness_probe: 'httpGet :80/',
        readiness_probe: 'httpGet :80/health',
        volume_mounts: [],
        ready: true,
        restart_count: 1,
      },
    ],
    volumes: [],
    conditions: [
      { type: 'PodScheduled', status: 'True', message: '' },
      { type: 'Ready', status: 'True', message: '' },
    ],
  },
  {
    name: 'worker-5f6g7h8i9-xyz99',
    namespace: 'production',
    status: 'Running',
    age: age(1, 'd'),
    containers: 1,
    restarts: 0,
    node: 'gke-acme-prod-pool-1-def34',
    qos: 'BestEffort',
    controlled_by: 'Deployment/worker',
    creation_timestamp: new Date(d(1)).toISOString(),
    labels: { app: 'worker', type: 'background', env: 'production' },
    annotations: {},
    pod_ip: '10.8.0.7',
    host_ip: '10.0.0.2',
    service_account: 'worker-sa',
    priority_class: '',
    container_details: [
      {
        name: 'worker',
        image: 'acme/worker:v1.5.0',
        ports: [],
        env_vars: [{ name: 'QUEUE_URL', value_from: 'configMapKeyRef' }],
        resource_requests: {},
        resource_limits: {},
        liveness_probe: 'exec /bin/sh -c "pgrep worker"',
        readiness_probe: null,
        volume_mounts: [],
        ready: true,
        restart_count: 0,
      },
    ],
    volumes: [],
    conditions: [{ type: 'Ready', status: 'True', message: '' }],
  },
  {
    name: 'prometheus-0',
    namespace: 'monitoring',
    status: 'Running',
    age: age(30, 'd'),
    containers: 1,
    restarts: 0,
    node: 'gke-acme-prod-pool-1-def34',
    qos: 'Guaranteed',
    controlled_by: 'StatefulSet/prometheus',
    creation_timestamp: new Date(d(30)).toISOString(),
    labels: { app: 'prometheus', component: 'monitoring' },
    annotations: { 'prometheus.io/scrape': 'false' },
    pod_ip: '10.8.0.8',
    host_ip: '10.0.0.2',
    service_account: 'prometheus-sa',
    priority_class: '',
    container_details: [],
    volumes: [],
    conditions: [{ type: 'Ready', status: 'True', message: '' }],
  },
  {
    name: 'fluentd-ds-k9x2m',
    namespace: 'kube-system',
    status: 'Running',
    age: age(47, 'd'),
    containers: 1,
    restarts: 0,
    node: 'gke-acme-prod-pool-1-ghi56',
    qos: 'Guaranteed',
    controlled_by: 'DaemonSet/fluentd',
    creation_timestamp: new Date(d(47)).toISOString(),
    labels: { app: 'fluentd', 'k8s-app': 'fluentd-logging' },
    annotations: {},
    pod_ip: '10.8.0.9',
    host_ip: '10.0.0.3',
    service_account: 'fluentd-sa',
    priority_class: 'system-node-critical',
    container_details: [],
    volumes: [],
    conditions: [{ type: 'Ready', status: 'True', message: '' }],
  },
]

// ─── Pod Events ───────────────────────────────────────────────────────────────

export const podEvents = [
  {
    id: 'pe-1',
    type: 'Normal',
    reason: 'Scheduled',
    message: 'Successfully assigned production/api-server-7d4b8c9f6-xkj2q to gke-acme-prod-pool-1-def34',
    age: age(2, 'd'),
    count: 1,
  },
  {
    id: 'pe-2',
    type: 'Normal',
    reason: 'Pulled',
    message: 'Container image "acme/api-server:v2.1.0" already present on machine',
    age: age(2, 'd'),
    count: 1,
  },
  {
    id: 'pe-3',
    type: 'Normal',
    reason: 'Started',
    message: 'Started container api',
    age: age(2, 'd'),
    count: 1,
  },
]

// ─── Deployments ──────────────────────────────────────────────────────────────

export const deployments = [
  {
    id: 'dep-1',
    name: 'api-server',
    namespace: 'production',
    status: 'Available',
    age: age(14, 'd'),
    images: ['acme/api-server:v2.1.0'],
    replicas: 3,
    available_replicas: 3,
    ready_replicas: 3,
    created_at: d(14),
    labels: { app: 'api-server', tier: 'backend' },
  },
  {
    id: 'dep-2',
    name: 'frontend',
    namespace: 'production',
    status: 'Available',
    age: age(7, 'd'),
    images: ['acme/frontend:v3.0.2'],
    replicas: 2,
    available_replicas: 2,
    ready_replicas: 2,
    created_at: d(7),
    labels: { app: 'frontend', tier: 'frontend' },
  },
  {
    id: 'dep-3',
    name: 'worker',
    namespace: 'production',
    status: 'Available',
    age: age(30, 'd'),
    images: ['acme/worker:v1.5.0'],
    replicas: 5,
    available_replicas: 5,
    ready_replicas: 5,
    created_at: d(30),
    labels: { app: 'worker', tier: 'backend' },
  },
  {
    id: 'dep-4',
    name: 'prometheus',
    namespace: 'monitoring',
    status: 'Available',
    age: age(60, 'd'),
    images: ['prom/prometheus:v2.47.0'],
    replicas: 1,
    available_replicas: 1,
    ready_replicas: 1,
    created_at: d(60),
    labels: { app: 'prometheus', component: 'monitoring' },
  },
  {
    id: 'dep-5',
    name: 'grafana',
    namespace: 'monitoring',
    status: 'Available',
    age: age(60, 'd'),
    images: ['grafana/grafana:10.1.0'],
    replicas: 1,
    available_replicas: 1,
    ready_replicas: 1,
    created_at: d(60),
    labels: { app: 'grafana', component: 'monitoring' },
  },
]

// ─── Cluster Metrics ──────────────────────────────────────────────────────────

export const clusterMetrics = {
  cpu: { capacity: 20000, allocatable: 19730, requests: 8500, limits: 15000, usage: 6200 },
  memory: { capacity: 81920, allocatable: 75776, requests: 35840, limits: 61440, usage: 28672 },
  pods: { capacity: 330, allocatable: 330, requests: 0, limits: 0, usage: 42 },
}

export const warningEvents = [
  {
    message: 'Readiness probe failed: Get "http://10.8.0.6:80/health": context deadline exceeded',
    object: 'Pod/frontend-6b5c7d8e9-abc12',
    type_: 'Warning',
    age: age(3, 'h'),
    count: 2,
  },
  {
    message: 'Failed to garbage collect required amount of images. Attempted to free 3221225472 bytes, but only achieved 0 bytes free',
    object: 'Node/gke-acme-prod-pool-1-ghi56',
    type_: 'Warning',
    age: age(45, 'm'),
    count: 5,
  },
]

// ─── Cluster Events ───────────────────────────────────────────────────────────

export const clusterEvents = [
  { id: 'ev-1', name: 'api-server-scaled', namespace: 'production', age: age(2, 'h'), status: 'Normal', event_type: 'Normal', reason: 'ScalingReplicaSet', message: 'Scaled up replica set api-server-7d4b8c9f6 to 3 from 2', object: 'Deployment/api-server', count: 1, created_at: h(2) },
  { id: 'ev-2', name: 'worker-scheduled', namespace: 'production', age: age(5, 'm'), status: 'Normal', event_type: 'Normal', reason: 'Scheduled', message: 'Successfully assigned production/worker-5f6g7h8i9-xyz99 to gke-acme-prod-pool-1-def34', object: 'Pod/worker-5f6g7h8i9-xyz99', count: 1, created_at: m(5) },
  { id: 'ev-3', name: 'frontend-probe-fail', namespace: 'production', age: age(3, 'h'), status: 'Warning', event_type: 'Warning', reason: 'Unhealthy', message: 'Readiness probe failed: context deadline exceeded', object: 'Pod/frontend-6b5c7d8e9-abc12', count: 2, created_at: h(3) },
  { id: 'ev-4', name: 'node-image-gc', namespace: 'kube-system', age: age(45, 'm'), status: 'Warning', event_type: 'Warning', reason: 'FreeDiskSpaceFailed', message: 'Failed to garbage collect required amount of images', object: 'Node/gke-acme-prod-pool-1-ghi56', count: 5, created_at: m(45) },
  { id: 'ev-5', name: 'grafana-pulled', namespace: 'monitoring', age: age(60, 'd'), status: 'Normal', event_type: 'Normal', reason: 'Pulled', message: 'Container image "grafana/grafana:10.1.0" already present', object: 'Pod/grafana-abc-def', count: 1, created_at: d(60) },
]

// ─── Generic resource helper ──────────────────────────────────────────────────

function res(name: string, ns: string, daysOld: number, extra?: Record<string, unknown>) {
  return {
    id: `${ns}-${name}`,
    name,
    namespace: ns,
    status: 'Active',
    age: age(daysOld, 'd'),
    images: [] as string[],
    created_at: d(daysOld),
    labels: { app: name } as Record<string, string>,
    ...extra,
  }
}

// ─── Services ─────────────────────────────────────────────────────────────────

export const services = [
  res('api-server', 'production', 14, { type: 'ClusterIP', cluster_ip: '10.96.1.100', external_ip: '', ports: '80/TCP,9090/TCP' }),
  res('frontend', 'production', 7, { type: 'LoadBalancer', cluster_ip: '10.96.1.101', external_ip: '34.12.34.56', ports: '80/TCP,443/TCP' }),
  res('worker-headless', 'production', 30, { type: 'ClusterIP', cluster_ip: 'None', external_ip: '', ports: '' }),
  res('prometheus', 'monitoring', 60, { type: 'ClusterIP', cluster_ip: '10.96.2.10', external_ip: '', ports: '9090/TCP' }),
  res('grafana', 'monitoring', 60, { type: 'NodePort', cluster_ip: '10.96.2.11', external_ip: '', ports: '3000:30000/TCP' }),
]

// ─── ConfigMaps ───────────────────────────────────────────────────────────────

export const configMaps = [
  res('api-server-config', 'production', 14),
  res('nginx-config', 'production', 7),
  res('worker-config', 'production', 30),
  res('prometheus-config', 'monitoring', 60),
  res('kube-dns', 'kube-system', 90, { labels: { 'k8s-app': 'kube-dns' } }),
]

// ─── Secrets ──────────────────────────────────────────────────────────────────

export const secrets = [
  res('api-credentials', 'production', 14, { type: 'Opaque' }),
  res('tls-secret', 'production', 7, { type: 'kubernetes.io/tls' }),
  res('registry-pull-secret', 'production', 90, { type: 'kubernetes.io/dockerconfigjson' }),
  res('monitoring-token', 'monitoring', 60, { type: 'kubernetes.io/service-account-token' }),
]

// ─── Namespaces (detailed) ────────────────────────────────────────────────────

export const namespacesDetailed = [
  res('default', 'default', 90, { status: 'Active' }),
  res('kube-system', 'kube-system', 90, { status: 'Active' }),
  res('monitoring', 'monitoring', 60, { status: 'Active' }),
  res('production', 'production', 30, { status: 'Active' }),
  res('ingress-nginx', 'ingress-nginx', 60, { status: 'Active' }),
]

// ─── StatefulSets ─────────────────────────────────────────────────────────────

export const statefulsets = [
  res('prometheus', 'monitoring', 30, { images: ['prom/prometheus:v2.47.0'], replicas: 1, ready_replicas: 1 }),
  res('alertmanager', 'monitoring', 30, { images: ['prom/alertmanager:v0.26.0'], replicas: 1, ready_replicas: 1 }),
  res('postgres', 'production', 14, { images: ['postgres:16'], replicas: 1, ready_replicas: 1 }),
]

// ─── DaemonSets ───────────────────────────────────────────────────────────────

export const daemonsets = [
  res('fluentd', 'kube-system', 47, { images: ['fluent/fluentd:v1.16'], desired: 3, ready: 3 }),
  res('node-exporter', 'monitoring', 60, { images: ['prom/node-exporter:v1.6.1'], desired: 3, ready: 3 }),
  res('kube-proxy', 'kube-system', 90, { images: ['registry.k8s.io/kube-proxy:v1.28.4'], desired: 3, ready: 3 }),
]

// ─── ReplicaSets ──────────────────────────────────────────────────────────────

export const replicasets = [
  res('api-server-7d4b8c9f6', 'production', 2, { images: ['acme/api-server:v2.1.0'], replicas: 3, ready_replicas: 3, owned_by: 'Deployment/api-server' }),
  res('frontend-6b5c7d8e9', 'production', 5, { images: ['acme/frontend:v3.0.2'], replicas: 2, ready_replicas: 2, owned_by: 'Deployment/frontend' }),
  res('worker-abc123def', 'production', 30, { images: ['acme/worker:v1.5.0'], replicas: 5, ready_replicas: 5, owned_by: 'Deployment/worker' }),
  res('grafana-xyz789', 'monitoring', 60, { images: ['grafana/grafana:10.1.0'], replicas: 1, ready_replicas: 1, owned_by: 'Deployment/grafana' }),
]

// ─── Jobs ─────────────────────────────────────────────────────────────────────

export const jobs = [
  res('db-migration-v2-1-0', 'production', 3, { status: 'Complete', completions: '1/1', duration: '45s', images: ['acme/migrate:v2.1.0'] }),
  res('db-seed-fixtures', 'default', 7, { status: 'Complete', completions: '1/1', duration: '12s', images: ['acme/migrate:v2.0.0'] }),
  res('backup-postgres-20260227', 'production', 1, { status: 'Complete', completions: '1/1', duration: '3m', images: ['acme/backup:v1.0'] }),
]

// ─── CronJobs ─────────────────────────────────────────────────────────────────

export const cronjobs = [
  res('backup-postgres', 'production', 30, { schedule: '0 2 * * *', last_schedule: age(22, 'h'), status: 'Active' }),
  res('cleanup-old-logs', 'production', 14, { schedule: '0 0 * * 0', last_schedule: age(3, 'd'), status: 'Active' }),
  res('send-digest-email', 'default', 7, { schedule: '0 8 * * 1', last_schedule: age(6, 'd'), status: 'Active' }),
]

// ─── Ingresses ────────────────────────────────────────────────────────────────

export const ingresses = [
  res('api-ingress', 'production', 14, { class: 'nginx', rules: 'api.acme.example.com → api-server:80', tls: 'tls-secret' }),
  res('frontend-ingress', 'production', 7, { class: 'nginx', rules: 'acme.example.com → frontend:80', tls: 'tls-secret' }),
  res('grafana-ingress', 'monitoring', 60, { class: 'nginx', rules: 'grafana.internal.example.com → grafana:3000', tls: '' }),
]

// ─── Endpoints ────────────────────────────────────────────────────────────────

export const endpoints = [
  res('api-server', 'production', 14, { subsets: '10.8.0.5:8080,10.8.0.6:8080,10.8.0.7:8080' }),
  res('frontend', 'production', 7, { subsets: '10.8.0.6:80' }),
  res('prometheus', 'monitoring', 60, { subsets: '10.8.0.8:9090' }),
  res('kubernetes', 'default', 90, { subsets: '10.0.0.1:6443' }),
]

// ─── Network Policies ────────────────────────────────────────────────────────

export const networkPolicies = [
  res('deny-all-ingress', 'production', 14, { policy_types: 'Ingress', pod_selector: '{}' }),
  res('allow-api-server', 'production', 14, { policy_types: 'Ingress,Egress', pod_selector: 'app=api-server' }),
  res('allow-monitoring', 'monitoring', 60, { policy_types: 'Ingress', pod_selector: 'app=prometheus' }),
]

// ─── PVCs ─────────────────────────────────────────────────────────────────────

export const pvcs = [
  res('postgres-data', 'production', 14, { status: 'Bound', volume: 'pvc-abc123', capacity: '20Gi', access_modes: 'ReadWriteOnce', storage_class: 'standard' }),
  res('prometheus-storage', 'monitoring', 60, { status: 'Bound', volume: 'pvc-def456', capacity: '50Gi', access_modes: 'ReadWriteOnce', storage_class: 'standard' }),
  res('grafana-storage', 'monitoring', 60, { status: 'Bound', volume: 'pvc-ghi789', capacity: '5Gi', access_modes: 'ReadWriteOnce', storage_class: 'standard' }),
]

// ─── PVs ──────────────────────────────────────────────────────────────────────

export const pvs = [
  { ...res('pvc-abc123', 'default', 14), namespace: '', status: 'Bound', capacity: '20Gi', access_modes: 'ReadWriteOnce', reclaim_policy: 'Delete', storage_class: 'standard', claim: 'production/postgres-data' },
  { ...res('pvc-def456', 'default', 60), namespace: '', status: 'Bound', capacity: '50Gi', access_modes: 'ReadWriteOnce', reclaim_policy: 'Delete', storage_class: 'standard', claim: 'monitoring/prometheus-storage' },
  { ...res('pvc-ghi789', 'default', 60), namespace: '', status: 'Bound', capacity: '5Gi', access_modes: 'ReadWriteOnce', reclaim_policy: 'Delete', storage_class: 'standard', claim: 'monitoring/grafana-storage' },
]

// ─── Storage Classes ─────────────────────────────────────────────────────────

export const storageClasses = [
  { ...res('standard', 'default', 90), namespace: '', provisioner: 'pd.csi.storage.gke.io', reclaim_policy: 'Delete', volume_binding: 'WaitForFirstConsumer', default: true },
  { ...res('premium-rwo', 'default', 60), namespace: '', provisioner: 'pd.csi.storage.gke.io', reclaim_policy: 'Delete', volume_binding: 'WaitForFirstConsumer', default: false },
]

// ─── Role Bindings ────────────────────────────────────────────────────────────

export const roleBindings = [
  res('api-server-binding', 'production', 14, { role_ref: 'ClusterRole/api-server-role', subjects: 'ServiceAccount/api-server-sa' }),
  res('developer-binding', 'production', 30, { role_ref: 'Role/developer', subjects: 'User/developer@acme.example.com' }),
  res('monitoring-binding', 'monitoring', 60, { role_ref: 'ClusterRole/view', subjects: 'ServiceAccount/prometheus-sa' }),
]

// ─── Roles ────────────────────────────────────────────────────────────────────

export const roles = [
  res('developer', 'production', 30, { rules: '["pods","deployments","services"]' }),
  res('pod-reader', 'production', 14, { rules: '["pods"]' }),
  res('monitoring-reader', 'monitoring', 60, { rules: '["pods","nodes","services"]' }),
]

// ─── Cluster Roles ────────────────────────────────────────────────────────────

export const clusterRoles = [
  { ...res('api-server-role', 'default', 14), namespace: '', rules: '["pods","deployments","services","configmaps"]' },
  { ...res('cluster-reader', 'default', 30), namespace: '', rules: '["*/*"]' },
  { ...res('node-exporter-role', 'default', 60), namespace: '', rules: '["nodes","nodes/metrics"]' },
]

// ─── Cluster Role Bindings ────────────────────────────────────────────────────

export const clusterRoleBindings = [
  { ...res('cluster-admin-binding', 'default', 90), namespace: '', role_ref: 'ClusterRole/cluster-admin', subjects: 'User/admin@acme.example.com' },
  { ...res('node-exporter-binding', 'default', 60), namespace: '', role_ref: 'ClusterRole/node-exporter-role', subjects: 'ServiceAccount/node-exporter' },
]

// ─── Service Accounts ─────────────────────────────────────────────────────────

export const serviceAccounts = [
  res('api-server-sa', 'production', 14, { secrets: 1 }),
  res('worker-sa', 'production', 30, { secrets: 1 }),
  res('default', 'production', 30, { secrets: 0 }),
  res('prometheus-sa', 'monitoring', 60, { secrets: 1 }),
  res('default', 'default', 90, { secrets: 0 }),
]

// ─── HPA ──────────────────────────────────────────────────────────────────────

export const hpas = [
  res('api-server-hpa', 'production', 14, { min_replicas: 2, max_replicas: 10, current_replicas: 3, target: 'Deployment/api-server', metrics: 'cpu: 70%' }),
  res('worker-hpa', 'production', 30, { min_replicas: 3, max_replicas: 20, current_replicas: 5, target: 'Deployment/worker', metrics: 'cpu: 60%' }),
]

// ─── Resource Quotas ─────────────────────────────────────────────────────────

export const resourceQuotas = [
  res('production-quota', 'production', 14, { hard_cpu: '20', used_cpu: '8.5', hard_memory: '40Gi', used_memory: '18Gi', hard_pods: '50', used_pods: '8' }),
  res('monitoring-quota', 'monitoring', 60, { hard_cpu: '8', used_cpu: '2', hard_memory: '16Gi', used_memory: '6Gi', hard_pods: '20', used_pods: '4' }),
]

// ─── Limit Ranges ────────────────────────────────────────────────────────────

export const limitRanges = [
  res('default-limits', 'production', 14, { default_cpu: '250m', default_memory: '256Mi', default_req_cpu: '50m', default_req_memory: '64Mi' }),
  res('monitoring-limits', 'monitoring', 60, { default_cpu: '500m', default_memory: '512Mi', default_req_cpu: '100m', default_req_memory: '128Mi' }),
]

// ─── Pod Disruption Budgets ───────────────────────────────────────────────────

export const pdbs = [
  res('api-server-pdb', 'production', 14, { min_available: '2', current_healthy: 3, desired_healthy: 3, disruptions_allowed: 1, selector: 'app=api-server' }),
  res('worker-pdb', 'production', 30, { min_available: '3', current_healthy: 5, desired_healthy: 5, disruptions_allowed: 2, selector: 'app=worker' }),
]

// ─── Helm Releases ────────────────────────────────────────────────────────────

export const helmReleases = [
  { id: 'hr-1', name: 'ingress-nginx', namespace: 'ingress-nginx', status: 'deployed', revision: '5', chart: 'ingress-nginx', app_version: '1.9.4', updated: age(7, 'd') },
  { id: 'hr-2', name: 'cert-manager', namespace: 'cert-manager', status: 'deployed', revision: '3', chart: 'cert-manager', app_version: 'v1.13.2', updated: age(14, 'd') },
  { id: 'hr-3', name: 'prometheus-stack', namespace: 'monitoring', status: 'deployed', revision: '8', chart: 'kube-prometheus-stack', app_version: '0.68.0', updated: age(30, 'd') },
  { id: 'hr-4', name: 'postgres', namespace: 'production', status: 'deployed', revision: '2', chart: 'postgresql', app_version: '16.1.0', updated: age(14, 'd') },
]

// ─── Helm Charts ─────────────────────────────────────────────────────────────

export const helmCharts = [
  { id: 'hc-1', name: 'ingress-nginx', chart: 'ingress-nginx/ingress-nginx', version: '4.8.3', app_version: '1.9.4', description: 'Ingress controller using NGINX as reverse proxy', status: 'local' },
  { id: 'hc-2', name: 'cert-manager', chart: 'cert-manager/cert-manager', version: '1.13.2', app_version: 'v1.13.2', description: 'Certificate management for Kubernetes', status: 'local' },
  { id: 'hc-3', name: 'kube-prometheus-stack', chart: 'prometheus-community/kube-prometheus-stack', version: '54.2.2', app_version: '0.68.0', description: 'Prometheus monitoring stack', status: 'local' },
]

// ─── CRDs ─────────────────────────────────────────────────────────────────────

export const crds = [
  { ...res('certificates.cert-manager.io', 'default', 14), namespace: '', group: 'cert-manager.io', scope: 'Namespaced', versions: 'v1' },
  { ...res('ingressclasses.networking.k8s.io', 'default', 90), namespace: '', group: 'networking.k8s.io', scope: 'Cluster', versions: 'v1' },
  { ...res('prometheuses.monitoring.coreos.com', 'default', 60), namespace: '', group: 'monitoring.coreos.com', scope: 'Namespaced', versions: 'v1' },
  { ...res('servicemonitors.monitoring.coreos.com', 'default', 60), namespace: '', group: 'monitoring.coreos.com', scope: 'Namespaced', versions: 'v1' },
]

// ─── Resource YAML ───────────────────────────────────────────────────────────

export const resourceYaml = `apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
  namespace: production
  labels:
    app: api-server
    tier: backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-server
  template:
    metadata:
      labels:
        app: api-server
        version: v2.1.0
    spec:
      serviceAccountName: api-server-sa
      containers:
        - name: api
          image: acme/api-server:v2.1.0
          ports:
            - name: http
              containerPort: 8080
          resources:
            requests:
              cpu: 100m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
`

// ─── Helm availability ────────────────────────────────────────────────────────

export const helmAvailability = {
  available: true,
  version: 'v3.13.2',
  message: null,
}
