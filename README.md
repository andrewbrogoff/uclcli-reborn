# uclcli reborn - CLI for libucl (NRV compression)

Command-line compression and decompression utilities using the UCL/NRV compression algorithm.

> **Fork of [afflux/uclcli](https://github.com/afflux/uclcli)** - Enhanced and maintained by Andrew Rogoff

## Requirements

- **Rust 1.92+** (edition 2024)
- **libucl 1.03** - The UCL compression library

## Platform-Specific Build Guides

Choose your operating system for detailed build instructions:

| Platform | Build Guide | Package Manager |
|----------|-------------|-----------------|
| **Linux** | [BUILD_LINUX.md](docs/BUILD_LINUX.md) | apt, yay/paru, dnf, or source |
| **macOS** | [BUILD_MACOS.md](docs/BUILD_MACOS.md) | Homebrew or source |
| **Windows** | [BUILD_WINDOWS.md](docs/BUILD_WINDOWS.md) | MSYS2/MinGW or Visual Studio |

## Quick Start

### Linux (Arch)

```bash
# Install libucl
yay -S ucl

# Build
cargo build --release

# Test
echo "test" | ./target/release/ucl | ./target/release/unucl
```

### macOS

```bash
# Install libucl
brew install ucl

# Build
cargo build --release

# Test
echo "test" | ./target/release/ucl | ./target/release/unucl
```

### Windows (MSYS2)

```bash
# Install build tools and libucl (see BUILD_WINDOWS.md for details)
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-rust

# Build libucl from source (see BUILD_WINDOWS.md)
# Then build uclcli
cargo build --release

# Test
echo test | ./target/release/ucl.exe | ./target/release/unucl.exe
```

## Usage

### Compression (`ucl`)

```bash
ucl -i input.bin -o output.nrv    # File to file
ucl < input.bin > output.nrv      # Stdin to stdout
cat file | ucl > compressed.nrv   # Pipe-friendly
```

### Decompression (`unucl`)

```bash
unucl -i input.nrv -o output.bin           # File to file
unucl < compressed.nrv > decompressed.bin  # Stdin to stdout
unucl -b 1073741824 -i large.nrv -o out    # Custom buffer (1GB)
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input file (default: stdin) |
| `-o, --output <FILE>` | Output file (default: stdout) |
| `-b, --buffersize <SIZE>` | Decompression buffer size (unucl only, default: 512MB) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Fuzzing

Fuzzing tests are available for testing the robustness of the compression/decompression routines.

### Requirements

- **Rust nightly toolchain** - Required for fuzzing with sanitizers

```bash
rustup toolchain install nightly
```

### Running Fuzz Tests

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run compression fuzzer
cargo +nightly fuzz run compress

# Run decompression fuzzer
cargo +nightly fuzz run decompress

# List available fuzz targets
cargo fuzz list
```

### Available Fuzz Targets

- `compress` - Tests compression with random input data
- `decompress` - Tests decompression with random compressed data

## License

This project is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.

### Copyright Notice

```
Original Work:
  Copyright (C) 2020-2021 BMW Group
  Author: Kjell Braden <kjell.braden@bmw.de>

Modified Work (Fork):
  Copyright (C) 2026-present Andrew Rogoff
  Maintainer: Andrew Rogoff <andrew@andrewbrogoff.com>
```

### License Terms

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

### License Files

- **COPYING** - Full text of the GNU General Public License v3.0
- **NOTICE** - Detailed copyright and attribution information

### Source Code

The complete source code is available at:
https://github.com/andrewbrogoff/uclcli-reborn/

As required by the GPL, all source code modifications are clearly marked and
documented. See the NOTICE file for details on modifications made to the
original work.

### Third-Party Dependencies

This software links against **libucl** (UCL compression library). Please refer
to the libucl documentation for its licensing terms.
