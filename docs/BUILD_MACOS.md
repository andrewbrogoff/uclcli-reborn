# Building uclcli on macOS

## Prerequisites

### Install Xcode Command Line Tools

```bash
xcode-select --install
```

### Install Homebrew (if not already installed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation (need 1.85+)
rustc --version
```

## Installing libucl

### Option 1: Homebrew (Recommended)

```bash
# Install libucl from Homebrew
brew install ucl
```

### Option 2: Build from Source

```bash
# Download UCL source
curl -O https://www.oberhumer.com/opensource/ucl/download/ucl-1.03.tar.gz
tar xzf ucl-1.03.tar.gz
cd ucl-1.03

# Configure and build
./configure --prefix=/usr/local
make

# Install (requires admin privileges)
sudo make install
```

> **Note:** macOS Big Sur (11.0) and later automatically update the dynamic linker cache—no manual cache refresh is needed.

## Building uclcli

```bash
cd uclcli-reborn
cargo build --release

# Binaries are in:
ls -la target/release/ucl target/release/unucl
```

## Installation (Optional)

```bash
# Copy to user binaries
sudo cp target/release/ucl target/release/unucl /usr/local/bin/

# Or install via cargo
cargo install --path .
```

## Testing

```bash
# Compress and decompress
echo "Hello World" | ./target/release/ucl > test.nrv
./target/release/unucl < test.nrv
rm test.nrv

# Test with piping
echo "Testing UCL compression" | ./target/release/ucl | ./target/release/unucl
```

## Architecture-Specific Notes

### Apple Silicon (M1/M2/M3)

The project works natively on Apple Silicon. No special configuration needed.

```bash
# Check your architecture
uname -m
# arm64 = Apple Silicon
# x86_64 = Intel
```

### Intel Macs

Works with standard configuration. If you need to build for both architectures:

```bash
# Build universal binary
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

## Troubleshooting

### "unable to find library -lucl"

```bash
# Check if libucl is installed
ls -l /usr/local/lib/libucl.*
ls -l /opt/homebrew/lib/libucl.*  # Apple Silicon Homebrew path

# If missing, reinstall libucl
brew reinstall ucl

# Or check pkg-config
pkg-config --libs --cflags ucl
```

### Library not found at runtime

The recommended approach is to ensure libucl is installed in standard Homebrew paths and embed the correct runtime path into your binaries using `install_name_tool`.

#### Step 1: Install libucl to standard paths

If you installed via Homebrew, libucl should already be in the correct location:

```bash
# Apple Silicon (M1/M2/M3)
ls -l /opt/homebrew/lib/libucl.*

# Intel Macs
ls -l /usr/local/lib/libucl.*
```

If you built from source, ensure you installed to `/usr/local` (Intel) or `/opt/homebrew` (Apple Silicon):

```bash
# Rebuild and install to Homebrew prefix (Apple Silicon example)
cd ucl-1.03
./configure --prefix=/opt/homebrew
make clean && make
sudo make install
```

#### Step 2: Embed runtime library path using install_name_tool

After building, use `install_name_tool` to set the correct `@rpath` or absolute path in your binaries:

```bash
# Check current library dependencies
otool -L target/release/ucl
otool -L target/release/unucl

# For Apple Silicon (Homebrew at /opt/homebrew)
install_name_tool -change libucl.1.dylib /opt/homebrew/lib/libucl.1.dylib target/release/ucl
install_name_tool -change libucl.1.dylib /opt/homebrew/lib/libucl.1.dylib target/release/unucl

# For Intel Macs (Homebrew at /usr/local)
install_name_tool -change libucl.1.dylib /usr/local/lib/libucl.1.dylib target/release/ucl
install_name_tool -change libucl.1.dylib /usr/local/lib/libucl.1.dylib target/release/unucl

# Verify the change
otool -L target/release/ucl
```

Alternatively, you can use `@rpath` for more flexible deployment:

```bash
# Add rpath to the binary
install_name_tool -add_rpath /opt/homebrew/lib target/release/ucl
install_name_tool -add_rpath /opt/homebrew/lib target/release/unucl

# Or embed relative to executable (for bundled apps)
install_name_tool -add_rpath @executable_path/../lib target/release/ucl
```

> **⚠️ Caution: DYLD_LIBRARY_PATH (Last Resort Only)**
>
> `DYLD_LIBRARY_PATH` is **ignored by macOS System Integrity Protection (SIP)** for system binaries and should **not be used in production**. It exists only as a temporary development workaround.
>
> If you absolutely must use it for quick local testing:
> ```bash
> # Temporary session-only (Apple Silicon)
> export DYLD_LIBRARY_PATH=/opt/homebrew/lib:$DYLD_LIBRARY_PATH
> 
> # Temporary session-only (Intel)
> export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH
> ```
>
> **Optional:** Add to `~/.zshrc` for persistent dev environments (not recommended):
> ```bash
> echo 'export DYLD_LIBRARY_PATH=/opt/homebrew/lib:$DYLD_LIBRARY_PATH' >> ~/.zshrc
> ```
>
> **Important limitations:**
> - SIP-protected binaries (e.g., those in `/usr/bin`) ignore `DYLD_LIBRARY_PATH` entirely
> - This setting does not work for notarized/sandboxed apps
> - Always prefer `install_name_tool` or proper library installation paths

### "ucl init failed" at runtime

Wrong libucl version. This code expects UCL 1.03.

```bash
# Check installed version
brew info ucl

# Or if built from source:
ls -l /usr/local/lib/libucl.*
```

### Homebrew not found (Apple Silicon)

On Apple Silicon Macs, Homebrew installs to `/opt/homebrew`:

```bash
# Add Homebrew to PATH
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zshrc
source ~/.zshrc
```

### pkg-config not found

```bash
brew install pkg-config
```

## Building for Distribution

### Create a relocatable build

```bash
# Build with static linking (if available)
RUSTFLAGS='-C link-arg=-Wl,-rpath,@executable_path/../lib' cargo build --release

# Create an app bundle structure
mkdir -p dist/uclcli.app/Contents/{MacOS,lib}
cp target/release/{ucl,unucl} dist/uclcli.app/Contents/MacOS/
cp /usr/local/lib/libucl.* dist/uclcli.app/Contents/lib/
```

## Performance Notes

- Native Apple Silicon builds offer best performance on M1/M2/M3 Macs
- Both architectures support the full feature set
- Memory-mapped file operations work efficiently on all macOS versions
