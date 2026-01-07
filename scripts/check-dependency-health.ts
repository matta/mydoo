/**
 * Dependency Health Report Generator
 *
 * This script scans the workspace for all unique dependencies and fetches
 * popularity and maintenance metrics from the npm registry.
 *
 * Usage from project root:
 *   pnpm exec tsx scripts/check-dependency-health.ts
 *
 * Requirements:
 *   - Internet access to hit registry.npmjs.org and api.npmjs.org
 *   - Node.js environment with pnpm installed
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { glob } from 'glob';

// ANSI color codes
const reset = '\x1b[0m';
const bold = '\x1b[1m';
const red = '\x1b[31m';
const green = '\x1b[32m';
const yellow = '\x1b[33m';
const gray = '\x1b[90m';

const NPM_REGISTRY = 'https://registry.npmjs.org';
const NPM_DOWNLOADS = 'https://api.npmjs.org/downloads/point/last-week';

// Retry configuration
const MAX_RETRIES = 20;
const INITIAL_DELAY_MS = 1000;
const MIN_DELAY_MS = 50;
const COOLDOWN_ITERATIONS = 5;
const SUCCESS_STEP = 0.1; // Decrease delay by 10% on success
const BACKOFF_MULTIPLIER = 2; // Double delay on failure

// Health status thresholds
const DAYS_AGING_THRESHOLD = 365;
const DAYS_STALE_THRESHOLD = 365 * 2;
const LOW_DOWNLOADS_THRESHOLD = 10000;
const VERY_LOW_DOWNLOADS_THRESHOLD = 1000;

interface PackageStats {
  name: string;
  latestVersion?: string;
  lastPublish?: Date;
  weeklyDownloads?: number;
  isDeprecated?: boolean;
  deprecatedMessage?: string;
  error?: string;
}

/** Returns colored maintenance status based on days since publish */
function getMaintenanceStatus(
  daysSincePublish: number,
  isDeprecated?: boolean,
): string {
  if (isDeprecated) return `${red}DEPRECATED${reset}`;
  if (daysSincePublish > DAYS_STALE_THRESHOLD)
    return `${red}STALE (>2y)${reset}`;
  if (daysSincePublish > DAYS_AGING_THRESHOLD)
    return `${yellow}AGING (>1y)${reset}`;
  return `${green}ACTIVE${reset}`;
}

/** Returns colored download count based on popularity thresholds */
function formatDownloads(downloads: number): string {
  const display = downloads.toLocaleString();
  if (downloads < VERY_LOW_DOWNLOADS_THRESHOLD)
    return `${red}${display}${reset}`;
  if (downloads < LOW_DOWNLOADS_THRESHOLD) return `${yellow}${display}${reset}`;
  return display;
}

/** Safely parse JSON, returning null on failure */
function safeJSON<T>(text: string): T | null {
  try {
    return JSON.parse(text) as T;
  } catch {
    return null;
  }
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/** Add ±25% random jitter to avoid thundering herd */
function jitter(ms: number): number {
  return Math.round(ms * (1 + 0.25 * (Math.random() * 2 - 1)));
}

/**
 * Adaptive rate limiter using AIMD (Additive Increase, Multiplicative Decrease).
 * Speeds up on success, backs off on throttle/error.
 */
class AdaptiveRateLimiter {
  private currentDelay = INITIAL_DELAY_MS;
  private cooldown = 0;

  async fetchWithBackoff(url: string): Promise<Response> {
    let hadFailure = false;

    for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
      // Wait: use currentDelay for first attempt, exponential backoff for retries (with jitter)
      const baseWait =
        attempt === 0 ? this.currentDelay : this.currentDelay * 2 ** attempt;
      if (baseWait > 0) {
        await sleep(jitter(baseWait));
      }

      try {
        const res = await fetch(url);

        // Handle throttling (429) or server overload (5xx)
        if (res.status === 429 || res.status >= 500) {
          hadFailure = true;
          continue;
        }

        // Success: maybe speed up, but first apply any failure penalty
        if (hadFailure) {
          this.handleThrottle();
        } else if (res.ok) {
          this.optimizeRate();
        }

        return res;
      } catch {
        // Network error: mark failure and retry
        hadFailure = true;
      }
    }

    // All retries exhausted - apply penalty for next call
    this.handleThrottle();
    throw new Error(`Request to ${url} failed after ${MAX_RETRIES} retries`);
  }

  private handleThrottle() {
    // Multiplicative decrease (at most 2x per fetchWithBackoff call)
    this.currentDelay = Math.max(this.currentDelay * BACKOFF_MULTIPLIER, 10);

    // Set cooldown to prevent immediate re-acceleration
    this.cooldown = COOLDOWN_ITERATIONS;
  }

  private optimizeRate() {
    if (this.cooldown > 0) {
      this.cooldown--;
      return;
    }

    // Additive increase (reduce delay asymptotically toward MIN_DELAY_MS)
    if (this.currentDelay > MIN_DELAY_MS) {
      this.currentDelay = Math.max(
        MIN_DELAY_MS,
        this.currentDelay * (1 - SUCCESS_STEP),
      );
    }
  }

  get delayMs(): number {
    return Math.round(this.currentDelay);
  }
}

const limiter = new AdaptiveRateLimiter();

async function fetchPackageStats(pkgName: string): Promise<PackageStats> {
  try {
    const registryRes = await limiter.fetchWithBackoff(
      `${NPM_REGISTRY}/${pkgName}`,
    );

    if (!registryRes.ok) {
      if (registryRes.status === 404) {
        return { name: pkgName, error: 'Not Found (404)' };
      }
      return { name: pkgName, error: `Registry Error: ${registryRes.status}` };
    }

    const downloadsRes = await limiter.fetchWithBackoff(
      `${NPM_DOWNLOADS}/${pkgName}`,
    );

    const registryData = await registryRes.json();

    const downloadsData = downloadsRes.ok
      ? await downloadsRes.json()
      : { downloads: 0 };

    const latestVersion = registryData['dist-tags']?.latest;
    const time = registryData.time;
    const lastPublish = time ? new Date(time[latestVersion]) : new Date(0);

    // Check deprecation (can be on the package or the specific version)
    const versionData = registryData.versions?.[latestVersion];
    const isDeprecated = !!versionData?.deprecated;

    return {
      name: pkgName,
      latestVersion,
      lastPublish,
      weeklyDownloads: downloadsData.downloads || 0,
      isDeprecated,
      deprecatedMessage: versionData?.deprecated,
    };
  } catch (_error: unknown) {
    const message = _error instanceof Error ? _error.message : 'Unknown Error';
    return { name: pkgName, error: message };
  }
}

async function main() {
  console.log(`${bold}🔍 Scanning workspace for dependencies...${reset}`);

  // Find all package.json files, ignoring node_modules
  const scriptDir = path.dirname(new URL(import.meta.url).pathname);
  const rootDir = path.resolve(scriptDir, '..');

  const packageFiles = await glob('**/package.json', {
    ignore: ['**/node_modules/**', '**/dist/**', '**/coverage/**'],
    cwd: rootDir,
    absolute: true,
  });

  const dependencies = new Set<string>();

  for (const file of packageFiles) {
    const content = await fs.readFile(file, 'utf-8');
    const json = safeJSON<{
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    }>(content);
    if (!json) {
      console.warn(`Warning: Could not parse ${file}`);
      continue;
    }
    for (const d of Object.keys(json.dependencies ?? {})) dependencies.add(d);
    for (const d of Object.keys(json.devDependencies ?? {}))
      dependencies.add(d);
  }

  const pkgList = Array.from(dependencies).sort();
  const total = pkgList.length;

  console.log(`Found ${bold}${total}${reset} unique dependencies.`);

  // Fetch stats with simple progress output
  const stats: PackageStats[] = [];
  for (let i = 0; i < total; i++) {
    const pkg = pkgList[i];
    const start = Date.now();
    const result = await fetchPackageStats(pkg);
    const ms = Date.now() - start;
    process.stdout.write(`\r${i + 1}/${total} ${pkg} (${ms}ms)`.padEnd(60));
    stats.push(result);
  }
  console.log(`\r${'✓ Fetched all packages'.padEnd(60)}`);

  stats.sort((a, b) => (a.weeklyDownloads || 0) - (b.weeklyDownloads || 0));

  // Print table header
  console.log(`\n${bold}Dependency Health Report${reset}`);
  console.log(`${gray}Sorted by weekly downloads (ascending)${reset}\n`);

  const colWidths = [42, 14, 14, 14, 20];
  const headers = [
    'PACKAGE',
    'VERSION',
    'LAST PUBLISH',
    'DOWNLOADS/WK',
    'STATUS',
  ];
  console.log(headers.map((h, i) => h.padEnd(colWidths[i])).join(''));
  console.log('-'.repeat(colWidths.reduce((a, b) => a + b, 0)));

  for (const pkg of stats) {
    if (pkg.error) {
      console.log(
        pkg.name.padEnd(colWidths[0]) +
          '-'.padEnd(colWidths[1]) +
          '-'.padEnd(colWidths[2]) +
          '-'.padEnd(colWidths[3]) +
          `${red}ERROR: ${pkg.error}${reset}`,
      );
      continue;
    }

    const daysSincePublish = pkg.lastPublish
      ? (Date.now() - pkg.lastPublish.getTime()) / (1000 * 3600 * 24)
      : 0;

    console.log(
      pkg.name.padEnd(colWidths[0]) +
        (pkg.latestVersion || 'N/A').padEnd(colWidths[1]) +
        (pkg.lastPublish
          ? pkg.lastPublish.toISOString().split('T')[0]
          : 'N/A'
        ).padEnd(colWidths[2]) +
        formatDownloads(pkg.weeklyDownloads || 0).padEnd(colWidths[3]) +
        getMaintenanceStatus(daysSincePublish, pkg.isDeprecated),
    );
  }
}

main().catch(console.error);
