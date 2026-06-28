use serde::Serialize;

/// Metadata about a discovered Java installation.
#[derive(Debug, Clone, Serialize)]
pub struct JavaInfo {
    /// Absolute path to the `java` (or `java.exe`) executable.
    pub path: String,
    /// Major version number (8, 17, 21, …).
    pub version: u32,
    /// Vendor string parsed from `java -version` output (e.g. "OpenJDK", "Oracle").
    pub vendor: String,
}
