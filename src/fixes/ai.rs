/// Removes AI components
use crate::{helpers, registry, term};
use anyhow::Result;
use std::{fs, path::Path};

const AI_APPX_PATTERNS: &[&str] = &[
    "Microsoft.Windows.Copilot",
    "Microsoft.Copilot",
    "MicrosoftWindows.Client.AIX",
    "MicrosoftWindows.Client.CoPilot",
    "MicrosoftWindows.Client.CoreAI",
    "Microsoft.Windows.Ai.Copilot.Provider",
    "Microsoft.Edge.GameAssist",
    "Microsoft.Office.ActionsServer",
    "Microsoft.WritingAssistant",
];

pub fn remove_ai(mount: &Path) -> Result<()> {
    term::section("Removing AI components");

    let search_dirs = [
        mount.join("Windows/SystemApps"),
        mount.join("Program Files/WindowsApps"),
        mount.join("ProgramData/Microsoft/Windows/AppRepository/Packages"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(rd) = fs::read_dir(dir) else { continue };
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            for pat in AI_APPX_PATTERNS {
                if name.to_lowercase().starts_with(&pat.to_lowercase()) {
                    term::info(&format!("  Removing {name}"));
                    helpers::h::remove_path(&entry.path());
                    break;
                }
            }
        }
    }

    // Registry tweaks
    registry::apply_ai_registry_tweaks(mount)?;
    registry::apply_cbs_ai_tweaks(mount)?;

    term::ok("AI components removed");
    Ok(())
}
