<div align="center">
  <img src="https://github.com/dest4590/CollapseLoader/assets/80628386/190926bf-cde4-4de4-a35f-476eb9d9ac7b" width="100">
  <h1>CollapseLoader</h1>

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/dest4590/CollapseLoader/build.yml?style=for-the-badge&logo=githubactions&logoColor=ffffff&color=%2382B155)](https://github.com/dest4590/CollapseLoader/actions)
[![GitHub Release](https://img.shields.io/github/v/release/dest4590/CollapseLoader?display_name=tag&style=for-the-badge&logo=alwaysdata&logoColor=ffffff)](https://github.com/dest4590/CollapseLoader/releases/latest)
[![GitHub Pre-release](https://img.shields.io/github/v/release/dest4590/CollapseLoader?include_prereleases&display_name=tag&style=for-the-badge&logo=buffer&label=Prerelease)](https://github.com/dest4590/CollapseLoader/releases)

**A secure, open-source launcher for verified Minecraft cheat clients.**

[Website](https://collapseloader.org) • [Discord](https://collapseloader.org/discord) • [Releases](https://github.com/dest4590/CollapseLoader/releases)

</div>

---

## 📑 Table of Contents

- [About CollapseLoader](#about-collapseloader)
- [Key Features](#key-features)
- [Safety & Transparency](#safety--transparency)
- [Screenshots](#screenshots)
- [Installation](#installation)
    - [Windows](#windows)
    - [Linux](#linux)
    - [Linux Troubleshooting](#linux-troubleshooting)
- [Development](#development)

---

## About CollapseLoader

CollapseLoader is a modern, cross-platform tool built with **Rust** and **Tauri**. It provides a safe environment for launching Minecraft cheat clients on Windows and Linux. Our mission is to eliminate the risks associated with third-party software by providing a transparent, community-reviewed platform.

### Key Features

- **Strict Verification**: Only clients that pass rigorous security audits are supported.
- **Cross-Platform**: Seamless performance on both Windows and Linux.
- **Virus Protection**: Integration with [CollapseScanner](https://github.com/CollapseLoader/CollapseScanner) and industry-standard antivirus tools.
- **Zero Obfuscation**: We only host clients with readable, reviewable code.
- **Automated Trust**: All binaries are built via GitHub Actions to ensure the code you see is the code you run.

---

## Safety & Transparency

We take security seriously. Unlike traditional loaders, CollapseLoader is built on the principle of "Trust through Verification":

- **Public Repositories**: Every component is open-source under the [CollapseLoader Organization](https://github.com/CollapseLoader).
- **Security Audits**: We check for malicious network activity, token loggers, and file system intrusions.
- **Security Policy**: For a deep dive into our protocols, check our [SECURITY.md](SECURITY.md).

---

## Screenshots

<div align="center">
  <img src="docs/screenshots/home.png" width="400">
    <img src="docs/screenshots/friends.png" width="400">
    <img src="docs/screenshots/settings.png" width="400">
    <img src="docs/screenshots/customization.png" width="400">
    <img src="docs/screenshots/marketplace.png" width="400">
    <img src="docs/screenshots/account.png" width="400">
    <img src="docs/screenshots/about.png" width="400">
</div>

---

## Installation

### Windows

- **System**: Windows 10 or 11.
- **Download**: Grab the `.msi` installer or the standalone `.exe` from [Releases](https://github.com/dest4590/CollapseLoader/releases).

### Linux

We provide several ways to install CollapseLoader depending on your distribution:

#### 1. Arch Linux (AUR)

Arch users can install the package using an AUR helper like `yay` or `paru`:

```bash
# To install the pre-compiled binary version
yay -S collapseloader-bin

# To build from the latest git source
yay -S collapseloader-git
```

#### 2. Debian / Ubuntu

Download the `.deb` package from the releases page and install it via:

```bash
sudo dpkg -i collapseloader_amd64.deb
sudo apt install -f # Fix potential missing dependencies
```

#### 3. Generic (AppImage)

Download the `.AppImage` file, make it executable, and run:

```bash
chmod +x CollapseLoader.AppImage
./CollapseLoader.AppImage
```

For a full list of supported clients on linux, [view our compatibility table](https://docs.google.com/spreadsheets/d/1TcMOdRKTSeGfns5WijZjvZBz6HBdwXZlClN0pcPd14k/edit?usp=sharing).

---

## Linux Troubleshooting

### 1. Missing Dependencies (webkit2)

If the application (especially the Binary or AppImage) fails to start, you likely need to install the `webkit2gtk` package for your system:

**Debian / Ubuntu / Mint:**

```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**Arch Linux / Manjaro:**

```bash
sudo pacman -S webkit2gtk-4.1
```

**Fedora / RHEL:**

```bash
sudo dnf install webkit2gtk4.1-devel
```

### 2. Rendering Issues (Fix)

If you experience a blank screen, flickering, or UI crashes (common on NVIDIA drivers or specific Wayland setups), use the following environment variables to launch the app:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 GDK_BACKEND=x11 ./CollapseLoader
```

---

## Development

To build CollapseLoader from source, ensure you have the Rust toolchain and Node.js installed.

1. Clone the repository: `git clone https://github.com/dest4590/CollapseLoader`
2. Install dependencies: `npm install`
3. Run in dev mode: `npm run tauri dev`
4. Build: `npm run tauri build`
