{
  inputs = {
    # nixpkgs 35.11 still contains rust 1.73
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

 		flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, flake-utils, nixpkgs, fenix, crane, ... }@inputs: flake-utils.lib.eachDefaultSystem (system: 

############################## LET BINDINGS ##############################
let
  pkgs = nixpkgs.legacyPackages.${system};
  wasmToolchain = fenix.packages.${system}.combine [
    fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.toolchain
    fenix.packages.${system}.complete.toolchain
  ];
  osToolchain = fenix.packages.${system}.complete.toolchain;
  wasmCrane = crane.lib.${system}.overrideToolchain wasmToolchain;
  osCrane = crane.lib.${system}.overrideToolchain osToolchain;

in {
############################## PACKAGES ##############################
    packages.default = osCrane.buildPackage {
      src = "${self}";
    };

    packages = {
      wasm = wasmCrane.buildPackage {
        src = "${self}";
      };
    };

############################## DEV SHELLS ##############################
    devShells.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        wasm-pack #pkg-config openssl #cargo rustc
        cargo-generate
        /*
        (fenix.packages.${system}.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ])
        */
        (fenix.packages.${system}.combine [ wasmToolchain osToolchain ])
      ];
    };

  });
}

