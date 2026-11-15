# cargo-ndk-arm — cargo-ndk that runs on Android (Termux) and ARM64

`cargo-ndk-arm` is a fork of [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk) by Brendan Molloy.
It does everything upstream `cargo-ndk` does (configures the environment so Rust code builds against the Android NDK, and produces a `jniLibs` layout),
and additionally **runs natively on an Android device** — in [Termux](https://termux.dev) or an Android IDE terminal — on **ARM64 (`aarch64`)** hardware.

The installed commands keep the upstream names, so existing scripts and docs keep working:

- `cargo ndk` — a passthrough for `cargo` that applies all the environment variables needed to build against the NDK
- `cargo ndk-test` — run tests on a device through `adb`
- `cargo ndk-env` — print the environment (shell exports, PowerShell, or JSON for rust-analyzer)
- `cargo-ndk-runner` — runner used by `cargo run`/`cargo test` to execute binaries through `adb`

> **Note:** this fork installs the same binary names as upstream `cargo-ndk`. Installing it replaces an upstream `cargo-ndk` in `~/.cargo/bin`.

## Table of contents

- [What is different from upstream](#what-is-different-from-upstream)
- [Requirements](#requirements)
- [Quick start (Termux, ARM64)](#quick-start-termux-arm64)
- [Getting an ARM64 NDK](#getting-an-arm64-ndk)
- [Verify the installation](#verify-the-installation)
- [Usage](#usage)
- [Environment variables](#environment-variables)
- [How the NDK host directory is chosen](#how-the-ndk-host-directory-is-chosen)
- [Troubleshooting](#troubleshooting)
- [Tested configuration and limitations](#tested-configuration-and-limitations)
- [Building and installing from source](#building-and-installing-from-source)
- [Credits](#credits)
- [License](#license)

## What is different from upstream

| Area | Upstream `cargo-ndk` | `cargo-ndk-arm` |
|------|----------------------|-----------------|
| Running on Android | Refused to build for an Android host (`CARGO_NDK_ON_ANDROID` opt-in) | Builds and runs on Android with no flags |
| NDK host directory | Hard-coded `linux-x86_64` on Linux | Detected at run time: `linux-aarch64`, `linux-arm64`, `linux-x86_64`, or the only `linux-*` directory; override with `CARGO_NDK_HOST_TAG` |
| Default NDK location on Android | none | `$HOME/android-sdk/ndk`, then `$HOME/Android/Sdk/ndk` |
| `adb` lookup | Ran `which adb` | Scans `PATH` directly (`which` is not installed in Termux by default) |

Everything else (CLI, flags, output layout, environment variables) is unchanged.

## Requirements

- An Android device with **ARM64 (`aarch64`)** CPU and [Termux](https://termux.dev) (or another Linux-like terminal environment on Android)
- The Rust toolchain for your terminal, with the `aarch64-linux-android` standard library
- An **Android NDK that runs on ARM64 Linux/Android** (see [Getting an ARM64 NDK](#getting-an-arm64-ndk))
- Optional: `adb` for `cargo ndk-test` / `cargo-ndk-runner`

## Quick start (Termux, ARM64)

1. Install Rust and build tools. In Termux the `rust` package already comes with the `aarch64-linux-android` standard library:

   ```sh
   pkg update
   pkg install rust clang git
   ```

2. Get the source into your Termux **home directory** (not shared storage such as `/storage/emulated/0/...` — Android does not allow executing files there, so builds fail):

   ```sh
   git clone https://github.com/CanavarIT/cargo-ndk-arm
   cd cargo-ndk-arm
   ```

3. Install:

   ```sh
   cargo install --path . --locked
   ```

   This builds on the device and puts `cargo-ndk`, `cargo-ndk-env`, `cargo-ndk-test` and `cargo-ndk-runner` in `~/.cargo/bin`.

4. Make sure `~/.cargo/bin` is on your `PATH`, and tell `cargo-ndk` where the NDK is:

   ```sh
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   echo 'export ANDROID_NDK_HOME=$HOME/android-sdk/ndk/<ndk-version>' >> ~/.bashrc
   source ~/.bashrc
   ```

   Replace `<ndk-version>` with your NDK directory name, for example `29.0.14033849`.

## Getting an ARM64 NDK

Google's official NDK ships host toolchains for `x86_64` only, so it does **not** run on an ARM64 phone. You need an NDK that was built to run on aarch64 Linux/Android; community builds exist.

Requirements for the NDK directory:

- It must live **inside Termux's own home directory** (for example `~/android-sdk/ndk/<version>`). Another app's private directory such as `/data/data/<other.app>/...` is not readable from Termux, and every command fails with `Permission denied (os error 13)`. Copy or extract the NDK into `~` instead (`tar` preserves permissions and symlinks).
- It must contain `toolchains/llvm/prebuilt/<tag>/` where `<tag>` is something like `linux-aarch64` or `linux-arm64`.

You can check what you have with:

```sh
ls "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/"
```

## Verify the installation

Print the environment `cargo-ndk` would use:

```sh
cargo ndk-env --target arm64-v8a
```

The paths in the output should point into `.../toolchains/llvm/prebuilt/<your-tag>/...`.

Build and run a real binary directly on the device:

```sh
cargo new hello && cd hello
cargo ndk -t arm64-v8a build --release
./target/aarch64-linux-android/release/hello     # prints: Hello, world!
```

Build a shared library, as you would for JNI:

```sh
cargo new --lib mylib && cd mylib
printf '\n[lib]\ncrate-type = ["cdylib"]\n' >> Cargo.toml
cargo ndk -t arm64-v8a -o jniLibs build --release
ls jniLibs/arm64-v8a                              # libmylib.so
```

## Usage

The command-line interface is the same as upstream `cargo-ndk`.

### Build for one or several ABIs

```sh
cargo ndk -t arm64-v8a build --release
cargo ndk -t armeabi-v7a -t arm64-v8a -o ./jniLibs build --release
```

`-t` selects the Android targets (ordinary Rust target triples are accepted too), `-o` sets the output directory in which the `.so` files are placed in the layout Android expects (`jniLibs/<abi>/lib<name>.so`), and everything after that is passed to `cargo`.

On a phone you will normally want only `arm64-v8a`. Other ABIs need the matching Rust standard library (for example `aarch64-linux-android` is bundled with Termux's `rust`, other targets may require extra packages — check `pkg search rust-std`).

### Choose the Android API level

```sh
cargo ndk -t arm64-v8a --platform 28 build --release
```

### Run binaries and tests through `adb`

To make `cargo run` execute Android builds through `adb`, add this to the project's `.cargo/config.toml`:

```toml
[target.aarch64-linux-android]
runner = "cargo ndk-runner"
```

Run tests on a connected device:

```sh
cargo ndk-test -t arm64-v8a
```

`adb` is looked up in `ANDROID_HOME` and then in `PATH`. On Termux: `pkg install android-tools`.

### Verbosity

Add `-v` or `-vv` after the cargo command as you normally would.

### Extra linking helpers

- `--link-builtins` — use if the build complains that compiler builtins are missing
- `--link-libcxx-shared` — use if you need to link `libc++_shared.so`

### Printing the environment

```sh
source <(cargo ndk-env)                        # bash/zsh
cargo ndk-env --powershell | Invoke-Expression # PowerShell
cargo ndk-env --json                           # for rust-analyzer and similar
```

By default `cargo ndk-env` omits cargo-ndk's internal linker-wrapper variables. To source the environment and then call plain `cargo build` with the exported linker configuration, add `--include-internal` and do it in a subshell:

```sh
(
  source <(cargo ndk-env --target arm64-v8a --include-internal)
  cargo build --target aarch64-linux-android
)
```

## Environment variables

Variables you can set:

| Variable | Description |
|----------|-------------|
| `ANDROID_NDK_HOME` | Path to the NDK root. If unset, the highest NDK version found in the default locations is used |
| `CARGO_NDK_HOST_TAG` | Force the `toolchains/llvm/prebuilt/<tag>` directory name (for example `linux-arm64`). **New in this fork** |
| `CARGO_NDK_TARGET` | Default target(s), comma-separated |
| `CARGO_NDK_PLATFORM` | Default Android API level |
| `CARGO_NDK_OUTPUT_PATH` | Default output directory |

Command-line arguments override these.

Variables exported to builds and build scripts:

| Variable | Description | Example |
|----------|-------------|---------|
| `CARGO_NDK_ANDROID_PLATFORM` | Android API level | `21` |
| `CARGO_NDK_NDK_VERSION` | Detected NDK revision | `29.0.14206865` |
| `CARGO_NDK_OUTPUT_PATH` | Output path given with `-o` | `./jniLibs` |
| `CARGO_NDK_SYSROOT_PATH` | Sysroot inside the NDK | `<ndk>/toolchains/llvm/prebuilt/<tag>/sysroot` |
| `CARGO_NDK_SYSROOT_TARGET` | Target name used inside the sysroot | `aarch64-linux-android` |
| `CARGO_NDK_SYSROOT_LIBS_PATH` | Libraries path inside the sysroot | `<sysroot>/usr/lib/aarch64-linux-android` |
| `LIBCLANG_PATH` | Directory with the NDK's libclang, when detected | |
| `ANDROID_PLATFORM` | Platform version in use | `21` |
| `ANDROID_ABI` | Android ABI name | `arm64-v8a` |

Variables for C dependencies are derived the same way as in the `cc` crate (`CC_aarch64_linux_android`, `AR_...`, `CFLAGS_...`, and so on), and bindgen variables are set automatically.

## How the NDK host directory is chosen

NDK toolchains live in `<NDK>/toolchains/llvm/prebuilt/<tag>`. `cargo-ndk-arm` picks `<tag>` in this order:

1. The value of `CARGO_NDK_HOST_TAG`, if set (use it for any unusual layout)
2. The first existing directory from this list for your host:
   - ARM64 Linux / Android: `linux-aarch64`, `linux-arm64`, `linux-x86_64`
   - x86_64 Linux: `linux-x86_64`
   - macOS: `darwin-x86_64`
   - Windows: `windows-x86_64`
3. On Linux/Android: the only `linux-*` directory in `prebuilt/`, if there is exactly one

Symlinked directories are supported.

## Troubleshooting

**`Error: Permission denied (os error 13)`** — the NDK is in a place Termux cannot read (for example another app's `/data/data/...` directory). Copy it into your Termux home and point `ANDROID_NDK_HOME` there.

**Builds fail on shared storage (`/storage/emulated/0/...`, `~/storage/...`)** — Android does not allow executing programs from shared storage. Keep projects and the NDK in the Termux home directory.

**`cargo: command not found` / `cargo-ndk: command not found`** — add `~/.cargo/bin` to `PATH` (see Quick start step 4).

**Could not find the NDK** — set `ANDROID_NDK_HOME`, or place the NDK in `~/android-sdk/ndk/<version>` or `~/Android/Sdk/ndk/<version>`.

**Toolchain directory not found** — run `ls "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/"` and set `CARGO_NDK_HOST_TAG` to the directory that contains `bin/clang` for your CPU.

**`adb` not found** — `pkg install android-tools`, or set `ANDROID_HOME`.

**Compiler builtins missing** — add `--link-builtins`. **Need `libc++_shared.so`** — add `--link-libcxx-shared`.

## Tested configuration and limitations

Tested by the author:

- Termux on an ARM64 Android device
- Rust 1.99.1 from the Termux `rust` package, which includes `aarch64-linux-android`
- Android NDK 29.0.14033849, an ARM64-hosted build with a `linux-arm64` prebuilt directory
- Building an executable and a `cdylib` for `arm64-v8a`, and running the executable on the device

Not tested: ABIs other than `arm64-v8a`, `cargo ndk-test`/`cargo-ndk-runner` on a device, native ARM64 desktop Linux, and other terminal environments on Android. Reports and fixes are welcome.

## Building and installing from source

```sh
git clone https://github.com/CanavarIT/cargo-ndk-arm
cd cargo-ndk-arm
cargo install --path . --locked
```

To run the unit tests: `cargo test`.

## Credits

`cargo-ndk` is written by Brendan Molloy and contributors: <https://github.com/bbqsrc/cargo-ndk>. This fork adds Android/ARM64 host support and keeps the upstream license. The original upstream `CHANGELOG.md` is preserved; see its "Unreleased" section for the changes made here.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option. Original copyright belongs to Brendan Molloy; modifications are copyright CanavarIT.
