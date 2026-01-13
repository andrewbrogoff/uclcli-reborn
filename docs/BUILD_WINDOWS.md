# Building uclcli on Windows

## Option 1: MSYS2/MinGW (Recommended)

### Step 1: Install MSYS2

Download and install from https://www.msys2.org/

### Step 2: Install Build Tools

Open **MSYS2 MinGW64** terminal and run:

```bash
pacman -Syu
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-make base-devel
```

### Step 3: Install Rust

In the same terminal:

```bash
pacman -S mingw-w64-x86_64-rust
```

Or install via rustup (https://rustup.rs) if you prefer.

### Step 4: Build libucl

```bash
# Download
curl -O https://www.oberhumer.com/opensource/ucl/download/ucl-1.03.tar.gz
tar xzf ucl-1.03.tar.gz
cd ucl-1.03

# Build and install
./configure --prefix=/mingw64
make
make install
```

### Step 5: Build uclcli

```bash
cd /path/to/uclcli
cargo build --release

# Binaries are in target/release/
ls target/release/*.exe
```

### Step 6: Copy to Windows PATH (Optional)

```bash
cp target/release/ucl.exe target/release/unucl.exe /mingw64/bin/
```

---

## Option 2: Visual Studio

### Step 1: Install Visual Studio

Install Visual Studio 2026 with "Desktop development with C++" workload.

### Step 2: Install Rust

Download and run rustup-init.exe from https://rustup.rs

### Step 3: Build libucl

1. Download UCL source from https://www.oberhumer.com/opensource/ucl/
2. Extract and open Developer Command Prompt
3. Build manually or create a Visual Studio project

```cmd
cd ucl-1.03\src
cl /c /O2 /I.. /I..\include ucl_init.c alloc.c n2b_99.c n2b_ds.c ucl_util.c ucl_ptr.c
lib /OUT:ucl.lib *.obj
```

4. Copy `ucl.lib` to a known location (e.g., `C:\libs\ucl\`)
5. Copy headers from `include\ucl\` to `C:\libs\ucl\include\`

### Step 4: Build uclcli

```cmd
set LIB=%LIB%;C:\libs\ucl
cargo build --release
```

---

## Testing

```cmd
echo Hello World | target\release\ucl.exe > test.nrv
target\release\unucl.exe < test.nrv
del test.nrv
```

## Troubleshooting

### "unable to find library -lucl" or "cannot find ucl.lib"

The library path isn't set correctly. For MSYS2:
```bash
export LIBRARY_PATH=/mingw64/lib
```

For Visual Studio:
```cmd
set LIB=%LIB%;C:\path\to\ucl\lib
```

### Runtime error: "ucl init failed"

Wrong libucl version. Ensure you're using UCL 1.03.
