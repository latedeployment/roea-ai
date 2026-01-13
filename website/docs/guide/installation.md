# Installation

## Download

Download the latest release for your platform:

### macOS

#### Apple Silicon (M1/M2/M3)

```bash
# Download DMG
curl -LO https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_aarch64.dmg

# Or use Homebrew (recommended)
brew install --cask roea-ai
```

#### Intel Mac

```bash
# Download DMG
curl -LO https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_x64.dmg

# Or use Homebrew
brew install --cask roea-ai
```

### Windows

```powershell
# Download MSI installer
Invoke-WebRequest -Uri "https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_x64_en-US.msi" -OutFile "roea-ai.msi"

# Run installer
.\roea-ai.msi
```

Or download directly from the [releases page](https://github.com/your-org/roea-ai/releases).

### Linux

#### Debian/Ubuntu

```bash
# Download .deb package
curl -LO https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_amd64.deb

# Install
sudo dpkg -i roea-ai_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f
```

#### Other Distributions

```bash
# Download standalone binary
curl -LO https://github.com/your-org/roea-ai/releases/latest/download/roea-agent-linux-x64
chmod +x roea-agent-linux-x64

# Move to PATH
sudo mv roea-agent-linux-x64 /usr/local/bin/roea-agent
```

## Building from Source

If you prefer to build from source:

### Prerequisites

- **Rust** 1.75 or later
- **Node.js** 18 or later
- **protobuf-compiler** (for gRPC)

#### Install Build Dependencies

::: code-group

```bash [Ubuntu/Debian]
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    protobuf-compiler \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev
```

```bash [Fedora]
sudo dnf install -y \
    @development-tools \
    protobuf-compiler \
    webkit2gtk4.1-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

```bash [macOS]
brew install protobuf
```

```powershell [Windows]
choco install protoc
```

:::

### Build Steps

```bash
# Clone repository
git clone https://github.com/your-org/roea-ai.git
cd roea-ai

# Build Rust components
cargo build --release

# Build UI
cd crates/roea-ui
npm install
npm run tauri build
```

The built application will be in `target/release/` and `crates/roea-ui/src-tauri/target/release/bundle/`.

## Verify Installation

After installation, verify everything is working:

```bash
# Check agent daemon
roea-agent --version
# Expected: roea-agent 1.0.0

# Check daemon starts
roea-agent &
# Expected: INFO roea_agent: Starting roea-agent daemon...

# Check UI can connect (if installed)
roea-ui
```

## Post-Installation

### Linux: Enable eBPF Monitoring (Optional)

For high-performance kernel-level monitoring on Linux:

```bash
# Check if BTF is available
ls -la /sys/kernel/btf/vmlinux

# Generate vmlinux.h (if building from source)
bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h

# Grant BPF capability (instead of running as root)
sudo setcap cap_bpf+ep /usr/local/bin/roea-agent

# Or run with sudo
sudo roea-agent
```

See [Linux eBPF Setup](/reference/ebpf) for detailed instructions.

### macOS: Grant Permissions

On first launch, macOS will prompt for:
- **Full Disk Access**: Required for file monitoring
- **Network Monitoring**: Required for connection tracking

You can grant these in System Settings → Privacy & Security.

### Windows: Firewall Rules

Windows Defender may prompt about network access. Allow roea-agent to receive connections if you want to use the separate UI application.

## Upgrading

### Homebrew (macOS)

```bash
brew upgrade roea-ai
```

### APT (Debian/Ubuntu)

```bash
# Re-download latest .deb and install
curl -LO https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_amd64.deb
sudo dpkg -i roea-ai_amd64.deb
```

### Manual

Download the latest release and replace the existing binary.

## Uninstalling

### macOS

```bash
# Homebrew
brew uninstall --cask roea-ai

# Or manually
rm -rf /Applications/roea-ai.app
rm -rf ~/Library/Application\ Support/roea-ai
```

### Linux

```bash
# Debian/Ubuntu
sudo dpkg -r roea-ai

# Manual
sudo rm /usr/local/bin/roea-agent
rm -rf ~/.local/share/roea-ai
```

### Windows

Use "Add or Remove Programs" in Windows Settings, or run the MSI installer again and select "Remove".

## Next Steps

- [Quick Start Guide](/guide/quick-start) - Run your first monitoring session
- [System Requirements](/guide/requirements) - Detailed platform requirements
