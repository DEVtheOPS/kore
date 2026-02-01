# Kore E2E Tests

End-to-end tests for Kore using Playwright.

## Test Structure

- `01-layout-navigation.spec.ts` - Tests for main layout, navigation, and theming
- `02-icon-sidebar.spec.ts` - Tests for the icon sidebar functionality
- `03-cluster-overview.spec.ts` - Tests for the cluster overview page
- `04-cluster-import.spec.ts` - Tests for the cluster import modal
- `05-settings.spec.ts` - Tests for the settings page
- `06-cluster-routes.spec.ts` - Tests for cluster route structure

## Running Tests

### Run all tests
```bash
pnpm test
```

### Run tests with UI
```bash
pnpm test:ui
```

### Run tests in debug mode
```bash
pnpm test:debug
```

### Run specific test file
```bash
pnpm test tests/01-layout-navigation.spec.ts
```

### Run in headed mode (see browser)
```bash
pnpm test --headed
```

## Test Coverage

The test suite covers:

### ✅ Layout & Navigation
- Icon sidebar visibility and structure
- Navigation between pages (overview, settings)
- Theme application and persistence
- Responsive layout structure

### ✅ Icon Sidebar
- Active state indicators
- Add cluster button
- Import modal opening/closing
- Bookmarked clusters display
- Section dividers

### ✅ Cluster Overview
- Page structure and content
- Empty state when no clusters
- DataTable display with columns
- Search and refresh functionality
- Column configuration

### ✅ Cluster Import
- Modal structure and tabs
- File import tab
- Folder import tab
- Modal closing (backdrop, button)
- Tab switching behavior

### ✅ Settings
- Page structure
- Appearance settings card
- Theme selector
- Theme persistence
- Navigation

### ✅ Cluster Routes
- Route structure validation
- Graceful handling of non-existent clusters
- URL preservation
- Sidebar structure in cluster views

## CI/CD Integration

Tests are configured to run with 2 retries in CI environments and generate HTML reports.

## Requirements

- Node.js 18+
- Chromium browser (installed automatically by Playwright)
- Dev server running on http://localhost:1420

## Notes

- Tests run against the dev server (automatically started by Playwright)
- Some tests verify UI structure without actual cluster data
- Tests are designed to be resilient and handle both empty and populated states
