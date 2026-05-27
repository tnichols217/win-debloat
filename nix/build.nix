{
  lib,
  rustPlatform,
  p7zip,
  xorriso,
  wimlib,
  hivex,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "win-debloat";
  version = "0.1.0";

  src = ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "xml_builder_macro-0.1.0" = "sha256-dOXCONTlp7lin+REaAXHUj84uB8rh+3prwqD6JQAmVw=";
    };
  };

  P7ZIP = "${p7zip}/bin/7z";
  XORRISO = "${xorriso}/bin/xorriso";
  WIMLIB = "${wimlib}/bin/wimlib-imagex";
  HIVEXREG = "${hivex}/bin/hivexregedit";

  meta = with lib; {
    description = "Debloat a Windows ISO on Linux";
    license = licenses.gpl3Plus;
    platforms = platforms.linux;
    mainProgram = "win-debloat";
  };
}
