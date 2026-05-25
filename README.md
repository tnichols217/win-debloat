# win-debloat

Takes a stock Windows 10/11 ISO, strips bloatware at the WIM image level, and produces a clean bootable ISO — entirely on Linux, no Wine required.

## What it does

| Step | Tool used | What it removes / changes |
|------|-----------|--------------------------|
| Extract ISO | `xorriso` | Unpacks ISO to staging dir |
| Export WIM index | `wimlib-imagex` | Handles both `.wim` and `.esd` sources |
| Mount WIM (FUSE) | `wimlib-imagex mountrw` | Read-write FUSE mount of the image |
| AppX removal | file deletion | Teams, Outlook, Copilot, Xbox, Clipchamp, Skype, Bing apps, etc. |
| Capabilities | file deletion | IE, WordPad, Media Player, PSE, Steps Recorder, speech/OCR packs |
| OneDrive | file deletion | `OneDriveSetup.exe`, default-user shortcut, AppX dirs |
| Edge | file deletion + registry | Program Files dirs, AppRepository, WebView, update tasks |
| AI components | file deletion + registry | Copilot, Recall, AIX, CoreAI packages; Notepad AI; search suggestions |
| Privacy tweaks | `hivexsh` | Telemetry=0, ad ID disabled, consumer features off, CDM silent installs off |
| User folders | `hivexsh` | Desktop/Documents/Downloads/Music/Pictures/Videos visible in Explorer |
| TPM bypass (opt-in) | file patch | Registry tweaks |
| Repack ISO | `xorriso` | Bootable ISO with original BIOS + UEFI boot entries preserved |

## Dependencies

- xorriso, wimlib, hivex, p7zip

Registry tweaks degrade gracefully if `hivexsh` is not present (a warning is printed, everything else proceeds).

## Quick start with Nix

```bash
nix develop
cargo build --release
```

## Usage

```
Usage: win-debloat [OPTIONS]

Options:
  -i, --iso <ISO>                    Path to the source Windows ISO [default: windows.iso]
  -e, --edition <EDITION>            Windows edition to process, e.g. "Windows 11 Pro" [default: "Windows 11 Pro"]
  -o, --output <OUTPUT>              Output ISO path (e.g. win11-debloat.iso) [default: output.iso]
      --appx <APPX>                  Remove bloatware AppX packages [default: yes] [default: true] [possible values: true, false]
      --capabilities <CAPABILITIES>  Remove unnecessary Windows Capabilities & Packages [default: yes] [default: true] [possible values: true, false]
      --onedrive <ONEDRIVE>          Remove OneDrive [default: yes] [default: true] [possible values: true, false]
      --edge <EDGE>                  Remove Microsoft Edge [default: yes] [default: true] [possible values: true, false]
      --ai <AI>                      Remove AI components (Copilot, Recall, etc.) [default: yes] [default: true] [possible values: true, false]
      --tpm-bypass <TPM_BYPASS>      Bypass TPM/SecureBoot checks [default: yes] [default: true] [possible values: true, false]
      --language <LANGUAGE>          Language to install [default: en-US]
      --keep-staging                 Keep staging directory after completion (useful for debugging)
      --staging-dir <STAGING_DIR>    Directory for staging files (default: OS temp dir). Must have ~15 GB free
      --test                         Test dependencies for the program
  -h, --help                         Print help
```

### Examples

```bash
# Full debloat
win-debloat --iso Win11_24H2_English_x64.iso --output win11-clean.iso

# Non-interactive, keep Edge (e.g. needed for WebView2 apps)
win-debloat \
  --iso Win11_24H2_English_x64.iso \
  --edition "Windows 11 Pro" \
  --output win11-clean.iso \
  --edge false

# Older hardware without TPM 2.0
win-debloat \
  --iso Win11_24H2_English_x64.iso \
  --edition "Windows 11 Pro" \
  --output win11-notpm.iso \
  --tpm-bypass true

# Use a specific staging dir with enough space
win-debloat \
  --iso Win11_24H2.iso \
  --edition "Windows 11 Pro" \
  --output win11-clean.iso \
  --staging-dir /mnt/scratch/win-debloat-work
```

### A note on AppX removal depth

On Linux without DISM, we delete the physical files from the WIM and remove AppRepository entries. This achieves the same outcome for most packages, though some deeply integrated CBS-tracked packages may require the additional registry CBS hive edits.

## Disk space

The process needs roughly **3× the ISO size** of free space in the staging directory:
- ~5 GB for ISO extraction
- ~5 GB for mounted WIM working copy  
- ~4 GB for the output ISO

A 16 GB USB has enough space for the output; the staging work happens on your build machine.

## Nix packaging (flake)

See `flake.nix` for a complete Nix flake that builds this as a proper derivation with all runtime dependencies in the closure.
