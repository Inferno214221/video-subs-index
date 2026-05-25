{
  description = "Function Overloading Demo";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk-pkg = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk-pkg, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-dyB9BstJde3dBftePhxGENy4P0+eshXFo9UCgUSKHrE=";
        };
        naersk = pkgs.callPackage naersk-pkg {
          cargo = toolchain;
          rustc = toolchain;
        };
        buildInputs = with pkgs; [];
        nativeBuildInputs = with pkgs; [
          toolchain
          pkg-config
          gcc
          cargo-expand
          cargo-public-api
          cargo-llvm-cov
          man-pages
        ] ++ buildInputs;
      in with pkgs; rec
      {
        devShells.default = mkShell {
          inherit nativeBuildInputs;
        };
      }
    );
}
