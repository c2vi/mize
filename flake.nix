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
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    mize_modules = {
      #url = "github:c2vi/mize-modules";
      url = "git+file:///home/me/work/modules?submodules=1";
      flake = false;
    };

 		flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, flake-utils, nixpkgs, fenix, crane, mize_modules, rust-overlay, ... }@inputs: flake-utils.lib.eachDefaultSystem (system: 

############################## LET BINDINGS ##############################
let
  pkgs = nixpkgs.legacyPackages.${system};
  wasmToolchain = fenix.packages.${system}.combine [
    fenix.packages.${system}.targets.wasm32-unknown-unknown.latest.toolchain
    fenix.packages.${system}.latest.toolchain
  ];
  wasmCrane = (crane.mkLib pkgs).overrideToolchain wasmToolchain;

  osToolchain = fenix.packages.${system}.latest.toolchain;
  osCrane = (crane.mkLib pkgs).overrideToolchain osToolchain;

  #winToolchain = fenix.packages.${system}.combine [
    #fenix.packages.${system}.targets.x86_64-pc-windows-gnu.latest.toolchain
    #fenix.packages.${system}.latest.toolchain
  #];
  winToolchain = with fenix.packages.${system};
    combine [
      minimal.rustc
      minimal.cargo
      targets.x86_64-pc-windows-gnu.latest.rust-std
  ];
  winCrane = (crane.mkLib pkgs).overrideToolchain winToolchain;

  wasmArtifacts = wasmCrane.buildDepsOnly ({
    src = self;
    doCheck = false; # tests does not work in wasm
    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  });

  defaultMizeConfig = {
    config = {
      namespace = "mize.buildtime.ns";
      module_url = "c2vi.dev";
    };
  };

  #systems = [ "aarch64-linux" "x86_64-linux" "x86_64-pc-windows-gnu" "wasm32-unknown-none-unknown" "x86_64-apple-darwin" "aarch64-apple-darwin" ];
  systems = [ "aarch64-unknown-linux-gnu" "x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu" "wasm32-unknown-none-unknown" ];

  mizeLib = import ./lib.nix {
    inherit inputs nixpkgs pkgs self osCrane defaultMizeConfig mize_modules;
    inherit rust-overlay crane fenix;
    localSystem = system;
    stdenv = pkgs.stdenv;
  };

in {
############################## PACKAGES ##############################

    packages = rec {

      inherit osCrane;
      #test = mizeLib.buildMizeForSystem "wasm32-unknown-none-unknown";
      test = mizeLib.buildMizeForSystem "aarch64-linux";
      #test = mizeLib.buildMizeForSystem "x86_64-pc-windows-gnu";

      mizeFor = let
        mizes = map mizeLib.buildMizeForSystem systems;
      in builtins.listToAttrs ( map ( mize: { name = mize.system.name; value = mize; } ) mizes );

      #pkgsCross = import nixpkgs { localSystem = system; crossSystem = { config = "x86_64-pc-windows-gnu"; }; overlays = [ rust-overlay.overlays.default ]; };

      pkgsTest = pkgs;

      #pkgsCross = import nixpkgs { localSystem = system; crossSystem = { config = "x86_64-w64-windows-mingw"; }; overlays = [ rust-overlay.overlays.default ]; };
      pkgsCross = import nixpkgs { localSystem = system; crossSystem = { config = "x86_64-unknown-linux-gnu"; }; overlays = [ rust-overlay.overlays.default ]; };
      craneLib = (crane.mkLib pkgsCross).overrideToolchain (p: p.rust-bin.stable.latest.default);


      one = craneLib.buildPackage {
        src = "${self}";
        #cargoExtraArgs = "--bin mize --features os-target";
        cargoExtraArgs = "--bin mize";
        doCheck = false; # tests does not work in wasm
        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
        MIZE_BUILD_CONFIG = pkgs.writeTextFile {
          name = "mize-build-config";
          text = builtins.toJSON (defaultMizeConfig // {
            mize_version = one.version;
          });
        };
      };

      webfiles = mizeLib.webfiles systems;

      default = osCrane.buildPackage {
        src = "${self}";
        cargoExtraArgs = "--bin mize --features os-target";
      };

      wasm = wasmCrane.buildPackage {
        src = "${self}";
        CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        cargoExtraArgs = "--bin mize --features wasm-target";
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
    devShells = {

      one = pkgs.stdenv.mkDerivation {
        name = "hiiiiiiii";
        nativeBuildInputs = [
          (fenix.packages.${system}.combine [ wasmToolchain osToolchain winToolchain ])
        ];
        buildInputs = [
          pkgs.pkgsCross.mingwW64.stdenv.cc.cc
          pkgs.pkgsCross.mingwW64.windows.pthreads
          (fenix.packages.${system}.combine [ wasmToolchain osToolchain winToolchain ])
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.pkgsCross.mingwW64.windows.pthreads
        ];

        RUSTFLAGS="-L ${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";


        #fixes issues related to openssl
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

        depsBuildBuild = with pkgs; [
          pkgsCross.mingwW64.stdenv.cc
          pkgsCross.mingwW64.windows.pthreads
        ];
      };

      win = winCrane.buildPackage {
        #src = winCrane.cleanCargoSource ./testing;
        #pname = "testing";
        src = ./testing;

        strictDeps = true;
        doCheck = false;

        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

        # fixes issues related to libring
        TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

        #fixes issues related to openssl
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

        depsBuildBuild = with pkgs; [
          pkgsCross.mingwW64.stdenv.cc
          pkgsCross.mingwW64.windows.pthreads
        ];
        shellHook = ''
          export MIZE_CONFIG_FILE=${self}/test-config.toml
        '';
        MIZE_BUILD_CONFIG = pkgs.writeTextFile {
          name = "vic-build-config";
          text = builtins.toJSON defaultMizeConfig;
        };
      };


      default = pkgs.mkShell {
        buildInputs = with pkgs; [

          wasm-pack #pkg-config openssl #cargo rustc
          cargo-generate
          (fenix.packages.${system}.combine [ wasmToolchain osToolchain ])
          lldb gdb
        ];

        MIZE_BUILD_CONFIG = pkgs.writeTextFile {
          name = "vic-build-config";
          text = builtins.toJSON defaultMizeConfig;
        };

        shellHook = ''
          export MIZE_CONFIG_FILE=${self}/test-config.toml
        '';
      };
    };

  }) // {

############################## SOME GLOBAL OUTPUTS ##############################
    inherit inputs self;
    rustPkgs = import nixpkgs { overlays = [rust-overlay.overlays.default]; system = "x86_64-linux"; };
  };
}

