# YCG Installation Guide

This guide explains how to install the YCG CLI tool on your system.

## Quick Install (Recommended)

### Prerequisites
- Rust 1.75+ and Cargo installed
- macOS, Linux, or WSL on Windows

### Installation Steps

1. **Clone and build the project:**
   ```bash
   git clone https://github.com/yourusername/ycg.git
   cd ycg
   cargo build --release
   ```

2. **Run the installation script:**
   ```bash
   sudo ./install.sh
   ```

3. **Verify installation:**
   ```bash
   ycg --help
   ```

The binary will be installed as `ycg` in `/usr/local/bin/`.

## Manual Installation

If you prefer to install manually:

```bash
# Build the release binary
cargo build --release

# Copy to a directory in your PATH
sudo cp target/release/ycg_cli /usr/local/bin/ycg

# Make it executable
sudo chmod +x /usr/local/bin/ycg
```

## Alternative: Install via Cargo

You can also install directly using Cargo:

```bash
cargo install --path crates/ycg_cli
```

This will install the binary to `~/.cargo/bin/ycg_cli`. Make sure `~/.cargo/bin` is in your PATH.

## Usage

Once installed, you can use `ycg` from any directory:

```bash
# Basic usage
ycg -i index.scip -o graph.yaml

# With compact mode (recommended)
ycg -i index.scip -o graph.yaml --compact

# High detail mode
ycg -i index.scip -o graph.yaml --lod 2

# Specify project root
ycg -i /path/to/index.scip -o output.yaml --root /path/to/project
```

## Uninstallation

To remove YCG from your system:

```bash
sudo rm /usr/local/bin/ycg
```

Or if installed via Cargo:

```bash
cargo uninstall ycg_cli
```

## Troubleshooting

### Command not found

If you get "command not found" after installation:

1. Check if `/usr/local/bin` is in your PATH:
   ```bash
   echo $PATH
   ```

2. If not, add it to your shell configuration:
   ```bash
   # For zsh (macOS default)
   echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   
   # For bash
   echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

### Permission denied

If you get permission errors during installation:

- Make sure you're using `sudo` for system-wide installation
- Or use the Cargo install method which installs to your user directory

### Binary not found during installation

If the install script can't find the binary:

```bash
cargo build --release
```

Make sure the build completes successfully before running the install script.

## Platform-Specific Notes

### macOS
- `/usr/local/bin` is the standard location for user-installed binaries
- Already in PATH by default

### Linux
- `/usr/local/bin` is standard on most distributions
- May need to add to PATH on some minimal installations

### Windows (WSL)
- Follow the Linux instructions within WSL
- Native Windows support coming soon

## Development Installation

For development, you can run the CLI without installing:

```bash
cargo run --release --bin ycg_cli -- -i index.scip -o output.yaml
```

Or create an alias:

```bash
alias ycg='cargo run --release --bin ycg_cli --'
```
