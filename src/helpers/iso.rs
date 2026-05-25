/// ISO helper functions

use crate::{consts, helpers, term};
use anyhow::Result;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn extract_iso(iso: &Path, dest: &Path) -> Result<()> {
    term::section("Extracting ISO");
    term::info(&format!("{} → {}", iso.display(), dest.display()));

    fs::create_dir_all(dest)?;

    helpers::h::run(
        "iso-extract",
        Command::new(consts::P7ZIP)
            .args([
                "x",
                iso.to_str().unwrap(),
                &format!("-o{}", dest.to_str().unwrap()),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null()),
    )?;

    fix_permissions(dest)?;
    term::ok("ISO extracted");
    Ok(())
}

fn fix_permissions(dir: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let mut perms = fs::metadata(entry.path())?.permissions();
        let mode = perms.mode();
        perms.set_mode(mode | 0o200); // set user-write bit
        fs::set_permissions(entry.path(), perms)?;
    }
    Ok(())
}

pub fn repack_iso(staging: &Path, output: &Path) -> Result<()> {
    term::section("Repacking ISO");

    // Parse relevant boot flags; fall back to known-good Windows defaults
    let has_efi = staging.join("efi").exists() || staging.join("EFI").exists();

    let mut args: Vec<String> = vec![
        "-as".into(),
        "mkisofs".into(),
        "-iso-level".into(),
        "3".into(),
        "-full-iso9660-filenames".into(),
        "-J".into(),
        "-joliet-long".into(),
        "-untranslated-filenames".into(),
        "-relaxed-filenames".into(),
        "-allow-lowercase".into(),
        "-volid".into(),
        "WINDOWS".into(),
    ];

    // Add boot catalog from original if we could parse it, otherwise use known paths
    let boot_cat: Option<PathBuf> = {
        let p = staging.join("boot.catalog");
        let p2 = staging.join("boot/etfsboot.com");
        if p.exists() {
            Some(p)
        } else if p2.exists() {
            Some(p2)
        } else {
            None
        }
    };

    let etfs = staging
        .join("boot/etfsboot.com")
        .exists()
        .then(|| staging.join("boot/etfsboot.com"))
        .or_else(|| {
            staging
                .join("Boot/etfsboot.com")
                .exists()
                .then(|| staging.join("Boot/etfsboot.com"))
        });

    if let Some(ref etfs_path) = etfs {
        args.extend([
            "-b".into(),
            {
                let rel = etfs_path.strip_prefix(staging).unwrap();
                rel.to_string_lossy().into_owned()
            },
            "-no-emul-boot".into(),
            "-boot-load-size".into(),
            "8".into(),
            "-boot-info-table".into(),
        ]);
    }

    if has_efi {
        // EFI boot image — find it
        let efi_img = find_efi_image(staging);
        if let Some(ref img) = efi_img {
            let rel = img.strip_prefix(staging).unwrap();
            args.extend([
                "--efi-boot".into(),
                rel.to_string_lossy().into_owned(),
                "--efi-boot-part".into(),
                "--efi-boot-image".into(),
            ]);
        }
    }

    // Also re-inject boot catalog if we found one
    if let Some(cat) = boot_cat {
        let rel = cat.strip_prefix(staging).unwrap_or(&cat);
        args.extend(["-c".into(), rel.to_string_lossy().into_owned()]);
    }

    args.extend([
        "-o".into(),
        output.to_str().unwrap().into(),
        staging.to_str().unwrap().into(),
    ]);

    term::info("Running xorriso to build ISO…");
    helpers::h::run(
        "iso-build",
        Command::new(consts::XORRISO)
            .args(&args)
            .stdout(Stdio::null()),
    )?;

    term::ok(&format!("ISO written to {}", output.display()));
    Ok(())
}

fn find_efi_image(staging: &Path) -> Option<PathBuf> {
    // Common EFI boot image paths in Windows ISOs
    let candidates = [
        "efi/microsoft/boot/efisys.bin",
        "EFI/Microsoft/Boot/efisys.bin",
        "efi/boot/bootx64.efi",
        "EFI/Boot/bootx64.efi",
    ];
    for c in &candidates {
        let p = staging.join(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}
