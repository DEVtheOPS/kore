---
title: Installation
description: Download and install Kore on macOS, Linux, or Windows.
sidebar:
  order: 1
---

Kore ships as a self-contained native binary built with [Tauri v2](https://tauri.app). There is no Electron runtime and no Node.js required — the installer is under 20 MB.

## Requirements

- A reachable Kubernetes cluster (local or remote)
- A valid `kubeconfig` file (usually at `~/.kube/config`)
- macOS 11+, Ubuntu 20.04+, or Windows 10+

---

## macOS

### Homebrew (recommended)

```sh
brew install --cask devtheops/tap/kore
```

Homebrew handles updates automatically. Run `brew upgrade --cask kore` to update.

### Manual download

1. Open the [latest GitHub Release](https://github.com/DEVtheOPS/kore/releases/latest)
2. Download the `.dmg` for your architecture:
   - **Apple Silicon** → `Kore_*_aarch64.dmg`
   - **Intel** → `Kore_*_x86_64.dmg`
   - **Universal** → `Kore_*_universal.dmg`
3. Open the `.dmg`, drag Kore to **Applications**, and launch.

:::note
macOS may show a security prompt the first time. Go to **System Settings → Privacy & Security** and click **Open Anyway**.
:::

---

## Linux

Download the appropriate package from the [latest release](https://github.com/DEVtheOPS/kore/releases/latest):

| Format | Architecture | Use when |
|--------|-------------|---------|
| `.AppImage` | `x86_64` | Any modern distro — no install needed |
| `.deb` | `x86_64` | Debian / Ubuntu |
| `.rpm` | `x86_64` | Fedora / RHEL |

**AppImage (no install):**

```sh
chmod +x Kore_*.AppImage
./Kore_*.AppImage
```

**Debian / Ubuntu:**

```sh
sudo dpkg -i kore_*.deb
```

---

## Windows

1. Download `Kore_*_x64_en-US.msi` from the [latest release](https://github.com/DEVtheOPS/kore/releases/latest)
2. Run the installer and follow the prompts
3. Launch **Kore** from the Start menu

---

## Updating

Kore includes a built-in auto-updater. When a new version is available you will see a notification in the app. Click **Update** to download and install in the background.

Alternatively, re-run the Homebrew command or download the latest installer manually.
