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
- **`release.yml`**: Triggered on tag pushes (`v*`). Builds signed binaries for all platforms and creates a GitHub release.
- **`pages.yml`**: Deploys the landing page and update manifest to GitHub Pages.

### Setting Up Auto-Updates

The Tauri updater requires signed binaries. To set this up:

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

1. Update the version in `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`
2. Commit the changes
3. Create and push a tag:

   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

The release workflow will automatically build all platforms and publish to GitHub Releases and Pages.

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
