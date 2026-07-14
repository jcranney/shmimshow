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
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [ 
            # default rust flake:
            cargo rustc rustfmt pre-commit rustPackages.clippy
            # wgpu
            wasm-pack # wayland pkg-config
            # pkg-config
            # vulkan-headers
            # vulkan-loader
            # vulkan-tools
            # gtk specific:
            # pkg-config gtk4
            ## The gtk-rs docs say that I'll need these, but I won't include
            ## them until I hit errors, because I guess some of them are
            ## included in the gtk4 package anyway.
            # libadwaita meson desktop-file-utils gcc glib desktop-file-utils
            
            # My tools:
            shmim-tools.packages.${system}.default
          ];
          # RUST_LOG = "debug";
          LD_LIBRARY_PATH = libPath;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}
