{
  description = "win-debloat — Windows ISO debloater for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    appimage.url = "github:ralismark/nix-appimage";
  };

  outputs =
    { ... }@inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import inputs.rust-overlay) ];
        pkgs = import inputs.nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        runtimeDeps = with pkgs; [
          p7zip
          xorriso
          wimlib
          hivex
          fuse
        ];
        win-debloat = pkgs.callPackage ./nix/build.nix { };

      in
      {
        packages = {
          inherit win-debloat;
          appimage = inputs.appimage.bundlers.${system}.default win-debloat;
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
              act
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
