import { describe, expect, it } from 'vitest';
import {
  cpuToMillicores,
  formatBytes,
  formatMillicores,
  isMetricsUnavailableError,
  parseQuantity,
  percentOf,
} from './quantity';

describe('parseQuantity', () => {
  it('parses plain numbers and exponents', () => {
    expect(parseQuantity('2')).toBe(2);
    expect(parseQuantity('0.5')).toBe(0.5);
    expect(parseQuantity('1e3')).toBe(1000);
  });

  it('parses decimal suffixes', () => {
    expect(parseQuantity('250m')).toBeCloseTo(0.25);
    expect(parseQuantity('123456789n')).toBeCloseTo(0.123456789);
    expect(parseQuantity('2k')).toBe(2000);
    expect(parseQuantity('1E')).toBe(1e18);
  });

  it('parses binary suffixes', () => {
    expect(parseQuantity('1Ki')).toBe(1024);
    expect(parseQuantity('512Mi')).toBe(512 * 1024 ** 2);
    expect(parseQuantity('2Gi')).toBe(2 * 1024 ** 3);
  });

  it('returns null for invalid input', () => {
    expect(parseQuantity('')).toBeNull();
    expect(parseQuantity(null)).toBeNull();
    expect(parseQuantity('abc')).toBeNull();
    expect(parseQuantity('Mi')).toBeNull();
  });
});

describe('formatting', () => {
  it('converts cpu to millicores', () => {
    expect(cpuToMillicores('250m')).toBeCloseTo(250);
    expect(cpuToMillicores('2')).toBe(2000);
    expect(cpuToMillicores('bad')).toBeNull();
  });

  it('formats millicores', () => {
    expect(formatMillicores(250)).toBe('250m');
    expect(formatMillicores(1500)).toBe('1.50');
    expect(formatMillicores(null)).toBe('-');
  });

  it('formats bytes', () => {
    expect(formatBytes(512)).toBe('512B');
    expect(formatBytes(2048)).toBe('2.0Ki');
    expect(formatBytes(1.5 * 1024 ** 3)).toBe('1.5Gi');
    expect(formatBytes(200 * 1024 ** 2)).toBe('200Mi');
    expect(formatBytes(undefined)).toBe('-');
  });

  it('computes clamped percentages', () => {
    expect(percentOf(50, 200)).toBe(25);
    expect(percentOf(500, 200)).toBe(100);
    expect(percentOf(10, 0)).toBeNull();
    expect(percentOf(null, 10)).toBeNull();
  });

  it('detects metrics unavailable errors', () => {
    expect(isMetricsUnavailableError('Metrics API not available (node usage ...)')).toBe(true);
    expect(isMetricsUnavailableError(new Error('boom'))).toBe(false);
  });
});
