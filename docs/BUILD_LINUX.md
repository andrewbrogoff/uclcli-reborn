# Building uclcli on Linux

## Prerequisites

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation (need 1.85+)
rustc --version
```

## Installing libucl

### Arch Linux

```bash
# Install from AUR (recommended)
yay -S ucl
# or with paru:
paru -S ucl

# Or build from AUR manually
git clone https://aur.archlinux.org/ucl.git
cd ucl
makepkg -si
```

### Debian/Ubuntu

```bash
sudo apt-get update
sudo apt-get install libucl-dev
```

### Fedora/RHEL

```bash
# Build from source (see below)
sudo dnf install gcc make
```

### From Source (Any Distro)

```bash
wget https://www.oberhumer.com/opensource/ucl/download/ucl-1.03.tar.gz
tar xzf ucl-1.03.tar.gz
cd ucl-1.03
./configure
make
sudo make install
sudo ldconfig
```

## Building

```bash
cd uclcli
cargo build --release

# Binaries are in:
ls -la target/release/ucl target/release/unucl
```

## Testing

```bash
# Compress and decompress
echo "Hello World" | ./target/release/ucl > test.nrv
./target/release/unucl < test.nrv
rm test.nrv
```

## Troubleshooting

### "unable to find library -lucl"

```bash
# Check if installed
ldconfig -p | grep ucl

# If missing, install and refresh cache
sudo ldconfig
```

### "ucl init failed" at runtime

Wrong libucl version. This code expects UCL 1.03.
