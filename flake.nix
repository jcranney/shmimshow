{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    shmim-tools.url = "github:jcranney/shmim-tools";
  };

  outputs = { self, nixpkgs, utils, naersk, shmim-tools }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        libPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
          vulkan-loader
        ];
      in
      {
        packages = rec {
          shmimshow = pkgs.stdenv.mkDerivation {
            buildInputs = with pkgs; [
              cargo rustc rustfmt pre-commit rustPackages.clippy
              wasm-pack 
            ];
            name = "shmimshow";
            src = ./.;
            buildPhase = ''
              cargo build --release --bin shmimshow
              cp ./target/release $out
            '';
          };
          default = shmimshow;
        };
        devShell = with pkgs; mkShell {
          buildInputs = [ 
            cargo rustc rustfmt pre-commit rustPackages.clippy
            wasm-pack
            shmim-tools.packages.${system}.default
          ];
          # RUST_LOG = "debug";
          LD_LIBRARY_PATH = libPath;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}
