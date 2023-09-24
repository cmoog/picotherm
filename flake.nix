{
  description = "A minimal thermostat controller for RP2040.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ (import rust-overlay) ]; };
        target = "thumbv6m-none-eabi";
        rust = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "rust-analyzer" ];
          targets = [ target ];
        };
        # embassy src includes the cyw firmware blobs we need to embed at build time
        embassy = pkgs.fetchFromGitHub {
          owner = "embassy-rs";
          repo = "embassy";
          # use same git revision for this source as embassy Cargo dependency
          rev = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).dependencies.embassy-rp.rev;
          sha256 = "sha256-FSkMIRIrAko3sprGBsZ6Inh0xHUWULtPlEIOVafB4YY=";
        };
        naersk' = pkgs.callPackage naersk { rustc = rust; cargo = rust; };
        therm-elf = naersk'.buildPackage {
          pname = "therm";
          src = pkgs.runCommand "src-with-firmware-blobs" { } ''
            mkdir $out
            cp -r ${./.}/* $out/
            ln -sf ${embassy} $out/embassy
          '';
          CARGO_BUILD_TARGET = target;
          # strangely, this was stripping nearly the entire binary
          dontStrip = true;
        };
        therm-uf2 = pkgs.runCommand "elf-to-uf2" { buildInputs = [ pkgs.elf2uf2-rs ]; } ''
          mkdir -p $out/bin
          elf2uf2-rs ${therm-elf}/bin/therm $out/bin/therm.uf2
        '';
      in
      {
        formatter = pkgs.nixpkgs-fmt;
        packages = {
          inherit therm-elf therm-uf2;
          default = therm-uf2;
        };
        devShells.default = with pkgs; mkShell {
          packages = [
            elf2uf2-rs
            go_1_21
            probe-rs
            rust
            rust-analyzer
          ];
          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/src";
          # link embassy src for including cyw firmware blobs
          shellHook = ''
            rm embassy
            ln -sf ${embassy} embassy
          '';
        };
      }
    );
}
