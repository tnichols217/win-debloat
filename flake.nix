{
  description = "win-debloat — Windows ISO debloater for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        runtimeDeps = with pkgs; [
          p7zip
          xorriso
          wimlib
          hivex
          fuse
        ];
        win-debloat = pkgs.rustPlatform.buildRustPackage {
          pname = "win-debloat";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "xml_builder_macro-0.1.0" = "sha256-dOXCONTlp7lin+REaAXHUj84uB8rh+3prwqD6JQAmVw=";
            };
          };

          P7ZIP = "${pkgs.p7zip}/bin/7z";
          XORRISO = "${pkgs.xorriso}/bin/xorriso";
          WIMLIB = "${pkgs.wimlib}/bin/wimlib-imagex";
          HIVEXREG = "${pkgs.hivex}/bin/hivexregedit";

          meta = with pkgs.lib; {
            description = "Debloat a Windows ISO on Linux";
            license = licenses.gpl3Plus;
            platforms = platforms.linux;
            mainProgram = "win-debloat";
          };
        };

      in
      {
        packages = {
          inherit win-debloat;
          default = win-debloat;
        };

        devShells.default = pkgs.mkShell {
          buildInputs =
            with pkgs;
            [
              rust-analyzer
              clippy
              rustfmt
              file
              util-linux
              nil
              nixd
            ]
            ++ runtimeDeps
            ++ [ rustToolchain ];

          P7ZIP = "${pkgs.p7zip}/bin/7z";
          XORRISO = "${pkgs.xorriso}/bin/xorriso";
          WIMLIB = "${pkgs.wimlib}/bin/wimlib-imagex";
          HIVEXREG = "${pkgs.hivex}/bin/hivexregedit";

          shellHook = ''
            echo "win-debloat dev shell — cargo build --release"
          '';
        };

        overlays.default = final: prev: { inherit win-debloat; };
      }
    );
}
