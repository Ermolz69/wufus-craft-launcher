pub mod java_checker;
pub mod java_finder;
pub mod java_info;

pub use java_info::JavaInfo;

use java_checker::check_java_version;
use java_finder::find_java_candidates;
use std::path::Path;

/// NeoForge on Minecraft 1.21.1 requires Java 21.
pub const MIN_JAVA_VERSION: u32 = 21;

/// Search for the best available Java installation.
///
/// If `explicit_path` is `Some`, only that executable is checked.
/// Otherwise all known locations are probed in priority order.
///
/// Returns the first [`JavaInfo`] whose major version ≥ [`MIN_JAVA_VERSION`],
/// or `None` if nothing suitable is found.
pub fn find_suitable_java(explicit_path: Option<&str>) -> Option<JavaInfo> {
    let candidates: Vec<_> = match explicit_path {
        Some(p) => vec![Path::new(p).to_path_buf()],
        None => find_java_candidates(),
    };

    candidates
        .into_iter()
        .find_map(|exe| {
            let ver = check_java_version(&exe)?;
            if ver.major >= MIN_JAVA_VERSION {
                Some(JavaInfo {
                    path: exe.to_string_lossy().into_owned(),
                    version: ver.major,
                    vendor: ver.vendor,
                })
            } else {
                None
            }
        })
}

/// Check *all* candidates and return the first one regardless of version.
/// Used to distinguish "found but too old" from "not found at all".
pub fn find_any_java(explicit_path: Option<&str>) -> Option<JavaInfo> {
    let candidates: Vec<_> = match explicit_path {
        Some(p) => vec![Path::new(p).to_path_buf()],
        None => find_java_candidates(),
    };

    candidates
        .into_iter()
        .find_map(|exe| {
            let ver = check_java_version(&exe)?;
            Some(JavaInfo {
                path: exe.to_string_lossy().into_owned(),
                version: ver.major,
                vendor: ver.vendor,
            })
        })
}
