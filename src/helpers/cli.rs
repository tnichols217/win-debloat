use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "win-debloat",
    about = "Debloat a Windows ISO on Linux reproducibly",
    long_about = None
)]
pub struct Cli {
    /// Path to the source Windows ISO
    #[arg(short, long, default_value = "windows.iso")]
    pub iso: PathBuf,

    /// Windows edition to process, e.g. "Windows 11 Pro"
    #[arg(short, long, default_value = "Windows 11 Pro")]
    pub edition: String,

    /// Output ISO path (e.g. win11-debloat.iso)
    #[arg(short, long, default_value = "output.iso")]
    pub output: PathBuf,

    /// Remove bloatware AppX packages [default: yes]
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    pub appx: bool,

    /// Remove unnecessary Windows Capabilities & Packages [default: yes]
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    pub capabilities: bool,

    /// Remove OneDrive [default: yes]
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    pub onedrive: bool,

    /// Remove Microsoft Edge [default: yes]
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    pub edge: bool,

    /// Remove AI components (Copilot, Recall, etc.) [default: yes]
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    pub ai: bool,

    /// Bypass TPM/SecureBoot checks [default: yes]
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    pub tpm_bypass: bool,

    /// Language to install
    #[arg(long, default_value = "en-US", action = clap::ArgAction::Set)]
    pub language: String,

    /// Keep staging directory after completion (useful for debugging)
    #[arg(long)]
    pub keep_staging: bool,

    /// Directory for staging files (default: OS temp dir). Must have ~15 GB free.
    #[arg(long)]
    pub staging_dir: Option<PathBuf>,

    /// Test dependencies for the program
    #[arg(long)]
    pub test: bool,
}
