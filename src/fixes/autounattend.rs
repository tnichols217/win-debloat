/// Generates and writes `autounattend.xml` into the ISO staging root.
use crate::term;
use anyhow::Result;
use std::{fs, path::Path};
use xml_builder_macro::{types::XmlSegment, xml};

// Settings
pub struct UnattendSettings {
    // windowsPE
    /// Accept the EULA automatically (true = no EULA prompt)
    pub accept_eula: bool,
    /// Show dynamic update UI only on error, suppressing it during normal flow
    pub dynamic_update_on_error_only: bool,
    /// Disable network access during WinPE phase
    pub disable_network_in_pe: bool,
    /// Keep firewall enabled during WinPE phase
    pub enable_firewall_in_pe: bool,
    /// Show product key prompt during setup (false = skip key entry)
    pub show_product_key_ui: bool,

    // oobeSystem
    /// Skip the machine OOBE flow entirely
    pub skip_machine_oobe: bool,
    /// Skip the per-user OOBE flow
    pub skip_user_oobe: bool,
    /// Hide the EULA page in OOBE
    pub hide_eula_page: bool,
    /// Hide "sign in with Microsoft" screens — allows local account creation
    pub hide_online_account_screens: bool,
    /// Hide wireless setup during OOBE
    pub hide_wireless_setup: bool,
    /// Hide the local account screen (keep false — we want local accounts)
    pub hide_local_account_screen: bool,
    /// 1 = recommended settings, 2 = install only, 3 = don't configure
    pub protect_your_pc: u8,
}

impl Default for UnattendSettings {
    /// Skip as much of OOBE as possible while still bypassing the Microsoft
    /// account requirement and keeping the firewall up during PE.
    fn default() -> Self {
        Self {
            accept_eula: true,
            dynamic_update_on_error_only: true,
            disable_network_in_pe: true,
            enable_firewall_in_pe: true,
            show_product_key_ui: false,

            skip_machine_oobe: true,
            skip_user_oobe: true,
            hide_eula_page: true,
            hide_online_account_screens: true,
            hide_wireless_setup: true,
            hide_local_account_screen: false,
            protect_your_pc: 3,
        }
    }
}

/// Write an autoattend file to the iso with default settings
pub fn write_autounattend(staging_root: &Path, lang: &str, image: u32) -> Result<()> {
    write_autounattend_with(staging_root, lang, image, &UnattendSettings::default())
}

pub fn write_autounattend_with(
    staging_root: &Path,
    lang: &str,
    image: u32,
    s: &UnattendSettings,
) -> Result<()> {
    term::section("Generating autounattend.xml");

    let xml = build_xml(s, lang, image).render()?;

    let dest = staging_root.join("autounattend.xml");
    fs::write(&dest, xml.as_bytes())?;

    term::ok(&format!("Written to {}", dest.display()));
    Ok(())
}

// XML builder
fn build_xml(s: &UnattendSettings, lang: &str, image: u32) -> XmlSegment {
    let dynamic_update_on_error_only = if s.dynamic_update_on_error_only {
        "OnError"
    } else {
        "Always"
    };
    let key_ui = if s.show_product_key_ui {
        "Always"
    } else {
        "Never"
    };

    // Build out the entire structural Unattend spec
    xml! {
        "unattend", [ "xmlns" => "urn:schemas-microsoft-com:unattend" ] {

            // windowsPE pass
            "settings", [ "pass" => "windowsPE" ] {
                "component", [
                    "name" => "Microsoft-Windows-International-Core-WinPE",
                    "processorArchitecture" => "amd64",
                    "publicKeyToken" => "31bf3856ad364e35",
                    "language" => "neutral",
                    "versionScope" => "nonSxS",
                    "xmlns:wcm" => "http://schemas.microsoft.com/WMIConfig/2002/State",
                    "xmlns:xsi" => "http://www.w3.org/2001/XMLSchema-instance"
                ] {
                    "SetupUILanguage", [] {
                        "UILanguage", [] { lang; }
                        "WillShowUI", [] { "Never"; }
                    }
                    "InputLocale", [] { lang; }
                    "SystemLocale", [] { lang; }
                    "UILanguage", [] { lang; }
                    "UserLocale", [] { lang; }
                }

                "component", [
                    "name" => "Microsoft-Windows-Setup",
                    "processorArchitecture" => "amd64",
                    "publicKeyToken" => "31bf3856ad364e35",
                    "language" => "neutral",
                    "versionScope" => "nonSxS",
                    "xmlns:wcm" => "http://schemas.microsoft.com/WMIConfig/2002/State",
                    "xmlns:xsi" => "http://www.w3.org/2001/XMLSchema-instance"
                ] {
                    "DynamicUpdate", [] {
                        "WillShowUI", [] { dynamic_update_on_error_only; }
                    }
                    "UserData", [] {
                        "ProductKey", [] {
                            "Key", [] {}
                            "WillShowUI", [] { key_ui; }
                        }
                        "AcceptEula", [] { s.accept_eula; }
                    }

                    "DiskConfiguration", [] {
                        "WillShowUI", [] { "Never"; }
                        "Disk", ["wcm:action" => "add"] {
                            "DiskID",     [] { "0"; }
                            "WillWipeDisk", [] { "true"; }
                            "CreatePartitions", [] {
                                "CreatePartition", ["wcm:action" => "add"] {
                                    "Order",  [] { "1"; }
                                    "Type",   [] { "EFI"; }
                                    "Size",   [] { "100"; }
                                }
                                "CreatePartition", ["wcm:action" => "add"] {
                                    "Order",  [] { "2"; }
                                    "Type",   [] { "MSR"; }
                                    "Size",   [] { "16"; }
                                }
                                "CreatePartition", ["wcm:action" => "add"] {
                                    "Order",   [] { "3"; }
                                    "Type",    [] { "Primary"; }
                                    "Extend",  [] { "true"; }
                                }
                            }
                            "ModifyPartitions", [] {
                                "ModifyPartition", ["wcm:action" => "add"] {
                                    "Order",      [] { "1"; }
                                    "PartitionID", [] { "1"; }
                                    "Format",      [] { "FAT32"; }
                                    "Label",       [] { "System"; }
                                }
                                "ModifyPartition", ["wcm:action" => "add"] {
                                    "Order",      [] { "2"; }
                                    "PartitionID", [] { "2"; }
                                }
                                "ModifyPartition", ["wcm:action" => "add"] {
                                    "Order",      [] { "3"; }
                                    "PartitionID", [] { "3"; }
                                    "Format",      [] { "NTFS"; }
                                    "Label",       [] { "Windows"; }
                                    "Letter",      [] { "C"; }
                                }
                            }
                        }
                    }

                    "ImageInstall", [] {
                        "OSImage", [] {
                            "InstallFrom", [] {
                                "MetaData", ["wcm:action" => "add"] {
                                    "Key", [] { "/IMAGE/INDEX"; }
                                    "Value", [] { image; }
                                }
                            }
                            "WillShowUI",    [] { "Never"; }
                            "InstallTo", [] {
                                "DiskID",      [] { "0"; }
                                "PartitionID", [] { "3"; }
                            }
                        }
                    }

                    "EnableNetwork", [] { !s.disable_network_in_pe; }
                    "EnableFirewall", [] { s.enable_firewall_in_pe; }
                }
            }

            // specialize
            "settings", [ "pass" => "specialize" ] {
                "component", [
                    "name" => "Microsoft-Windows-Shell-Setup",
                    "processorArchitecture" => "amd64",
                    "publicKeyToken" => "31bf3856ad364e35",
                    "language" => "neutral",
                    "versionScope" => "nonSxS"
                ] {
                    "RegisteredOwner", [] {}
                    "RegisteredOrganization", [] {}
                }
            }

            // oobeSystem
            "settings", [ "pass" => "oobeSystem" ] {
                "component", [
                    "name" => "Microsoft-Windows-Shell-Setup",
                    "processorArchitecture" => "amd64",
                    "publicKeyToken" => "31bf3856ad364e35",
                    "language" => "neutral",
                    "versionScope" => "nonSxS",
                    "xmlns:wcm" => "http://schemas.microsoft.com/WMIConfig/2002/State",
                    "xmlns:xsi" => "http://www.w3.org/2001/XMLSchema-instance"
                ] {
                    "OOBE", [] {
                        "NetworkLocation", [] { "Home"; }
                        "SkipMachineOOBE", [] { s.skip_machine_oobe; }
                        "SkipUserOOBE", [] { s.skip_user_oobe; }
                        "HideEULAPage", [] { s.hide_eula_page; }
                        "HideLocalAccountScreen", [] { s.hide_local_account_screen; }
                        "HideOnlineAccountScreens", [] { s.hide_online_account_screens; }
                        "HideWirelessSetupInOOBE", [] { s.hide_wireless_setup; }
                        "ProtectYourPC", [] { s.protect_your_pc; }
                    }
                    "UserAccounts", [] {
                        "LocalAccounts", [] {
                            "LocalAccount", [ "wcm:action" => "add" ] {
                                "Password", [] {
                                    "Value", [] { "password"; }
                                    "PlainText", [] { "true"; }
                                }
                                "Description", [] { "Local Administrator Account"; }
                                "DisplayName", [] { "Admin"; }
                                "Group", [] { "Administrators"; }
                                "Name", [] { "Admin"; }
                            }
                        }
                    }
                }

                "component", [
                    "name" => "Microsoft-Windows-International-Core",
                    "processorArchitecture" => "amd64",
                    "publicKeyToken" => "31bf3856ad364e35",
                    "language" => "neutral",
                    "versionScope" => "nonSxS",
                    "xmlns:wcm" => "http://schemas.microsoft.com/WMIConfig/2002/State",
                    "xmlns:xsi" => "http://www.w3.org/2001/XMLSchema-instance"
                ] {
                    "InputLocale", [] { lang; }
                    "SystemLocale", [] { lang; }
                    "UILanguage", [] { lang; }
                    "UserLocale", [] { lang; }
                }
            }
        }
    }
}
