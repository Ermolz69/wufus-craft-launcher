use std::path::PathBuf;
use tracing::trace;

/// Returns candidate `java` executable paths to probe, ordered by priority:
/// 1. `JAVA_HOME` env var (user explicitly set this)
/// 2. `java` resolved from PATH via `where`/`which`
/// 3. Well-known vendor installation directories (Windows or Unix)
pub fn find_java_candidates() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    collect_java_home(&mut candidates);
    collect_from_path(&mut candidates);
    collect_from_well_known_dirs(&mut candidates);
    candidates
}

fn collect_java_home(out: &mut Vec<PathBuf>) {
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let exe = PathBuf::from(&java_home).join("bin").join(java_exe_name());
        if exe.exists() {
            trace!("JAVA_HOME candidate: {}", exe.display());
            push_unique(out, exe);
        }
    }
}

fn collect_from_path(out: &mut Vec<PathBuf>) {
    if let Some(exe) = locate_in_path() {
        trace!("PATH candidate: {}", exe.display());
        push_unique(out, exe);
    }
}

#[cfg(target_os = "windows")]
fn locate_in_path() -> Option<PathBuf> {
    let output = std::process::Command::new("where.exe")
        .arg("java.exe")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first = stdout.lines().next()?.trim();
    let path = PathBuf::from(first);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
fn locate_in_path() -> Option<PathBuf> {
    let output = std::process::Command::new("which")
        .arg("java")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first = stdout.lines().next()?.trim();
    let path = PathBuf::from(first);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn collect_from_well_known_dirs(out: &mut Vec<PathBuf>) {
    for dir_str in well_known_dirs() {
        let base = PathBuf::from(dir_str);
        if !base.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&base) {
            // Sort entries so newer JDK versions (higher directory name) come first.
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort_by(|a, b| b.cmp(a));

            for subdir in paths {
                let exe = subdir.join("bin").join(java_exe_name());
                if exe.exists() {
                    trace!("Well-known dir candidate: {}", exe.display());
                    push_unique(out, exe);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
const fn well_known_dirs() -> &'static [&'static str] {
    &[
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files\Eclipse Foundation",
        r"C:\Program Files\Java",
        r"C:\Program Files (x86)\Java",
        r"C:\Program Files\Microsoft",
        r"C:\Program Files\Amazon Corretto",
        r"C:\Program Files\Zulu",
        r"C:\Program Files\BellSoft",
        r"C:\Program Files\GraalVM",
        r"C:\Program Files\IBM Semeru Runtime Open Edition",
    ]
}

#[cfg(target_os = "macos")]
const fn well_known_dirs() -> &'static [&'static str] {
    &["/Library/Java/JavaVirtualMachines", "/usr/lib/jvm"]
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const fn well_known_dirs() -> &'static [&'static str] {
    &[
        "/usr/lib/jvm",
        "/usr/local/lib/jvm",
        "/opt/java",
        "/opt/jdk",
    ]
}

#[cfg(target_os = "windows")]
const fn java_exe_name() -> &'static str {
    "java.exe"
}

#[cfg(not(target_os = "windows"))]
const fn java_exe_name() -> &'static str {
    "java"
}

fn push_unique(vec: &mut Vec<PathBuf>, path: PathBuf) {
    if !vec.contains(&path) {
        vec.push(path);
    }
}
