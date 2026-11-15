use std::path::{Path, PathBuf};

pub mod cargo;
pub mod cli;
pub mod meta;
pub mod shell;

/// NDK `toolchains/llvm/prebuilt/<tag>` directory names that can be used on this
/// host, in order of preference.
///
/// Google only ships `linux-x86_64` NDK hosts, but community builds of the NDK
/// that run natively on aarch64 Linux / Android (Termux) use `linux-aarch64`.
#[cfg(target_os = "macos")]
const HOST_TAGS: &[&str] = &["darwin-x86_64"];
#[cfg(all(any(target_os = "linux", target_os = "android"), target_arch = "aarch64"))]
const HOST_TAGS: &[&str] = &["linux-aarch64", "linux-arm64", "linux-x86_64"];
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "aarch64")
))]
const HOST_TAGS: &[&str] = &["linux-x86_64"];
#[cfg(target_os = "windows")]
const HOST_TAGS: &[&str] = &["windows-x86_64"];

/// Default prebuilt tag, used when nothing can be detected inside the NDK.
pub(crate) const ARCH: &str = HOST_TAGS[0];

#[cfg(not(any(
    target_os = "android",
    target_os = "macos",
    target_os = "linux",
    target_os = "windows"
)))]
compile_error!("Unsupported target OS");

/// Work out the name of the `toolchains/llvm/prebuilt/<tag>` directory to use
/// for the given NDK.
///
/// Order: the `CARGO_NDK_HOST_TAG` environment variable, then the first
/// existing directory from the list of tags that fit this host, then (on
/// Linux/Android) the only `linux-*` directory present, and finally the
/// default tag.
pub(crate) fn host_tag(ndk_home: &Path) -> String {
    if let Some(tag) = std::env::var("CARGO_NDK_HOST_TAG")
        .ok()
        .filter(|tag| !tag.is_empty())
    {
        return tag;
    }

    let prebuilt = ndk_home.join("toolchains").join("llvm").join("prebuilt");

    if let Some(tag) = HOST_TAGS.iter().find(|tag| prebuilt.join(tag).is_dir()) {
        return (*tag).to_string();
    }

    if cfg!(any(target_os = "linux", target_os = "android")) {
        let mut linux_dirs: Vec<String> = std::fs::read_dir(&prebuilt)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.starts_with("linux-"))
            .collect();

        if linux_dirs.len() == 1 {
            return linux_dirs.remove(0);
        }
    }

    ARCH.to_string()
}

pub(crate) fn clang_target(rust_target: &str, api_level: u8) -> String {
    let target = match rust_target {
        "arm-linux-androideabi" => "armv7a-linux-androideabi",
        "armv7-linux-androideabi" => "armv7a-linux-androideabi",
        _ => rust_target,
    };
    format!("--target={target}{api_level}")
}

pub(crate) fn sysroot_target(rust_target: &str) -> &str {
    (match rust_target {
        "armv7-linux-androideabi" => "arm-linux-androideabi",
        _ => rust_target,
    }) as _
}

pub(crate) fn ndk_tool(arch: &str, tool: &str) -> PathBuf {
    ["toolchains", "llvm", "prebuilt", arch, "bin", tool]
        .iter()
        .collect()
}

pub(crate) fn sysroot_suffix(arch: &str) -> PathBuf {
    ["toolchains", "llvm", "prebuilt", arch, "sysroot"]
        .iter()
        .collect()
}

#[cfg(all(test, any(target_os = "linux", target_os = "android")))]
mod tests {
    use super::host_tag;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn host_tag_uses_the_only_linux_prebuilt_dir() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let ndk_home = std::env::temp_dir().join(format!("cargo-ndk-host-tag-{nanos}"));
        let prebuilt = ndk_home.join("toolchains").join("llvm").join("prebuilt");
        fs::create_dir_all(prebuilt.join("linux-custom")).expect("failed to create prebuilt dir");

        if std::env::var_os("CARGO_NDK_HOST_TAG").is_none() {
            assert_eq!(host_tag(&ndk_home), "linux-custom");
        }

        fs::remove_dir_all(ndk_home).ok();
    }
}
