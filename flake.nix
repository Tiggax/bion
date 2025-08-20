{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, flake-utils, naersk, rust-overlay, nixpkgs}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [
            (import rust-overlay)
          ];
        };

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            toolchain
            mdbook
            rustPackages.clippy

            xorg.libX11 
            xorg.libXcursor 
            xorg.libXi 
            xorg.libXrandr
            xorg.libxcb  
            libxkbcommon
            libGL
            
            wayland

            pkg-config 

          ];
          
          shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath nativeBuildInputs)}";
            '';
        };
      }
    );
}
