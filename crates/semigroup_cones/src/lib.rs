//! Shared utilities for the `waldicone` family of binaries: Normaliz
//! invocation and output parsing, execution-mode control, and path helpers.

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]
#![warn(
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Controls whether the `(m, q₁)` workload and per-lattice-point
/// post-processing run across rayon worker threads or single-threaded.
///
/// Sequential mode exists because Normaliz itself parallelises heavily inside
/// each spawn; layering rayon on top causes thread contention with no
/// measurable speedup on the machines tested so far.
#[derive(Debug, Clone, Copy)]
pub enum ExecMode {
    /// Distribute the workload over rayon's thread pool.
    Parallel,
    /// Process every item in the order it appears, without rayon.
    Sequential,
}

/// Joins an iterator of [`Display`][std::fmt::Display]-able values with spaces,
/// producing a single space-separated row suitable for a Normaliz input file.
pub fn join_row(iter: impl Iterator<Item = impl ToString>) -> String {
    iter.map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
}

/// The CLI flag that selects sequential execution mode.
///
/// Pass this as the second argument to any `waldicone` binary to disable
/// rayon parallelism (useful when Normaliz already saturates all cores).
pub const SEQ_FLAG: &str = "seq";

/// Returns the project-relative path to the bundled Normaliz 3.11.1 binary
/// for the current OS, resolved relative to the current working directory.
///
/// * Linux   → `normaliz/normaliz-3.11.1-Linux/normaliz`
/// * Windows → `normaliz/normaliz-3.11.1-Windows/normaliz.exe`
#[must_use]
pub fn bundled_normaliz_path() -> PathBuf {
    let (subdir, exe) = if cfg!(target_os = "windows") {
        ("normaliz-3.11.1-Windows", "normaliz.exe")
    } else {
        ("normaliz-3.11.1-Linux", "normaliz")
    };
    Path::new("normaliz").join(subdir).join(exe)
}

/// Spawns `normaliz_bin` with `in_path` as its sole argument and waits for
/// completion, capturing stdout/stderr.
///
/// Inheriting the parent's stdio (the old behaviour) caused two Windows-only
/// failure modes to be silent: child errors lost in the parallel storm, and a
/// phantom console window flashing for every spawn.
///
/// # Errors
///
/// Returns an error if the process cannot be spawned or if Normaliz exits with
/// a non-zero status; the error message includes both stdout and stderr.
pub fn run_normaliz(normaliz_bin: &Path, in_path: &Path) -> std::io::Result<()> {
    let output = Command::new(normaliz_bin)
        .arg(in_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(std::io::Error::other(format!(
        "normaliz exited with code {} for {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status.code().unwrap_or(-1),
        in_path.display(),
        String::from_utf8_lossy(&output.stdout).trim_end(),
        String::from_utf8_lossy(&output.stderr).trim_end(),
    )))
}

/// Parses a Normaliz `.out` file, returning the lattice-point count and each
/// point's coordinate vector with the trailing dehomogenization column removed.
///
/// Returns `(0, [])` for infeasible (empty) polytopes.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be read.
pub fn parse_out_file(path: &Path) -> std::io::Result<(usize, Vec<Vec<i64>>)> {
    let content = fs::read_to_string(path)?;
    let first = content.lines().next().unwrap_or_default();
    let count: usize = first
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if count == 0 {
        return Ok((0, Vec::new()));
    }
    let sep_pos = content.find("***").unwrap_or(content.len());
    let after = &content[sep_pos..];
    let marker = format!("{count} lattice points in polytope (module generators):");
    let mut points = Vec::with_capacity(count);
    if let Some(pos) = after.find(&marker) {
        for line in after[pos..].lines().skip(1).take(count) {
            let row: Vec<i64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if !row.is_empty() {
                let n = row.len().saturating_sub(1);
                points.push(row[..n].to_vec());
            }
        }
    }
    Ok((count, points))
}

/// Resolves and validates the bundled Normaliz binary, prints its version
/// banner, and returns its **absolute** path for unambiguous child-process
/// spawning.
///
/// Passing a relative or bare name to `CreateProcess` on Windows searches the
/// directory of the calling executable first, which is `target\release\` and
/// contains the cargo-built `normaliz.exe` (this binary itself). That
/// self-spawn caused the historical fork-bomb where the loop kept restarting
/// the same g/m/q1. The absolute-path requirement prevents that.
///
/// Calls [`std::process::exit(1)`] on any error. Intended for call from
/// `main` only.
// ALLOW: diagnostic prints are the explicit purpose of this preflight function.
#[must_use]
#[allow(clippy::print_stdout, clippy::print_stderr)]
pub fn ensure_normaliz_available() -> PathBuf {
    let rel = bundled_normaliz_path();
    let abs = match rel.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "error: cannot resolve bundled Normaliz at `{}` ({e}). Run \
                 from the project root and make sure the `normaliz/` folder \
                 with the bundled distribution is present.",
                rel.display(),
            );
            std::process::exit(1);
        }
    };

    // Defence against the original Windows fork-bomb: bail loudly if the
    // bundled binary path resolves to this very executable.
    if let Ok(self_path) = std::env::current_exe()
        && let Ok(self_canon) = self_path.canonicalize()
        && self_canon == abs
    {
        eprintln!(
            "error: bundled Normaliz path {} resolves to this binary itself \
             — refusing to self-spawn. Check the `normaliz/` folder layout.",
            abs.display(),
        );
        std::process::exit(1);
    }

    match Command::new(&abs)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
    {
        Ok(out) if out.status.success() => {
            let banner = String::from_utf8_lossy(&out.stdout);
            let version = banner.lines().next().unwrap_or("Normaliz");
            println!("using bundled {version} at {}", abs.display());
            abs
        }
        Ok(out) => {
            eprintln!(
                "error: `{} --version` exited with {}\n--- stderr ---\n{}",
                abs.display(),
                out.status,
                String::from_utf8_lossy(&out.stderr).trim_end(),
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "error: cannot launch bundled Normaliz at {} ({e}). On Linux \
                 ensure the file is executable (`chmod +x {}`); only Linux \
                 and Windows binaries are bundled.",
                abs.display(),
                abs.display(),
            );
            std::process::exit(1);
        }
    }
}
