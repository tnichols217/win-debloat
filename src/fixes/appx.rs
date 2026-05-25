// AppX removal — delete provisioned package dirs from mounted WIM
use crate::{h, term};
use anyhow::Result;
use std::{fs, path::Path};

/// Patterns translated directly from the PS script's $appxPatternsToRemove.
const APPX_PATTERNS: &[&str] = &[
    "Microsoft.Microsoft3DViewer",
    "Microsoft.WindowsAlarms",
    "Microsoft.BingNews",
    "Microsoft.BingSearch",
    "Microsoft.BingWeather",
    "Windows.CBSPreview",
    "Clipchamp.Clipchamp",
    "Microsoft.549981C3F5F10", // Cortana
    "MicrosoftWindows.CrossDevice",
    "Microsoft.Windows.DevHome",
    "MicrosoftCorporationII.MicrosoftFamily",
    "Microsoft.WindowsFeedbackHub",
    "Microsoft.GetHelp",
    "Microsoft.Getstarted",
    "Microsoft.WindowsCommunicationsapps",
    "Microsoft.WindowsMaps",
    "Microsoft.MixedReality.Portal",
    "Microsoft.ZuneMusic",
    "Microsoft.MicrosoftOfficeHub",
    "Microsoft.Office.OneNote",
    "Microsoft.OutlookForWindows",
    "Microsoft.MSPaint",
    "Microsoft.People",
    "Microsoft.Windows.PeopleExperienceHost",
    "Microsoft.YourPhone",
    "Microsoft.PowerAutomateDesktop",
    "MicrosoftCorporationII.QuickAssist",
    "Microsoft.SkypeApp",
    "Microsoft.MicrosoftStickyNotes",
    "Microsoft.MicrosoftSolitaireCollection",
    "MicrosoftTeams",
    "MSTeams",
    "Microsoft.Windows.Teams",
    "Microsoft.Todos",
    "Microsoft.ZuneVideo",
    "Microsoft.Wallet",
    "Microsoft.GamingApp",
    "Microsoft.XboxApp",
    "Microsoft.XboxGameOverlay",
    "Microsoft.XboxGamingOverlay",
    "Microsoft.XboxSpeechToTextOverlay",
    "Microsoft.Xbox.TCUI",
];

/// AppX packages live in several locations inside the mounted WIM:
///   Windows/SystemApps/<PackageName>_<publisher>/
///   Program Files/WindowsApps/<PackageName>_<version>_<arch>__<publisher>/
///   ProgramData/Microsoft/Windows/AppRepository/Packages/<PackageName>_*/
pub fn remove_appx(mount: &Path) -> Result<()> {
    term::section("Removing AppX packages");

    let search_dirs = [
        mount.join("Windows/SystemApps"),
        mount.join("Program Files/WindowsApps"),
        mount.join("ProgramData/Microsoft/Windows/AppRepository/Packages"),
    ];

    let mut total = 0u32;

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(rd) = fs::read_dir(dir) else { continue };
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            for pat in APPX_PATTERNS {
                // Package dirs start with the pattern name
                if name.to_lowercase().starts_with(&pat.to_lowercase()) {
                    term::info(&format!("  Removing {}", name));
                    h::remove_path(&entry.path());
                    total += 1;
                    break;
                }
            }
        }
    }

    term::ok(&format!("{total} AppX package directories removed"));
    Ok(())
}
