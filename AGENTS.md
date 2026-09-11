# Agent Instructions

## Project Context: Kore

**Kore** (Kubernetes Orchestration and Resource Explorer) is a high-performance Kubernetes IDE built as an OpenLens alternative. It leverages Tauri v2 for the backend (Rust) and Svelte 5 (Runes) for the frontend.

### Tech Stack

- **Frontend**: Svelte 5 (Runes based reactivity), Tailwind CSS v4, Lucide Icons.
- **Backend**: Rust (Tauri v2), `kube` crate, `k8s-openapi`.
- **State Management**: Svelte 5 `$state` and `$derived` in `.svelte.ts` store files (e.g., `cluster.svelte.ts`, `settings.svelte.ts`).
- **Styling**: Semantic CSS variables mapped to Tailwind `@theme` (see `src/routes/layout.css`).

### Key Architectural Patterns

1. **Stores**: Centralized logic in `src/lib/stores/`. Use `class` based stores with `$state` fields.
   - `clustersStore`: Manages all clusters with SQLite persistence
   - `activeClusterStore`: Tracks currently selected cluster and namespaces
   - `bookmarksStore`: Manages cluster bookmarks for icon sidebar
   - `settingsStore`: App-level settings (theme, etc.)
2. **Routing Structure**:
   - `/` - Cluster overview (management page)
   - `/cluster/[id]/*` - Cluster-scoped views (all resources)
   - `/cluster/[id]/settings` - Cluster-specific settings
   - `/settings` - App-level settings
3. **Theming**: Do not hardcode colors. Use semantic variables (`--bg-main`, `--text-muted`, `--color-primary`, etc.).
4. **Kubernetes Interactions**:
    - **Commands**: Simple actions (list, delete) use `#[tauri::command]`.
    - **Streaming**: Resource watching uses Tauri Events (`start_pod_watch` -> `window.emit`).
    - **Cluster Management**: SQLite database at `~/.kore/clusters.db` stores cluster metadata.
    - **Config Storage**: Each cluster's kubeconfig stored at `~/.kore/kubeconfigs/<uuid>.yaml`.
    - **UUID-based**: Clusters identified by UUID v4 for stable routing.

### Process environment (PATH)

Kubeconfigs commonly authenticate through `exec` credential plugins (`aws eks
get-token`, `gke-gcloud-auth-plugin`, `kubelogin`), and the YAML/scale/Helm
features shell out to `kubectl` and `helm`. A GUI app launched from Finder/Dock or
a desktop launcher gets a minimal `PATH` (`/usr/bin:/bin:/usr/sbin:/sbin` on
macOS), so those binaries are not found even though they work in a terminal.
`lib.rs` calls `fix_path_env::fix()` first thing in `run()` to import `PATH` from
the user's login shell. Do not spawn processes before that call, and keep the
`kubectl`/`helm` invocations as plain command names so they resolve via `PATH`.

Surface backend errors verbatim in the UI (`Failed to load X: ${e}`) — a generic
message hid this exact problem for a long time.

### Component Library (`src/lib/components/ui/`)

- **DataTable**: Powerful table with sorting, filtering, column visibility, drag-and-drop, and batch actions.
- **Select/Input/Button/Badge**: Reusable primitives matching the design system.
- **Drawer**: Right-side panel for details.
- **Menu**: Dropdowns for row actions.
- **IconSidebar**: Left-most sidebar with cluster bookmarks and app navigation.
- **ResourceSidebar**: Cluster-specific navigation for resource types.
- **ClusterImportModal**: Import clusters from files or folders with context extraction.

---

## Workflow Instructions

### Tracking work

Open work is listed in [`docs/ROADMAP.md`](docs/ROADMAP.md); use GitHub Issues for
anything that needs discussion. Do not leave TODO comments in code — add an entry to
the roadmap or open an issue instead.

### Landing the Plane (Session Completion)

**When ending a work session**, complete ALL steps below. Work is NOT complete until
`git push` succeeds.

1. **Record remaining work** — update `docs/ROADMAP.md` (or open GitHub Issues) for
   anything that needs follow-up.
2. **Run quality gates** (if code changed):
   - `bun run check`, `bun run test:unit --run`, `bun run test`
   - `cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
3. **Use Conventional Commits** (`feat:`, `fix:`, `chore:`, ...) — release-please
   generates `CHANGELOG.md`, version bumps, and GitHub releases from them. Never
   edit `CHANGELOG.md` or version fields by hand.
4. **PUSH TO REMOTE** — this is MANDATORY:

   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```

5. **Clean up** — clear stashes, prune remote branches.
6. **Hand off** — provide context for the next session.

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing — that leaves work stranded locally
- If push fails, resolve and retry until it succeeds
