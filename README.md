# Kore

**Kubernetes Orchestration and Resource Explorer** - A lightweight, open-source Kubernetes IDE built with Tauri v2 and Svelte 5.

![Kore](https://raw.githubusercontent.com/tauri-apps/tauri/dev/.github/splash.png) <!-- Placeholder for actual screenshot -->

## Features

- **🚀 Blazing Fast**: Built on Rust and Tauri, consuming a fraction of the RAM of Electron-based competitors.
- **🎨 Theming System**:
  - **Kore** (Default - Kubernetes Blue)
  - **Kore Light**
  - **Dracula**
  - **Alucard** (Light Dracula)
  - **Rusty** & **Rusty Light** (Legacy)
- **☸️ Multi-Cluster Management**:
  - Import kubeconfigs from files or folders with automatic context extraction.
  - Each cluster stored independently with UUID-based routing.
  - SQLite database for cluster metadata (name, icon, description, tags).
  - Bookmark favorite clusters in the icon sidebar for quick access.
  - Drag-and-drop to reorder bookmarks.
- **⚡ Real-time Updates**: Kubernetes resources update in real-time using efficient watch streams.
- **📊 Advanced Data Tables**:
  - Sorting, Filtering, and Column Reordering.
  - Multi-selection and Batch Actions (e.g., Bulk Delete).
  - Persistent user preferences for column visibility.
- **🛠️ Workload Management**: View, Edit, Log, Shell, and Delete Pods (more resources coming soon).

## Tech Stack

- **Frontend Framework**: [Svelte 5](https://svelte.dev/) (Runes)
- **Desktop Framework**: [Tauri v2](https://v2.tauri.app/)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/)
- **Kubernetes Client**: `kube-rs` & `k8s-openapi`
- **Icons**: `lucide-svelte`

## Project Structure

```text
├── src/                         # Svelte Frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/              # Reusable UI components
│   │   │   ├── IconSidebar.svelte    # Left-most navigation
│   │   │   ├── ResourceSidebar.svelte # Cluster resource navigation
│   │   │   └── ClusterImportModal.svelte
│   │   └── stores/
│   │       ├── clusters.svelte.ts     # Cluster CRUD operations
│   │       ├── activeCluster.svelte.ts # Current cluster state
│   │       ├── bookmarks.svelte.ts    # Sidebar bookmarks
│   │       └── settings.svelte.ts     # App settings
│   ├── routes/
│   │   ├── +page.svelte              # Cluster overview
│   │   ├── cluster/[id]/             # Cluster-scoped routes
│   │   │   ├── pods/
│   │   │   ├── deployments/
│   │   │   ├── settings/             # Cluster settings
│   │   │   └── ...
│   │   └── settings/                 # App settings
│   └── ...
├── src-tauri/            # Rust Backend
│   ├── src/
│   │   ├── cluster_manager.rs # SQLite cluster storage
│   │   ├── import.rs          # Kubeconfig import & extraction
│   │   ├── k8s.rs             # Kubernetes API & Watchers
│   │   └── ...
│   └── ...
```

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/) (v1.3+)
- Docker (optional, for local k8s testing)

### Setup

1. **Install dependencies**:
2.
   ```bash
   bun install
   cd src-tauri && cargo fetch
   ```

3. **Run Development Server**:

   ```bash
   bun run tauri dev
   ```

### Building for Production

```bash
bun run tauri build
```

## CI/CD & Releases

Kore uses GitHub Actions for automated testing and releases.

### Workflows

- **`test.yml`**: Runs on PRs and pushes to `main`. Performs linting, type checking, and tests on all platforms.
- **`release-please.yml`**: Runs on every push to `main`. Maintains a release PR from [Conventional Commits](https://www.conventionalcommits.org/); merging it creates the GitHub release, builds signed installers for every platform, publishes the release, and updates the auto-updater manifest on GitHub Pages.
- **`deploy-docs.yml`** / **`screenshots.yml`**: Build the docs site and app screenshots for GitHub Pages.
- **`security.yml`**: Dependency and lint security scans.

### Setting Up Auto-Updates

The in-app updater checks `https://devtheops.github.io/kore/update.json` (falling back to the `latest.json` asset on the latest GitHub Release) and verifies downloads against the public key below. To set up signing for a fork:

1. **Generate signing keys**:

   ```bash
   bun run tauri signer generate -w ~/.tauri/kore.key

   ```
   This creates a private key (`kore.key`) and outputs the public key.

2. **Update the public key** in `src-tauri/tauri.conf.json`:

   ```json
   "plugins": {
     "updater": {
       "pubkey": "YOUR_PUBLIC_KEY_HERE",
       ...
     }
   }
   ```

3. **Add GitHub secrets** (Settings → Secrets → Actions):
   - `TAURI_SIGNING_PRIVATE_KEY`: Contents of `~/.tauri/kore.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: Your key password (if set)

4. **Set up the `gh-pages` branch** (orphan branch for the website):

   ```bash
   # Create orphan branch
   git checkout --orphan gh-pages
   git rm -rf .

   # Add your landing page (index.html, etc.)
   echo '<!DOCTYPE html><html>...</html>' > index.html
   git add index.html
   git commit -m "Initial gh-pages"
   git push origin gh-pages

   # Return to main
   git checkout main
   ```

   **Working on the site with a worktree**:

   ```bash
   git worktree add ../kore-pages gh-pages
   cd ../kore-pages
   # Edit site files, commit, push
   ```

5. **Enable GitHub Pages** (Settings → Pages):
   - Source: Deploy from a branch
   - Branch: `gh-pages` / `/ (root)`

### Creating a Release

Releases are fully automated with [release-please](https://github.com/googleapis/release-please):

1. Land changes on `main` using Conventional Commit messages (`feat:` → minor bump, `fix:` → patch bump; `feat!:` / `BREAKING CHANGE:` → minor while pre-1.0).
2. release-please opens (or updates) a **"chore(main): release x.y.z"** PR that bumps `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `docs/package.json`, `src-tauri/Cargo.lock`, and `CHANGELOG.md`.
3. Merge that PR. The workflow creates the `vX.Y.Z` release, builds and signs installers for all platforms, attaches them, and publishes the updater manifest (`latest.json` from tauri-action) to `gh-pages` as `update.json`.

If a build fails or a release had to be created by hand, re-run the **Release Please** workflow via *Run workflow* with `tag` set to the release tag (e.g. `v0.4.0`) to rebuild and re-attach the assets.

Never bump versions or edit `CHANGELOG.md` by hand.

### Running Tests & Coverage

**Frontend (Svelte/TS)**

```bash
# Run Unit Tests
bun run test:unit

# Run Unit Tests with Coverage
bun run test:coverage

# Run Playwright E2E Tests
bun run test
```

**Backend (Rust)**

```bash
# Run Unit Tests
cd src-tauri
cargo test

# Run Coverage (requires cargo-llvm-cov)
# Install: cargo install cargo-llvm-cov
cargo llvm-cov
```

See [tests/README.md](tests/README.md) for more details.

## Configuration

Kore stores its configuration in:

- **macOS/Linux**: `~/.kore/`
- **Windows**: `C:\Users\<User>\.kore\`

Storage structure:

```text
~/.kore/
├── clusters.db              # SQLite database (cluster metadata)
├── kubeconfigs/             # Extracted single-context configs
│   ├── <uuid-1>.yaml
│   ├── <uuid-2>.yaml
│   └── ...
└── bookmarks.json           # Sidebar bookmarks
```

If you used a pre-rebrand build that stored data in `~/.rustylens`, Kore migrates that directory to `~/.kore` (and rewrites the stored kubeconfig paths) automatically on first launch.

### Live usage metrics

The Nodes page and the Deployment / StatefulSet detail panels show live CPU and memory usage when [metrics-server](https://github.com/kubernetes-sigs/metrics-server) is installed in the cluster. Without it, Kore shows a notice and everything else keeps working.
