use crate::{consts, h, term};
use anyhow::Result;
use std::{io::Write, path::Path, process::Command};

pub enum HiveOp {
    DeleteKey(String),
    DeleteValue(String, String),
    SetDword(String, String, u32),
    SetString(String, String, String),
    SetMultiString(String, String, Vec<String>),
}

// reg_ops! macro
//
//   -"Key\\Path";                                    → DeleteKey
//   -"Key\\Path", "ValueName";                       → DeleteValue
//   "Key\\Path", "Name" => dword:N;                  → SetDword
//   "Key\\Path", "Name" => sz:"text";                → SetString
//   "Key\\Path", "Name" => multi:["a", "b"];         → SetMultiString

#[macro_export]
macro_rules! reg_ops {
    // DeleteKey: -"path";
    ( - $path:expr ; $( $tail:tt )* ) => {{
        let mut ops = vec![ $crate::registry::HiveOp::DeleteKey($path.to_string()) ];
        ops.extend($crate::reg_ops!( $( $tail )* ));
        ops
    }};

    // DeleteValue: -"path", "name";
    ( - $path:expr, $name:expr ; $( $tail:tt )* ) => {{
        let mut ops = vec![ $crate::registry::HiveOp::DeleteValue($path.to_string(), $name.to_string()) ];
        ops.extend($crate::reg_ops!( $( $tail )* ));
        ops
    }};

    // SetDword: "path", "name" => dword:N;
    ( $path:expr, $name:expr => dword : $val:expr ; $( $tail:tt )* ) => {{
        let mut ops = vec![ $crate::registry::HiveOp::SetDword($path.to_string(), $name.to_string(), $val) ];
        ops.extend($crate::reg_ops!( $( $tail )* ));
        ops
    }};

    // SetString: "path", "name" => sz:"value";
    ( $path:expr, $name:expr => sz : $val:expr ; $( $tail:tt )* ) => {{
        let mut ops = vec![ $crate::registry::HiveOp::SetString($path.to_string(), $name.to_string(), $val.to_string()) ];
        ops.extend($crate::reg_ops!( $( $tail )* ));
        ops
    }};

    // SetMultiString: "path", "name" => multi:["a", "b"];
    ( $path:expr, $name:expr => multi : [ $( $val:expr ),* ] ; $( $tail:tt )* ) => {{
        let vals: Vec<String> = vec![ $( $val.to_string() ),* ];
        let mut ops = vec![ $crate::registry::HiveOp::SetMultiString($path.to_string(), $name.to_string(), vals) ];
        ops.extend($crate::reg_ops!( $( $tail )* ));
        ops
    }};

    // Base case
    () => { vec![] };
}

fn hivex_edit(hive_path: &Path, ops: &[HiveOp]) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }

    let mut script = String::from("Windows Registry Editor Version 5.00\n");

    for op in ops {
        match op {
            HiveOp::DeleteKey(path) => {
                script.push_str(&format!("\n[-\\{}]\n", path));
            }

            HiveOp::DeleteValue(path, name) => {
                script.push_str(&format!("\n[\\{}]\n", path));
                script.push_str(&format!("\"{}\"=-\n", name));
            }

            HiveOp::SetDword(path, name, value) => {
                let mut current_subpath = String::new();
                for part in path.split('\\') {
                    if !part.is_empty() {
                        if !current_subpath.is_empty() {
                            current_subpath.push('\\');
                        }
                        current_subpath.push_str(part);
                        script.push_str(&format!("\n[\\{}]\n", current_subpath));
                    }
                }
                script.push_str(&format!("\"{}\"=dword:{:08x}\n\n", name, value));
            }

            HiveOp::SetString(path, name, value) => {
                let mut current_subpath = String::new();
                for part in path.split('\\') {
                    if !part.is_empty() {
                        if !current_subpath.is_empty() {
                            current_subpath.push('\\');
                        }
                        current_subpath.push_str(part);
                        script.push_str(&format!("\n[\\{}]\n", current_subpath));
                    }
                }
                let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
                script.push_str(&format!("\"{}\"=\"{}\"\n", name, escaped));
            }

            HiveOp::SetMultiString(path, name, values) => {
                let mut current_subpath = String::new();
                for part in path.split('\\') {
                    if !part.is_empty() {
                        if !current_subpath.is_empty() {
                            current_subpath.push('\\');
                        }
                        current_subpath.push_str(part);
                        script.push_str(&format!("\n[\\{}]\n", current_subpath));
                    }
                }
                let mut bytes: Vec<u8> = Vec::new();
                for s in values {
                    for c in s.encode_utf16() {
                        bytes.extend_from_slice(&c.to_le_bytes());
                    }
                    bytes.extend_from_slice(&[0x00, 0x00]);
                }
                bytes.extend_from_slice(&[0x00, 0x00]);
                let hex = bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(",");
                script.push_str(&format!("\"{}\"=hex(7):{}\n", name, hex));
            }
        }
    }

    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(script.as_bytes())?;
    tmp.flush()?;

    h::run(
        "hivexregedit",
        Command::new(consts::HIVEXREG)
            .arg("--merge")
            .arg(hive_path)
            .arg(tmp.path()),
    )
}

pub fn apply_edge_registry_tweaks(mount: &Path) -> Result<()> {
    let software = mount.join("Windows/System32/config/SOFTWARE");
    let system = mount.join("Windows/System32/config/SYSTEM");
    let default = mount.join("Windows/System32/config/DEFAULT");
    let ntuser = mount.join("Users/Default/NTUSER.DAT");

    hivex_edit(
        &software,
        &reg_ops! {
            -"Microsoft\\EdgeUpdate";
            -"Microsoft\\Windows\\CurrentVersion\\Uninstall\\Microsoft Edge";
            -"Microsoft\\Active Setup\\Installed Components\\{9459C573-B17A-45AE-9F64-1857B5D58CEE}";
            -"WOW6432Node\\Microsoft\\Edge";
            -"WOW6432Node\\Microsoft\\EdgeUpdate";
            -"WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Microsoft Edge";
            -"WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Microsoft Edge Update";

            "Microsoft\\MicrosoftEdge\\Main", "AllowPrelaunch" => dword:1;
            "Policies\\Microsoft\\MicrosoftEdge\\Main", "AllowPrelaunch" => dword:1;
            "Microsoft\\MicrosoftEdge\\TabPreloader", "AllowTabPreloading" => dword:1;
            "Policies\\Microsoft\\MicrosoftEdge\\TabPreloader", "AllowTabPreloading" => dword:1;
            "Policies\\Microsoft\\EdgeUpdate", "UpdateDefault" => dword:0;

            "Microsoft\\EdgeUpdate", "DoNotUpdateToEdgeWithChromium" => dword:1;
            "Microsoft\\EdgeUpdate", "UpdaterExperimentationAndConfigurationServiceControl" => dword:1;
            "Microsoft\\EdgeUpdate", "InstallDefault" => dword:1;
            "Policies\\Microsoft\\EdgeUpdate", "DoNotUpdateToEdgeWithChromium" => dword:1;
            "Policies\\Microsoft\\EdgeUpdate", "UpdaterExperimentationAndConfigurationServiceControl" => dword:1;
            "Policies\\Microsoft\\EdgeUpdate", "InstallDefault" => dword:1;
            "WOW6432Node\\Microsoft\\EdgeUpdate", "DoNotUpdateToEdgeWithChromium" => dword:1;
            "WOW6432Node\\Microsoft\\EdgeUpdate", "UpdaterExperimentationAndConfigurationServiceControl" => dword:1;
            "WOW6432Node\\Microsoft\\EdgeUpdate", "InstallDefault" => dword:1;
        },
    )?;

    hivex_edit(
        &system,
        &reg_ops! {
            -"CurrentControlSet\\Services\\edgeupdate";
            -"CurrentControlSet\\Services\\edgeupdatem";
            -"ControlSet001\\Services\\edgeupdate";
            -"ControlSet001\\Services\\edgeupdatem";
        },
    )?;

    hivex_edit(
        &default,
        &reg_ops! {
            -"Software\\Microsoft\\EdgeUpdate";
        },
    )?;

    hivex_edit(
        &ntuser,
        &reg_ops! {
            -"Software\\Microsoft\\EdgeUpdate";

            "Software\\Microsoft\\MicrosoftEdge\\Main", "AllowPrelaunch" => dword:1;
            "Software\\Policies\\Microsoft\\MicrosoftEdge\\Main", "AllowPrelaunch" => dword:1;
            "Software\\Microsoft\\MicrosoftEdge\\TabPreloader", "AllowTabPreloading" => dword:1;
            "Software\\Policies\\Microsoft\\MicrosoftEdge\\TabPreloader", "AllowTabPreloading" => dword:1;

            "Software\\Microsoft\\EdgeUpdate", "DoNotUpdateToEdgeWithChromium" => dword:1;
            "Software\\Microsoft\\EdgeUpdate", "UpdaterExperimentationAndConfigurationServiceControl" => dword:1;
            "Software\\Microsoft\\EdgeUpdate", "InstallDefault" => dword:1;
            "Software\\Policies\\Microsoft\\EdgeUpdate", "DoNotUpdateToEdgeWithChromium" => dword:1;
            "Software\\Policies\\Microsoft\\EdgeUpdate", "UpdaterExperimentationAndConfigurationServiceControl" => dword:1;
            "Software\\Policies\\Microsoft\\EdgeUpdate", "InstallDefault" => dword:1;
        },
    )?;

    Ok(())
}

// AI / Copilot / Recall
pub fn apply_ai_registry_tweaks(mount: &Path) -> Result<()> {
    let software = mount.join("Windows/System32/config/SOFTWARE");
    let system = mount.join("Windows/System32/config/SYSTEM");
    let ntuser = mount.join("Users/Default/NTUSER.DAT");

    hivex_edit(
        &software,
        &reg_ops! {
            // Search
            "Policies\\Microsoft\\Windows\\Explorer", "DisableSearchBoxSuggestions" => dword:1;

            // Notepad AI
            "Policies\\WindowsNotepad", "DisableAIFeatures" => dword:1;

            // Paint AI
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Paint", "DisableCocreator" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Paint", "DisableImageCreator" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Paint", "DisableGenerativeFill" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Paint", "DisableGenerativeErase" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Paint", "DisableRemoveBackground" => dword:1;

            // App AI model access
            "Policies\\Microsoft\\Windows\\AppPrivacy", "LetAppsAccessSystemAIModels" => dword:2;
            "Policies\\Microsoft\\Windows\\AppPrivacy", "LetAppsAccessGenerativeAI" => dword:2;

            // Capability access manager (REG_SZ "Deny")
            "Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\generativeAI", "Value" => sz:"Deny";
            "Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\systemAIModels", "Value" => sz:"Deny";

            // Edge AI policies
            "Policies\\Microsoft\\Edge", "HubsSidebarEnabled" => dword:0;
            "Policies\\Microsoft\\Edge", "CopilotPageContext" => dword:0;
            "Policies\\Microsoft\\Edge", "CopilotCDPPageContext" => dword:0;
            "Policies\\Microsoft\\Edge", "EdgeHistoryAISearchEnabled" => dword:0;
            "Policies\\Microsoft\\Edge", "BuiltInAIAPIsEnabled" => dword:0;
            "Policies\\Microsoft\\Edge", "AIGenThemesEnabled" => dword:0;
            "Policies\\Microsoft\\Edge", "ShareBrowsingHistoryWithCopilotSearchAllowed" => dword:0;

            // Copilot / Recall / WindowsAI system-wide
            "Policies\\Microsoft\\Windows\\WindowsCopilot", "TurnOffWindowsCopilot" => dword:1;
            "Policies\\Microsoft\\Windows\\WindowsAI", "DisableAIDataAnalysis" => dword:1;
            "Policies\\Microsoft\\Windows\\WindowsAI", "AllowRecallEnablement" => dword:0;
            "Policies\\Microsoft\\Windows\\WindowsAI", "TurnOffSavingSnapshots" => dword:1;
            "Policies\\Microsoft\\Windows\\WindowsAI", "DisableSettingsAgent" => dword:1;
            "Policies\\Microsoft\\Windows\\WindowsAI", "DisableClickToDo" => dword:1;

            // Shell Copilot state
            "Microsoft\\Windows\\Shell\\Copilot", "IsCopilotAvailable" => dword:0;
            "Microsoft\\Windows\\Shell\\Copilot", "CopilotDisabledReason" => sz:"FeatureIsDisabled";

            // Hide AI from Settings page
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer", "SettingsPageVisibility" => sz:"hide:aicomponents";

            // Prevent Copilot auto-open on large screens
            "Microsoft\\Windows\\CurrentVersion\\Notifications\\Settings", "AutoOpenCopilotLargeScreens" => dword:0;

            // Prevent Copilot Appx reinstall via Store
            "Policies\\Microsoft\\Windows\\Appx\\RemoveDefaultMicrosoftStorePackages", "Enabled" => dword:1;
            "Policies\\Microsoft\\Windows\\Appx\\RemoveDefaultMicrosoftStorePackages\\Microsoft.Copilot_8wekyb3d8bbwe", "RemovePackage" => dword:1;

            // Disable WSAIFabricSvc + Recall on first logon
            "Microsoft\\Windows\\CurrentVersion\\RunOnce", "DisableWSAIFabricSvc" => sz:"reg add \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\WSAIFabricSvc\" /v \"Start\" /t REG_DWORD /d \"4\" /f";
            "Microsoft\\Windows\\CurrentVersion\\RunOnce", "StopWSAIFabricSvc" => sz:"net stop WSAIFabricSvc";
            "Microsoft\\Windows\\CurrentVersion\\RunOnce", "DisableRecall" => sz:"dism.exe /online /disable-feature /FeatureName:recall";

            // Delete WindowsAI scheduled task tree entry
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\WindowsAI";
        },
    )?;

    hivex_edit(
        &system,
        &reg_ops! {
            "ControlSet001\\Services\\WSAIFabricSvc", "Start" => dword:4;
        },
    )?;

    hivex_edit(
        &ntuser,
        &reg_ops! {
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced", "ShowCopilotButton" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced", "Start_AccountNotifications" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Taskband\\AuxilliaryPins", "CopilotPWAPin" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Taskband\\AuxilliaryPins", "RecallPin" => dword:0;

            // Copilot runtime
            "Software\\Microsoft\\Windows\\CurrentVersion\\WindowsCopilot", "AllowCopilotRuntime" => dword:0;

            // Prevent Copilot PWA auto-reinstall
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\AutoInstalledPWAs", "CopilotPWAPreinstallCompleted" => dword:1;

            // Block Copilot background access
            "Software\\Microsoft\\Windows\\CurrentVersion\\BackgroundAccessApplications\\Microsoft.Copilot_8wekyb3d8bbwe", "Disabled" => dword:1;
            "Software\\Microsoft\\Windows\\CurrentVersion\\BackgroundAccessApplications\\Microsoft.Copilot_8wekyb3d8bbwe", "DisabledByUser" => dword:1;

            // Remove Ask Copilot from right-click context menu
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Shell Extensions\\Blocked", "{CB3B0003-8088-4EDE-8769-8B354AB2FF8C}" => sz:"Ask Copilot";

            // Per-user Copilot / Recall / WindowsAI policies
            "Software\\Policies\\Microsoft\\Windows\\WindowsCopilot", "TurnOffWindowsCopilot" => dword:1;
            "Software\\Policies\\Microsoft\\Windows\\WindowsAI", "DisableAIDataAnalysis" => dword:1;
            "Software\\Policies\\Microsoft\\Windows\\WindowsAI", "AllowRecallEnablement" => dword:0;
            "Software\\Policies\\Microsoft\\Windows\\WindowsAI", "TurnOffSavingSnapshots" => dword:1;
            "Software\\Policies\\Microsoft\\Windows\\WindowsAI", "DisableSettingsAgent" => dword:1;
            "Software\\Policies\\Microsoft\\Windows\\WindowsAI", "DisableClickToDo" => dword:1;

            // Per-user shell Copilot state
            "Software\\Microsoft\\Windows\\Shell\\Copilot", "IsCopilotAvailable" => dword:0;
            "Software\\Microsoft\\Windows\\Shell\\Copilot", "CopilotDisabledReason" => sz:"FeatureIsDisabled";

            // Office Copilot
            "Software\\Microsoft\\Office\\16.0\\Word\\Options", "EnableCopilot" => dword:0;
            "Software\\Microsoft\\Office\\16.0\\Excel\\Options", "EnableCopilot" => dword:0;
            "Software\\Microsoft\\Office\\16.0\\OneNote\\Options\\Copilot", "CopilotEnabled" => dword:0;
            "Software\\Policies\\Microsoft\\office\\16.0\\common\\privacy", "controllerconnectedservicesenabled" => dword:2;
        },
    )?;

    Ok(())
}

pub fn apply_cbs_ai_tweaks(mount: &Path) -> Result<()> {
    let software = mount.join("Windows/System32/config/SOFTWARE");
    let packages_dir = mount.join("Windows/servicing/Packages");

    if !packages_dir.exists() {
        return Ok(());
    }

    let ai_patterns = ["AIX", "Recall", "Copilot", "CoreAI"];
    let cbs_base = "Microsoft\\Windows\\CurrentVersion\\Component Based Servicing\\Packages";

    let matching: Vec<String> = std::fs::read_dir(&packages_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".mum") && ai_patterns.iter().any(|p| name.contains(p)) {
                Some(name.trim_end_matches(".mum").to_string())
            } else {
                None
            }
        })
        .collect();

    if matching.is_empty() {
        return Ok(());
    }

    let mut ops: Vec<HiveOp> = Vec::new();
    for pkg in &matching {
        let key = format!("{}\\{}", cbs_base, pkg);
        ops.extend(reg_ops! {
            key, "Visibility" => dword:1;
            key, "DefVis"     => dword:2;
            -format!("{}\\Owners",  key);
            -format!("{}\\Updates", key);
        });
    }

    hivex_edit(&software, &ops)?;
    term::info(&format!(
        "CBS AI visibility tweaks applied to {} packages",
        matching.len()
    ));
    Ok(())
}

pub fn apply_privacy_tweaks(mount: &Path) -> Result<()> {
    term::section("Applying privacy & performance tweaks");

    let software = mount.join("Windows/System32/config/SOFTWARE");
    let system = mount.join("Windows/System32/config/SYSTEM");
    let ntuser = mount.join("Users/Default/NTUSER.DAT");

    hivex_edit(
        &software,
        &reg_ops! {
            // Telemetry
            "Policies\\Microsoft\\Windows\\DataCollection", "AllowTelemetry" => dword:0;

            // Sponsored / consumer apps
            "Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "OemPreInstalledAppsEnabled" => dword:0;
            "Policies\\Microsoft\\Windows\\CloudContent", "DisableWindowsConsumerFeatures" => dword:1;
            "Policies\\Microsoft\\Windows\\CloudContent", "DisableConsumerAccountStateContent" => dword:1;
            "Policies\\Microsoft\\Windows\\CloudContent", "DisableCloudOptimizedContent" => dword:1;
            "Policies\\Microsoft\\Windows\\CloudContent", "DisableSoftLanding" => dword:1;

            // Start pins — empty pinned list
            "Microsoft\\PolicyManager\\current\\device\\Start", "ConfigureStartPins" => sz:"{\"pinnedList\": [{}]}";

            // Advertising ID
            "Policies\\Microsoft\\Windows\\AdvertisingInfo", "DisabledByGroupPolicy" => dword:1;

            // Meet Now
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer", "HideSCAMeetNow" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer", "AllowOnlineTips" => dword:0;

            // News and Interests
            "Policies\\Microsoft\\Windows\\Windows Feeds", "EnableFeeds" => dword:0;

            // Cortana
            "Policies\\Microsoft\\Windows\\Windows Search", "AllowCortana" => dword:0;

            // MRT
            "Policies\\Microsoft\\MRT", "DontOfferThroughWUAU" => dword:1;

            // Teams auto-install
            "Policies\\Microsoft\\Teams", "DisableInstallation" => dword:1;

            // Outlook / Mail
            "Policies\\Microsoft\\Windows\\Windows Mail", "PreventRun" => dword:1;

            // BitLocker auto-encryption
            "Microsoft\\BitLocker", "PreventDeviceEncryption" => dword:1;

            // OneDrive
            "Policies\\Microsoft\\Windows\\OneDrive", "DisableLibrariesDefaultSaveToOneDrive" => dword:0;
            "Policies\\Microsoft\\Windows\\OneDrive", "DisableFileSyncNGSC" => dword:1;
            "Policies\\Microsoft\\OneDrive", "KFMBlockOptIn" => dword:1;

            // GameDVR
            "Policies\\Microsoft\\Windows\\GameDVR", "AllowGameDVR" => dword:0;

            // DevHome update prevention
            -"Microsoft\\WindowsUpdate\\Orchestrator\\UScheduler_Oobe\\DevHomeUpdate";
            "Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Orchestrator\\UScheduler\\DevHomeUpdate", "workCompleted" => dword:1;

            // Outlook update prevention
            -"Microsoft\\WindowsUpdate\\Orchestrator\\UScheduler_Oobe\\OutlookUpdate";
            "Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Orchestrator\\UScheduler\\OutlookUpdate", "workCompleted" => dword:1;

            // Chat (Teams) auto-install
            "Microsoft\\Windows\\CurrentVersion\\Communications", "ConfigureChatAutoInstall" => dword:0;
            "Policies\\Microsoft\\Windows\\Windows Chat", "ChatIcon" => dword:3;

            // OOBE
            "Policies\\Microsoft\\Windows\\OOBE", "DisablePrivacyExperience" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\OOBE", "BypassNRO" => dword:1;
            "Microsoft\\Windows\\CurrentVersion\\OOBE", "BypassNROGatherOptions" => dword:1;

            // Scheduled task tree
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Application Experience\\PcaPatchDbTask";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Application Experience\\MareBackup";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser Exp";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Autochk\\Proxy";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Customer Experience Improvement Program\\KernelCeipTask";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip";
            -"Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tree\\Microsoft\\Windows\\Customer Experience Improvement Program";
        },
    )?;

    hivex_edit(
        &system,
        &reg_ops! {
            "ControlSet001\\Services\\dmwappushservice", "Start" => dword:4;
            "ControlSet001\\Services\\BcastDVRUserService", "Start" => dword:4;
            "ControlSet001\\Services\\GameBarPresenceWriter", "Start" => dword:4;
        },
    )?;

    hivex_edit(
        &ntuser,
        &reg_ops! {
            // Content delivery / sponsored apps
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "PreInstalledAppsEnabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SilentInstalledAppsEnabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContentEnabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "ContentDeliveryAllowed" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "PreInstalledAppsEverEnabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SoftLandingEnabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SystemPaneSuggestionsEnabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-310093Enabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-338387Enabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-338388Enabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-338389Enabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-338393Enabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-353694Enabled" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager", "SubscribedContent-353696Enabled" => dword:0;
            -"Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager\\Subscriptions";
            -"Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager\\SuggestedApps";

            // Telemetry / privacy
            "Software\\Microsoft\\Personalization\\Settings", "AcceptedPrivacyPolicy" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\Privacy", "TailoredExperiencesWithDiagnosticDataEnabled" => dword:0;
            "Software\\Microsoft\\Speech_OneCore\\Settings\\OnlineSpeechPrivacy", "HasAccepted" => dword:0;
            "Software\\Microsoft\\InputPersonalization", "RestrictImplicitInkCollection" => dword:1;
            "Software\\Microsoft\\InputPersonalization", "RestrictImplicitTextCollection" => dword:1;
            "Software\\Microsoft\\InputPersonalization\\TrainedDataStore", "HarvestContacts" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\AdvertisingInfo", "Enabled" => dword:0;

            // Explorer / Start
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced", "ShowTaskViewButton" => dword:0;
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced", "Start_IrisRecommendations" => dword:0;

            // Spotlight desktop icon
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\HideDesktopIcons\\NewStartPanel", "{2cc5ca98-6485-489a-920e-b3e88a6ccce3}" => dword:1;

            // Mouse acceleration
            "Control Panel\\Mouse", "MouseSpeed" => sz:"0";
            "Control Panel\\Mouse", "MouseThreshold1" => sz:"0";
            "Control Panel\\Mouse", "MouseThreshold2" => sz:"0";

            // Menu show delay
            "Control Panel\\Desktop", "MenuShowDelay" => sz:"200";

            // GameDVR
            "Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR", "AppCaptureEnabled" => dword:0;
            "System\\GameConfigStore", "GameDVR_Enabled" => dword:0;

            // GameBar
            "Software\\Microsoft\\GameBar", "AutoGameModeEnabled" => dword:0;

            // OneDrive run key
            -"Software\\Microsoft\\Windows\\CurrentVersion\\Run", "OneDriveSetup";
        },
    )?;

    term::ok("Privacy & performance tweaks applied");
    Ok(())
}

pub fn apply_task_tweaks(mount: &Path) -> Result<()> {
    let software = mount.join("Windows/System32/config/SOFTWARE");

    // Customer Experience Improvement Program, Program Data Updater, Application Compatibility Appraiser
    let task_guids: &[&str] = &[
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{780E487D-C62F-4B55-AF84-0E38116AFE07}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{FD607F42-4541-418A-B812-05C32EBA8626}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{E4FED5BC-D567-4044-9642-2EDADF7DE108}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{E292525C-72F1-482C-8F35-C513FAA98DAE}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{3047C197-66F1-4523-BA92-6C955FEF9E4E}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{A0C71CB8-E8F0-498A-901D-4EDA09E07FF4}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{4738DE7A-BCC1-4E2D-B1B0-CADB044BFA81}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{6FAC31FA-4A85-4E64-BFD5-2154FF4594B3}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{FC931F16-B50A-472E-B061-B6F79A71EF59}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{0671EB05-7D95-4153-A32B-1426B9FE61DB}",
        "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache\\Tasks\\{0600DD45-FAF2-4131-A006-0B17509B9F78}",
    ];

    let ops: Vec<HiveOp> = task_guids
        .iter()
        .map(|k| HiveOp::DeleteKey(k.to_string()))
        .collect();

    hivex_edit(&software, &ops)
}

pub fn apply_tpm_bypass_tweaks(mount: &Path) -> Result<()> {
    let software = mount.join("Windows/System32/config/SOFTWARE");
    let system = mount.join("Windows/System32/config/SYSTEM");
    let default = mount.join("Windows/System32/config/DEFAULT");
    let ntuser = mount.join("Users/Default/NTUSER.DAT");

    hivex_edit(
        &system,
        &reg_ops! {
            "Setup\\LabConfig", "BypassTPMCheck" => dword:1;
            "Setup\\LabConfig", "BypassSecureBootCheck" => dword:1;
            "Setup\\LabConfig", "BypassStorageCheck" => dword:1;
            "Setup\\LabConfig", "BypassCPUCheck" => dword:1;
            "Setup\\LabConfig", "BypassRAMCheck" => dword:1;
            "Setup\\LabConfig", "BypassDiskCheck" => dword:1;
            "Setup\\MoSetup", "AllowUpgradesWithUnsupportedTPMOrCPU" => dword:1;
        },
    )?;

    hivex_edit(
        &software,
        &reg_ops! {
            "Microsoft\\Windows\\CurrentVersion\\Policies\\System", "HideUnsupportedHardwareNotifications" => dword:1;
            -"Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\CompatMarkers";
            -"Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Shared";
            -"Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\TargetVersionUpgradeExperienceIndicators";
            "Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\HwReqChk", "HwReqChkVars" => multi:[
                "SQ_SecureBootCapable=TRUE",
                "SQ_SecureBootEnabled=TRUE",
                "SQ_TpmVersion=2",
                "SQ_RamMB=8192"
            ];
        },
    )?;

    hivex_edit(
        &default,
        &reg_ops! {
            "Control Panel\\UnsupportedHardwareNotificationCache", "SV1" => dword:0;
            "Control Panel\\UnsupportedHardwareNotificationCache", "SV2" => dword:0;
        },
    )?;

    hivex_edit(
        &ntuser,
        &reg_ops! {
            "Control Panel\\UnsupportedHardwareNotificationCache", "SV1" => dword:0;
            "Control Panel\\UnsupportedHardwareNotificationCache", "SV2" => dword:0;
            "Software\\Microsoft\\PCHC", "UpgradeEligibility" => dword:1;
        },
    )?;

    Ok(())
}

pub fn enable_user_folders(mount: &Path) -> Result<()> {
    term::section("Enabling user shell folders");

    let software = mount.join("Windows/System32/config/SOFTWARE");

    // GUIDs match what the PS script adds under MyComputer\NameSpace
    let folder_guids = [
        "{B4BFCC3A-DB2C-424C-B029-7FE99A87C641}", // Desktop
        "{d3162b92-9365-467a-956b-92703aca08af}", // Documents
        "{088e3905-0323-4b02-9826-5d99428e115f}", // Downloads
        "{3dfdf296-dbec-4fb4-81d1-6a3438bcf4de}", // Music
        "{24ad3ad4-a569-4530-98e1-ab02f9417aa8}", // Pictures
        "{f86fa3ab-70d2-4fc7-9c99-fcbf05467f3a}", // Videos
    ];

    let ops: Vec<HiveOp> = folder_guids
        .iter()
        .flat_map(|guid| {
            let key = format!(
                "Microsoft\\Windows\\CurrentVersion\\Explorer\\MyComputer\\NameSpace\\{}",
                guid
            );
            vec![
                HiveOp::SetDword(key.clone(), "HideIfEnabled".into(), 0),
                HiveOp::SetDword(key.clone(), "HiddenByDefault".into(), 0),
            ]
        })
        .collect();

    hivex_edit(&software, &ops)?;

    term::ok("User folders enabled");
    Ok(())
}

pub fn apply_gamebar_tweaks(mount: &Path) -> Result<()> {
    let software = mount.join("Windows/System32/config/SOFTWARE");
    let system = mount.join("Windows/System32/config/SYSTEM");

    let root = "SOFTWARE\\Classes\\";
    let shell_path = "\\shell\\open\\command";

    hivex_edit(&software, &["ms-gamebar", "ms-gamebarservices", "ms-gamingoverlay"]
        .iter()
        .flat_map(|scheme| {
            let base = format!("{}{}", root, scheme);
            let shell_cmd = format!("{}{}", base, shell_path);
            reg_ops! {
                base,      "NoOpenWith"   => sz:"";
                base,      "(Default)"    => sz:format!("URL:{}", scheme);
                base,      "URL Protocol" => sz:"";
                shell_cmd, "(Default)"    => sz:"%SystemRoot%\\System32\\systray.exe";
            }
        })
        .chain(reg_ops! {
            "Microsoft\\WindowsRuntime\\Server\\Windows.Gaming.GameBar.Internal.PresenceWriterServer",
            "ExePath" => sz:"%SystemRoot%\\System32\\systray.exe";

            "Microsoft\\WindowsRuntime\\ActivatableClassId\\Windows.Gaming.GameBar.PresenceServer.Internal.PresenceWriter",
            "ActivationType" => dword:0;
        })
        .collect::<Vec<HiveOp>>())?;

    // xbgm service — disable (Start=4)
    hivex_edit(
        &system,
        &reg_ops! {
            "ControlSet001\\Services\\xbgm", "Start" => dword:4;
        },
    )?;

    Ok(())
}
