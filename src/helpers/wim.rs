/// WIM mounting and editing helpers
use crate::{consts, h, term};
use anyhow::Result;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

/// List images in a WIM/ESD file. Returns Vec<(index, name)>.
pub fn list_wim_images(wim: &Path) -> Result<Vec<(u32, String)>> {
    let out = h::run_output(
        Command::new(consts::WIMLIB)
            .args(["info", wim.to_str().unwrap()])
            .stderr(Stdio::null()),
    )?;

    let mut images = Vec::new();
    let mut current_index: Option<u32> = None;
    let mut current_name: Option<String> = None;

    for line in out.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Index:") {
            // Commit previous
            if let (Some(i), Some(n)) = (current_index, current_name.take()) {
                images.push((i, n));
            }
            current_index = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("Name:") {
            current_name = Some(rest.trim().to_string());
        }
    }
    if let (Some(i), Some(n)) = (current_index, current_name) {
        images.push((i, n));
    }
    Ok(images)
}

/// Mount a WIM image (read-write, FUSE) at mount_point
pub fn mount_wim(wim: &Path, index: u32, mount_point: &Path) -> Result<()> {
    fs::create_dir_all(mount_point)?;
    term::info(&format!(
        "Mounting {} (index {}) at {}",
        wim.display(),
        index,
        mount_point.display()
    ));
    h::run(
        "wim-mount",
        Command::new(consts::WIMLIB).args([
            "mountrw",
            wim.to_str().unwrap(),
            &index.to_string(),
            mount_point.to_str().unwrap(),
        ]),
    )
}

/// Commit and unmount a WIM FUSE mount
pub fn unmount_wim(mount_point: &Path, stage_dir: &Path) -> Result<()> {
    term::info("Committing and unmounting WIM...");
    h::run(
        "wim-umount",
        Command::new(consts::WIMLIB).args(["unmount", mount_point.to_str().unwrap(), "--commit"]),
    )?;
    h::run(
        "wim-umount",
        Command::new("sync").args(["-f", stage_dir.to_str().unwrap()]),
    )
}
