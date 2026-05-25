/// Removes windows capabilities from the iso
use crate::{h, term};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Capabilities to remove fromo windows
const CAPABILITY_PATTERNS: &[&str] = &[
    "Microsoft-Windows-InternetExplorer-Optional-Package",
    "Microsoft-Windows-StepsRecorder-Package",
    "Microsoft-Windows-WordPad-FoD-Package",
    "Microsoft-Windows-TabletPCMath-Package",
    "Microsoft-Windows-MediaPlayer-Package",
    "Microsoft-Windows-Wallpaper-Content-Extended-FoD-Package",
    // Language-specific; LANG substituted below
    "Microsoft-Windows-LanguageFeatures-Handwriting-LANG-Package",
    "Microsoft-Windows-LanguageFeatures-OCR-LANG-Package",
    "Microsoft-Windows-LanguageFeatures-Speech-LANG-Package",
    "Microsoft-Windows-LanguageFeatures-TextToSpeech-LANG-Package",
];

pub fn remove_capabilities(mount: &Path, lang: &str) -> Result<()> {
    term::section("Removing Windows Capabilities & Packages");

    // Substitute the detected language code into patterns that need it.
    let patterns: Vec<String> = CAPABILITY_PATTERNS
        .iter()
        .map(|p| p.replace("LANG", lang))
        .collect();

    let packages_dir = mount.join("Windows/servicing/Packages");

    if !packages_dir.exists() {
        term::warn("Windows/servicing/Packages not found — skipping capability removal");
        return Ok(());
    }

    // Collect all .mum files that match our patterns
    let matched = find_matching_mums(&packages_dir, &patterns)?;

    if matched.is_empty() {
        term::info("No matching capability manifests found");
        return Ok(());
    }

    term::info(&format!("Found {} matching manifests", matched.len()));

    let mut removed_manifests = 0u32;

    for mum_path in &matched {
        // Delete the .mum
        h::remove_path(mum_path);
        removed_manifests += 1;

        // Delete the matching .cat (same stem, different extension)
        let cat_path = mum_path.with_extension("cat");
        h::remove_path(&cat_path);
    }

    term::ok(&format!("Removed {} manifests", removed_manifests));
    Ok(())
}

/// Find .mum files whose stem matches any of the given patterns
fn find_matching_mums(packages_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut matched = Vec::new();

    for entry in fs::read_dir(packages_dir)?.flatten() {
        let path = entry.path();

        // Only .mum files
        if path.extension().and_then(|e| e.to_str()) != Some("mum") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        for pattern in patterns {
            if stem.contains(&pattern.to_lowercase()) {
                matched.push(path);
                break;
            }
        }
    }

    Ok(matched)
}
