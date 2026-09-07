/**
 * Helpers for Kubernetes resource quantities and live usage values.
 *
 * Mirrors the backend `usage.rs` parser so allocatable/capacity strings from
 * the core API can be compared against metrics-server usage values.
 */

const BINARY: Array<[string, number]> = [
  ['Ki', 1024],
  ['Mi', 1024 ** 2],
  ['Gi', 1024 ** 3],
  ['Ti', 1024 ** 4],
  ['Pi', 1024 ** 5],
  ['Ei', 1024 ** 6],
];

const DECIMAL: Array<[string, number]> = [
  ['n', 1e-9],
  ['u', 1e-6],
  ['m', 1e-3],
  ['k', 1e3],
  ['M', 1e6],
  ['G', 1e9],
  ['T', 1e12],
  ['P', 1e15],
];

/** Strict numeric parse: rejects empty strings (which `Number()` coerces to 0). */
function parseNumber(s: string): number | null {
  if (!s.trim()) return null;
  const n = Number(s);
  return Number.isFinite(n) ? n : null;
}

/** Parse a Kubernetes quantity ("500m", "2Gi", "1e3") into base units. Returns null when invalid. */
export function parseQuantity(q: string | null | undefined): number | null {
  if (q == null) return null;
  const s = q.trim();
  if (!s) return null;

  for (const [suffix, mult] of BINARY) {
    if (s.endsWith(suffix)) {
      const n = parseNumber(s.slice(0, -suffix.length));
      return n == null ? null : n * mult;
    }
  }
  for (const [suffix, mult] of DECIMAL) {
    if (s.endsWith(suffix)) {
      const n = parseNumber(s.slice(0, -suffix.length));
      return n == null ? null : n * mult;
    }
  }
  if (s.endsWith('E')) {
    const n = parseNumber(s.slice(0, -1));
    if (n != null) return n * 1e18;
  }
  return parseNumber(s);
}

/** CPU quantity -> millicores. */
export function cpuToMillicores(q: string | null | undefined): number | null {
  const cores = parseQuantity(q);
  return cores == null ? null : cores * 1000;
}

/** Format millicores for display: "250m" below one core, otherwise cores with 2 decimals. */
export function formatMillicores(mc: number | null | undefined): string {
  if (mc == null || !Number.isFinite(mc)) return '-';
  if (mc < 1000) return `${Math.round(mc)}m`;
  return `${(mc / 1000).toFixed(2)}`;
}

/** Format bytes using binary units (Ki/Mi/Gi/Ti). */
export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isFinite(bytes)) return '-';
  if (bytes < 1024) return `${Math.round(bytes)}B`;
  const units = ['Ki', 'Mi', 'Gi', 'Ti', 'Pi'];
  let value = bytes / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value >= 100 ? value.toFixed(0) : value.toFixed(1)}${units[i]}`;
}

/** Percentage of `used` over `total`, clamped to [0, 100]. Null when total is unknown or zero. */
export function percentOf(used: number | null | undefined, total: number | null | undefined): number | null {
  if (used == null || total == null || total <= 0) return null;
  return Math.max(0, Math.min(100, (used / total) * 100));
}

/** Derive an error message for a missing metrics API from a backend error. */
export function isMetricsUnavailableError(e: unknown): boolean {
  return String(e).includes('Metrics API not available');
}
