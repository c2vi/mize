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
    fenix.packages.${system}.targets.wasm32-unknown-unknown.latest.toolchain
    fenix.packages.${system}.latest.toolchain
  ];
  osToolchain = fenix.packages.${system}.latest.toolchain;
  wasmCrane = crane.lib.${system}.overrideToolchain wasmToolchain;
  osCrane = crane.lib.${system}.overrideToolchain osToolchain;

  wasmArtifacts = wasmCrane.buildDepsOnly ({
    src = self;
    doCheck = false; # tests does not work in wasm
    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  });

in {
############################## PACKAGES ##############################

    webfiles = pkgs.callPackage ./webfiles.nix { inherit inputs nixpkgs self; };

    packages.default = osCrane.buildPackage {
      src = "${self}";
      cargoExtraArgs = "--bin mize --features os-binary";
    };

    packages = rec {
      wasm = wasmCrane.buildPackage {
        src = "${self}";
        CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        doCheck = false;
      };

      # Thanks to:
      # - https://github.com/dl-solarity/hardhat-diamond-tools/blob/e5b8bc9624fbf89eda83a8ce4f6b6672ea83f550/flake.nix#L103
      # - https://github.com/ipetkov/crane/issues/362#issuecomment-1683220603
      npmPackage = wasmCrane.mkCargoDerivation {
        name = "mize-npm-package";

        # i can't use CARGO_BUILD_TARGET to set the target
        # because then the cargo install run by wasm-pack tries to build the wasm-bindgen-cli for a wasm target...
        cargoExtraArgs = "--target wasm32-unknown-unknown --features wasm-target";
        src = self;

        cargoArtifacts = wasmArtifacts;
        doCheck = false;

        buildPhaseCargoCommand = ''
            mkdir -p $out/pkg

            HOME=$(mktemp -d fake-homeXXXX) wasm-pack build --out-dir $out/pkg --scope=c2vi -- --features wasm-target
        '';

        buildInputs = with pkgs; [ wasm-bindgen-cli binaryen wasm-pack wasmToolchain ];
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
        lldb gdb
      ];

      shellHook = ''
        export MIZE_CONFIG_FILE=${self}/test-config.toml
      '';
    };

  }) // {

############################## SOME GLOBAL OUTPUTS ##############################
    inherit inputs self;
  };
}

