{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            targets = ["wasm32-unknown-unknown"];
          });

        linker =
          if system == "x86_64-linux"
          then with pkgs; [clang mold]
          else [];

        just = pkgs.callPackage ./justfile.nix {};
      in {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs =
            (with pkgs; [pkg-config])
            ++ linker;

          buildInputs = with pkgs; [
            just
            rust
            rust-analyzer
            udev
            alsa-lib
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr # To use the x11 feature
            libxkbcommon
            wayland # To use the wayland feature
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          shellHook = ''
            ${just}/bin/just
          '';
        };
      }
    );
}
