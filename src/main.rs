/// win-debloat — Windows ISO debloater for Linux
///
/// Requires: xorriso, wimlib-imagex, hivexregedit/hivex
mod fixes;
mod helpers;

use anyhow::{Context, Error, Result, bail};
use clap::Parser;
use colored::Colorize;
use fixes::{ai, apps, appx, autounattend, cap, registry};
use helpers::{cli, consts, h, iso, term, wim};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

fn check_deps() -> Result<()> {
    term::section("Checking dependencies");
    let mut good = true;
    let tools = [
        ("7z", consts::P7ZIP),
        ("xorriso", consts::XORRISO),
        ("wimlib-imagex", consts::WIMLIB),
        ("hivexregedit", consts::HIVEXREG),
    ];
    for (name, path) in tools {
        if std::path::Path::new(path).is_absolute() {
            // Built with Nix — check the store path exists
            if std::path::Path::new(path).exists() {
                term::ok(&format!("Static: {} ({})", name, path));
            } else {
                term::err(&format!("Static: {} ({})", name, path));
                good = false;
            }
        } else {
            // Fell back to PATH lookup
            match which::which(path) {
                Ok(p) => term::ok(&format!("PATH: {} ({})", name, p.display())),
                Err(_) => {
                    term::err(&format!("PATH: {} ({})", name, path));
                    good = false;
                }
            }
        }
    }
    match good {
        true => return Ok(()),
        false => return Err(Error::msg("Missing dependencies")),
    }
}

fn main() -> Result<()> {
    // Banner
    term::banner();

    let cli = cli::Cli::parse();

    if cli.test {
        return check_deps();
    }

    // Validate input
    if !cli.iso.exists() {
        bail!("ISO not found: {}", cli.iso.display());
    }

    // Set up staging directory
    let _tmp_guard; // keeps TempDir alive
    let staging_root: PathBuf = if let Some(ref d) = cli.staging_dir {
        fs::create_dir_all(&d)?;
        d.canonicalize()?
    } else {
        _tmp_guard = TempDir::new().context("Cannot create temp dir")?;
        _tmp_guard.path().to_path_buf()
    };

    let staging = staging_root.join("iso_staging");
    let mount_dir = staging_root.join("wim_mount");
    let work_wim = staging_root.join("install_work.wim");

    // Extract ISO
    fs::create_dir_all(&staging)?;
    iso::extract_iso(&cli.iso, &staging)?;

    // Find install.wim
    let sources = staging.join("sources");
    let install_wim = sources.join("install.wim");

    // List images
    term::section("Selecting Windows edition");
    let images = wim::list_wim_images(&install_wim)?;
    for (idx, name) in &images {
        term::info(&format!("{}. {}", idx, name));
    }

    // Pick edition
    let chosen_index: u32 = images
        .iter()
        .find(|(_, n)| n.to_lowercase() == cli.edition.to_lowercase())
        .map(|(i, _)| *i)
        .unwrap_or_else(|| {
            term::warn(&format!(
                "Edition '{}' not found — using index 1",
                cli.edition
            ));
            1
        });

    // Mount WIM
    wim::mount_wim(&install_wim, chosen_index, &mount_dir)?;
    term::section("Mounting WIM");
    term::ok("WIM mounted");

    // Use specified language
    let lang = cli.language;
    term::info(&format!("Using language: {lang}"));

    // Run debloat steps (errors in individual steps are logged but non-fatal)
    let run_step = |result: Result<()>, label: &str| {
        if let Err(e) = result {
            term::warn(&format!("{label} failed: {e}"));
        }
    };

    if cli.appx {
        run_step(appx::remove_appx(&mount_dir), "AppX removal");
    }
    if cli.capabilities {
        run_step(
            apps::remove_capabilities(&mount_dir, &lang),
            "Capabilities removal",
        );
        run_step(
            cap::remove_capabilities(&mount_dir, &lang),
            "Capabilities removal",
        );
    }
    if cli.onedrive {
        run_step(apps::remove_onedrive(&mount_dir), "OneDrive removal");
    }
    if cli.edge {
        run_step(apps::remove_edge(&mount_dir), "Edge removal");
    }
    if cli.ai {
        run_step(ai::remove_ai(&mount_dir), "AI removal");
    }
    if cli.tpm_bypass {
        run_step(registry::apply_tpm_bypass_tweaks(&mount_dir), "TPM bypass");
    }

    // Always apply privacy tweaks and enable user folders
    run_step(registry::apply_privacy_tweaks(&mount_dir), "Privacy tweaks");
    run_step(registry::enable_user_folders(&mount_dir), "User folders");
    run_step(registry::apply_gamebar_tweaks(&mount_dir), "Gamebar tweaks");
    run_step(registry::apply_task_tweaks(&mount_dir), "Task tweaks");

    // Unmount + commit
    wim::unmount_wim(&mount_dir, &staging_root)?;

    // Write autounattend
    autounattend::write_autounattend(&staging, &lang, chosen_index)?;

    // Repack ISO
    iso::repack_iso(&staging, &cli.output)?;

    // Cleanup
    if !cli.keep_staging {
        term::info("Cleaning up staging directory…");
        h::remove_path(&staging);
        h::remove_path(&mount_dir);
        h::remove_path(&work_wim);
    }

    term::ok(&format!(
        "Debloated ISO ready: {}",
        cli.output.display().to_string().bold()
    ));

    Ok(())
}
