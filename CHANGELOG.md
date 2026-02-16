# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-16

### Added

#### Sidebar Sections & Navigation
- **Complete sidebar implementation** - All placeholder sections now have functional pages with real data
  - Nodes page with detailed node information and status
  - Workloads Overview with aggregated health summaries
  - Namespaces page with namespace management
  - Events page with filtering and real-time updates
  - RoleBindings and ClusterRoleBindings pages
  - Helm Releases and Charts pages with capability detection
  - CRDs page with schema details

#### YAML Editing & Mutation Workflows
- **YAML Editor with Syntax Highlighting** - Professional code editor for all resources
  - CodeMirror 6 integration (~150-250KB bundle increase)
  - Full YAML syntax highlighting with theme-aware colors
  - Line numbers, auto-indentation, bracket matching
  - Search/replace (Cmd/Ctrl+F), undo/redo support
  - Read-only mode for static displays

- **Edit YAML Functionality** - Edit Kubernetes resources directly from info panels
  - Edit button in Deployment detail drawer
  - Edit button in all WorkloadList resource detail panels
  - Apply changes with confirmation dialog
  - Automatic refresh after successful apply

- **Workload Mutation Operations** - Day-2 operations for Deployments, StatefulSets, DaemonSets
  - Scale workload replicas
  - Restart rollout
  - Backend commands: `cluster_scale_workload`, `cluster_restart_workload`

- **Syntax Highlighting for Static YAML** - Beautiful highlighting in annotations and configs
  - `YamlDisplay` component for read-only YAML
  - Theme-aware colors matching app theme
  - Automatic highlighting for JSON annotations converted to YAML

#### Theming & Customization
- **Independent Code Theme Setting** - Separate theme for code blocks
  - New setting: "Code Editor Theme" in Settings page
  - Options: "same-as-app" (default), or any of the 6 themes
  - Allows light app with dark code or vice versa
  - Persists across sessions

#### Log Streaming Improvements
- **Multiple Log Tabs Fix** - Each tab now maintains its own log subscription
  - Fixed bug where all tabs showed logs from the first tab
  - Each tab has independent subscription and log buffer
  - Logs continue streaming in background for all tabs
  - Proper cleanup when tabs are closed

- **Stream Cleanup & Resource Management** - Prevent memory leaks
  - Backend stream registry using `tokio::sync::broadcast`
  - New command: `stop_stream_logs` to cancel streams
  - Frontend calls cleanup on tab close
  - Automatic cleanup when streams end naturally

### Changed

#### UI/UX Improvements
- **Settings Page Layout** - Added proper padding (24px) to prevent content from bumping against sidebar
- **Removed Duplicate Settings Button** - Removed redundant "App Settings" from ResourceSidebar
- **Removed Edit Button from Pod Details** - Editing pods directly doesn't make sense (managed by controllers)

#### Code Quality
- **Chart.js Components Registration** - Fixed "linear scale not registered" error
  - Explicitly registered: LineController, LineElement, PointElement, LinearScale, CategoryScale, Title, Tooltip, Legend, Filler
  - Tree-shakeable architecture properly configured

### Technical Details

#### Backend (Rust/Tauri)
- Added `tokio` dependency with `sync` and `macros` features
- Stream management with broadcast channels for cancellation
- YAML get/apply commands: `cluster_get_resource_yaml`, `cluster_apply_resource_yaml`
- Workload operations: `cluster_scale_workload`, `cluster_restart_workload`
- Stream control: `stop_stream_logs`

#### Frontend (Svelte 5 + TypeScript)
- Added CodeMirror 6 packages:
  - `codemirror@6.0.2`
  - `@codemirror/lang-yaml@6.1.2`
  - `@codemirror/language@6.12.1`
  - `@lezer/highlight@1.2.3`
- New components:
  - `CodeEditor.svelte` - Editable YAML editor with syntax highlighting
  - `YamlDisplay.svelte` - Read-only syntax-highlighted YAML display
- Updated stores:
  - `settings.svelte.ts` - Added `codeTheme` setting with `effectiveCodeTheme` getter

#### Theme Support
All features support all 6 themes:
- Kore (dark)
- Kore Light
- Rusty (dark)
- Rusty Light
- Dracula
- Alucard (Dracula Light)

### Developer Experience
- All sidebar sections now data-backed (no placeholders)
- Consistent UX patterns across all resource types
- Standardized error handling with retry/dismiss options
- Professional code editing experience matching VS Code quality

---

## [0.1.1] - Previous Release

Initial release with basic functionality.
