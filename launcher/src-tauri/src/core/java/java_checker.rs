use std::path::Path;
use std::process::Command;
use tracing::trace;

pub struct JavaVersion {
    pub major: u32,
    pub vendor: String,
}

/// Run `java -version` on the given executable and parse its output.
///
/// Returns `None` if the executable cannot be run or the output cannot be parsed.
pub fn check_java_version(java_exe: &Path) -> Option<JavaVersion> {
    // `java -version` always writes to stderr regardless of JVM vendor.
    let output = Command::new(java_exe)
        .arg("-version")
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stderr);
    trace!("java -version output from {}: {}", java_exe.display(), text.trim());
    parse_version_output(&text)
}

fn parse_version_output(output: &str) -> Option<JavaVersion> {
    // First line examples:
    //   openjdk version "21.0.2" 2024-01-16
    //   java version "17.0.9" 2023-10-17
    //   openjdk version "1.8.0_392" 2023-10-17   ← legacy "1.x" scheme
    let first_line = output.lines().next()?;

    let vendor = if first_line.starts_with("openjdk") {
        "OpenJDK"
    } else if first_line.starts_with("java") {
        "Oracle"
    } else {
        "Java"
    };

    // Extract the quoted version string
    let after_open  = first_line.find('"')? + 1;
    let after_close = first_line[after_open..].find('"')? + after_open;
    let version_str = &first_line[after_open..after_close];

    let major = parse_major_version(version_str)?;
    Some(JavaVersion { major, vendor: vendor.to_string() })
}

/// Parse the major version from strings like "21.0.2", "17.0.9", "1.8.0_392".
fn parse_major_version(version_str: &str) -> Option<u32> {
    let mut parts = version_str.splitn(3, ['.', '_']);
    match parts.next()? {
        "1" => parts.next()?.parse().ok(), // Legacy: "1.8" → 8
        major => major.parse().ok(),        // Modern: "21" → 21
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modern_openjdk() {
        let out = r#"openjdk version "21.0.2" 2024-01-16
OpenJDK Runtime Environment Temurin-21.0.2+13 (build 21.0.2+13)
"#;
        let v = parse_version_output(out).unwrap();
        assert_eq!(v.major, 21);
        assert_eq!(v.vendor, "OpenJDK");
    }

    #[test]
    fn parses_legacy_java8() {
        let out = r#"java version "1.8.0_392" 2023-10-17
Java(TM) SE Runtime Environment (build 1.8.0_392-b08)
"#;
        let v = parse_version_output(out).unwrap();
        assert_eq!(v.major, 8);
        assert_eq!(v.vendor, "Oracle");
    }

    #[test]
    fn parses_java17_too_old() {
        let out = r#"openjdk version "17.0.9" 2023-10-17
OpenJDK Runtime Environment (build 17.0.9+9)
"#;
        let v = parse_version_output(out).unwrap();
        assert_eq!(v.major, 17);
        // Java 17 < MIN_JAVA_VERSION (21) → would be reported as TooOld
        assert!(v.major < super::super::MIN_JAVA_VERSION);
    }
}
