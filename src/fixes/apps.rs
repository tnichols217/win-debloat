/// Remove windows packages from the WIM file
use crate::{h, registry, term};
use anyhow::Result;
use std::{fs, path::Path};

pub fn remove_capabilities(mount: &Path, lang: &str) -> Result<()> {
    term::section("Removing Windows Capabilities & Packages");

    // IE
    h::remove_path(&mount.join("Program Files/Internet Explorer"));
    h::remove_path(&mount.join("Program Files (x86)/Internet Explorer"));

    // WordPad
    h::remove_path(&mount.join("Program Files/Windows NT/Accessories/wordpad.exe"));
    h::remove_path(&mount.join("Program Files/Windows NT/Accessories/wordpadfilter.dll"));

    // Steps Recorder
    h::remove_path(&mount.join("Windows/System32/psr.exe"));

    // Math Recognizer / Handwriting
    h::remove_path(&mount.join("Windows/System32/InkObj.dll"));
    h::remove_path(&mount.join("Windows/System32/InputPersonalization.exe"));

    // Legacy Windows Media Player
    h::remove_path(&mount.join("Program Files/Windows Media Player"));
    h::remove_path(&mount.join("Program Files (x86)/Windows Media Player"));

    // Language-specific speech / OCR / TTS / handwriting packs
    // These live under Windows/System32/<lang>/ as MUI packages
    let lang_dirs = [mount.join(format!("Windows/System32/{lang}"))];
    // Only nuke the known speech/OCR/handwriting MUI files, not the whole lang dir
    let speech_files = ["Speech", "SpeechUX", "SpeechSynthesis", "HandwritingRec"];
    for ld in &lang_dirs {
        if !ld.exists() {
            continue;
        }
        for sf in &speech_files {
            for ext in &["dll", "exe", "mui"] {
                h::remove_path(&ld.join(format!("{sf}.{ext}")));
            }
        }
    }

    // PowerShell ISE
    h::remove_path(&mount.join("Windows/System32/WindowsPowerShell/v1.0/PowerShell_ISE.exe"));
    h::remove_path(&mount.join("Windows/SysWOW64/WindowsPowerShell/v1.0/PowerShell_ISE.exe"));

    term::ok("Capabilities removed");
    Ok(())
}

// OneDrive
pub fn remove_onedrive(mount: &Path) -> Result<()> {
    term::section("Removing OneDrive");

    // Primary setup binary (SysWOW64 variant; System32 one is typically a stub)
    h::remove_path(&mount.join("Windows/SysWOW64/OneDriveSetup.exe"));

    // Default user shortcut
    h::remove_path(
        &mount.join(
            "Users/Default/AppData/Roaming/Microsoft/Windows/Start Menu/Programs/OneDrive.lnk",
        ),
    );

    // AppX provisioned package dirs
    for dir in h::find_matching_dirs(
        &mount.join("Windows/SystemApps"),
        &["microsoft.microsoftskydrive*", "microsoft.onedrive*"],
    ) {
        h::remove_path(&dir);
    }

    term::ok("OneDrive removed");
    Ok(())
}

// Edge
pub fn remove_edge(mount: &Path) -> Result<()> {
    term::section("Removing Microsoft Edge");

    // Program Files dirs
    for base in &["Program Files", "Program Files (x86)"] {
        let ms = mount.join(base).join("Microsoft");
        for name in &["Edge", "EdgeCore", "EdgeUpdate", "EdgeWebView"] {
            h::remove_path(&ms.join(name));
        }
    }

    // ProgramData
    h::remove_path(&mount.join("ProgramData/Microsoft/EdgeUpdate"));

    // AppRepository entries
    let repo = mount.join("ProgramData/Microsoft/Windows/AppRepository/Packages");
    for dir in h::find_matching_dirs(
        &repo,
        &[
            "microsoft.microsoftedge.stable*",
            "microsoft.microsoftedgedevtoolsclient*",
        ],
    ) {
        h::remove_path(&dir);
    }

    // WebView
    h::remove_path(&mount.join("Windows/System32/Microsoft-Edge-WebView"));

    // Tasks
    let tasks = mount.join("Windows/System32/Tasks");
    for entry in fs::read_dir(&tasks).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with("microsoftedge") {
            h::remove_path(&entry.path());
        }
    }

    // Legacy Edge (Windows 10)
    for dir in h::find_matching_dirs(
        &mount.join("Windows/SystemApps"),
        &["microsoft.microsoftedge_*"],
    ) {
        h::remove_path(&dir);
    }

    // Registry tweaks via hivex
    term::info("Applying Edge registry tweaks…");
    registry::apply_edge_registry_tweaks(mount)?;

    term::ok("Microsoft Edge removed");
    Ok(())
}
