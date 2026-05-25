/// Helpers
use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use crate::term;

pub fn run(section: &str, cmd: &mut Command) -> Result<()> {
    term::running(section, &format!(">> {:?}", cmd));

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to launch {:?}", cmd.get_program()))?;

    let stdout = child.stdout.take().context("Failed to open stdout")?;
    let stderr = child.stderr.take().context("Failed to open stderr")?;

    let err_sec = section.to_string();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            term::log(&err_sec, &format!("{} {}", "[ERR]".red(), line));
        }
    });

    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        term::log(section, &format!("| {}", line));
    }

    let _ = stderr_handle.join();
    let status = child.wait()?;

    if !status.success() {
        bail!("{:?} exited with {}", cmd.get_program(), status);
    }

    Ok(())
}

pub fn run_output(cmd: &mut Command) -> Result<String> {
    let out = cmd
        .output()
        .with_context(|| format!("Failed to launch {:?}", cmd.get_program()))?;
    if !out.status.success() {
        bail!(
            "{:?} exited with {}\nstderr: {}",
            cmd.get_program(),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Glob-style filename match (supports leading/trailing * only, as in the PS script).
pub fn matches_pattern(name: &str, pattern: &str) -> bool {
    let pat = pattern.trim_matches('*');
    if pattern.starts_with('*') && pattern.ends_with('*') {
        name.contains(pat)
    } else if pattern.starts_with('*') {
        name.ends_with(pat)
    } else if pattern.ends_with('*') {
        name.starts_with(pat)
    } else {
        name == pattern
    }
}

/// Case-insensitive directory glob: find all subdirs of `parent` whose names
/// match any of `patterns`.
pub fn find_matching_dirs(parent: &Path, patterns: &[&str]) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let Ok(rd) = fs::read_dir(parent) else {
        return results;
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        for pat in patterns {
            if matches_pattern(&name, &pat.to_lowercase()) {
                results.push(entry.path());
                break;
            }
        }
    }
    results
}

/// Silently remove a path (file or directory), logging on failure.
pub fn remove_path(p: &Path) {
    if !p.exists() {
        return;
    }
    let r = if p.is_dir() {
        fs::remove_dir_all(p)
    } else {
        fs::remove_file(p)
    };
    if let Err(e) = r {
        term::warn(&format!("Could not remove {}: {e}", p.display()));
    }
}
