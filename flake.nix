{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [ 
            cargo rustc rustfmt pre-commit rustPackages.clippy 
            xorg.libX11 
            xorg.libXcursor 
            xorg.libXi 
            xorg.libXrandr
            libxkbcommon
            pipewire
            libGL
            wayland 
            xorg.libxcb  
            alsa-lib 
            libudev-zero 
            openssl 
            llvm 
            pkg-config 
            gcc 
            sqlite
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
              with pkgs;
              pkgs.lib.makeLibraryPath [
                xorg.libX11 
                xorg.libXcursor 
                xorg.libXi
                xorg.libXrandr
                libGL
                libxkbcommon 
                wayland
                xorg.libxcb  
                pkgs.vulkan-loader
                pkgs.glfw
              ]
            }";
        };
      }
    );
}
