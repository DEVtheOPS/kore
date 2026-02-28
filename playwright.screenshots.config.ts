import { defineConfig, devices } from '@playwright/test'

/**
 * Playwright config specifically for screenshot capture.
 *
 * Differences from the main config:
 * - Points at tests/screenshots/ only
 * - Runs sequentially (fullyParallel: false) for predictable output
 * - Wider viewport (1440×900) to show sidebars + panels comfortably
 * - Uses the same Vite dev server as the main config
 */
export default defineConfig({
  testDir: './tests/screenshots',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [['html', { outputFolder: 'screenshot-report' }], ['list']],

  use: {
    baseURL: 'http://localhost:1420',
    viewport: { width: 1440, height: 900 },
    trace: 'off',
    // Keep screenshots even on pass (handled manually in tests, but just in case)
    screenshot: 'off',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  webServer: {
    // `pnpm dev` runs just Vite (not Tauri) – see package.json scripts
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: true,
    timeout: 120_000,
  },
})
