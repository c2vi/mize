{
	nixConfig.bash-prompt = ''\033[22m\033[31mdev \033[01;34m\W\033[00m ❯❯❯ '';
  inputs = {
    # nixpkgs 35.11 still contains rust 1.73
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

		vicpkgs.url = "github:c2vi/vicpkgs";
		vicpkgs.flake = false;

 		flake-utils.url = "github:numtide/flake-utils";

    c2vi-config.url = "github:c2vi/nixos";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, crane, fenix, flake-utils, nixpkgs, c2vi-config, ... }@inputs: 
  ####################################
  # some global outputs
  {
    inherit inputs self;

  } //

  ####################################
  # system specific outpugs
  flake-utils.outputs.lib.eachSystem flake-utils.outputs.lib.allSystems (system: let 
    ####################################
    # let bindings for outputs

    pkgs = nixpkgs.legacyPackages.${system};

    osCrane = crane.lib.${system}.overrideToolchain fenix.packages.${system}.latest.toolchain;

    url = "http://c2vi.dev";

    repoUrl = "github:c2vi/victorinix";

    buildVicForSystem = import ./build-vic-for-system.nix;

    defaultVicConfig = {
      inherit url repoUrl;
      vic_dir = "~/.victorinix";
      #vicpkgs_url = "github:c2vi/vicpkgs/${inputs.vicpkgs.rev}";
      vicpkgs_url = "/home/me/work/pkgsvic";
    };

    filteredRustSource = builtins.filterSource (name: type: (
      pkgs.lib.strings.hasInfix "src" name ||
      baseNameOf name == "Cargo.toml" ||
      baseNameOf name == "build.rs" ||
      baseNameOf name == "Cargo.lock"
    )) ./.;

    getTarballClosure = pkgs: system: let
      pkgsCross = if pkgs.system == system then
        (import nixpkgs { system = pkgs.system; overlays = [ c2vi-config.overlays.static ]; })
      else
        (import nixpkgs { system = pkgs.system; crossSystem = system; overlays = [ c2vi-config.overlays.static ]; })
      ;
      in rec {
        proot = pkgsCross.pkgsStatic.proot;
        nix = pkgsCross.pkgsStatic.nix;
        busybox = pkgsCross.pkgsStatic.busybox;
        cacert = pkgsCross.cacert;
        info = pkgs.buildPackages.closureInfo { rootPaths = [ nix busybox cacert ]; };
        inherit system;
      };

    # /*
    getVicorinix = pkgs: crossSystem: short: cargoSha256: let
      pkgsStatic = if pkgs.system == crossSystem then pkgs.pkgsStatic else pkgs.pkgsCross.${crossSystem}.pkgsStatic;
    in pkgsStatic.rustPlatform.buildRustPackage rec {
      name = "victorinix-${short}";
      VIC_BUILD_CONFIG = pkgs.writeTextFile {
        name = "vic-build-config";
        text = builtins.toJSON defaultVicConfig;
      };
      MIZE_BUILD_CONFIG = pkgs.writeTextFile {
        name = "vic-build-config";
        text = builtins.toJSON {};
      };
      buildInputs = with pkgsStatic; [ libelf openssl ];
      nativeBuildInputs = with pkgs; [ pkg-config ];
      src = filteredRustSource;
      inherit cargoSha256;
      #cargoSha256 = "sha256-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=";
    };
    # */

     /*
    # this does not work, failing to find cc: error: linker `cc` not found ... No such file or directory (os error 2)
    getVicorinix = pkgs: crossSystem: short: cargoSha256 let
      pkgsStatic = if pkgs.system == crossSystem then pkgs.pkgsStatic else pkgs.pkgsCross.${crossSystem}.pkgsStatic;
      craneLib = crane.mkLib pkgsStatic;
    in craneLib.buildPackage {
        VIC_BUILD_CONFIG = pkgs.writeTextFile {
          name = "victorinix-${short}";
          text = builtins.toJSON defaultVicConfig;
        };
        src = self;
        nativeBuildInputs = with pkgsStatic; [ pkg-config ];
        buildInputs = with pkgsStatic; [ openssl libelf ];
    };
    # */

   in rec {



    packages = rec {

      proot-la =
        let
          pkgs = import nixpkgs { system = system; crossSystem = "aarch64-linux"; overlays = [ c2vi-config.overlays.static ]; };
        in pkgs.pkgsStatic.proot;

      proot-l =
        let
          pkgs = import nixpkgs { system = system; crossSystem = "x86_64-linux"; overlays = [ c2vi-config.overlays.static ]; };
        in pkgs.pkgsStatic.proot;
      victorinix-l = getVicorinix pkgs "x86_64-linux" "l" "sha256-0kAb+sieN+Ipnr8E3CS3oy+9+4qvUQU3rXrhpJyGTIM=";

      webfiles = pkgs.callPackage ./webfiles.nix {inherit inputs nixpkgs self c2vi-config url getTarballClosure getVicorinix; vicConfig = defaultVicConfig; };

      webrun = pkgs.writeShellScriptBin "vic-webrun" ''
        ${pkgs.darkhttpd}/bin/darkhttpd ${packages.webfiles} --log ./victorinix-access.log $@
      '';

      npmPackage = pkgs.buildNpmPackage {
        VIC_URL = url;
        pname = "victorinix-npmPackage";
        version = self.packages.${system}.vic.version;
        src = "${self}/npm-package";
      };

      pkgsStatic.vic = let
        craneLib = crane.mkLib pkgs.pkgsStatic;
      in craneLib.buildPackage {
        VIC_BUILD_CONFIG = pkgs.writeTextFile {
          name = "vic-build-config";
          text = builtins.toJSON defaultVicConfig;
        };
        src = self;
        nativeBuildInputs = with pkgs.pkgsStatic; [ pkg-config libgcc gcc openssl libelf ];
        buildInputs = with pkgs.pkgsStatic; [ libgcc gcc ];
      };

      victorinix-la = getVicorinix pkgs "aarch64-multiplatform" "la" "sha256-eB/+tcI5+pWSMq2fIKI3qPcuRKOg0r1C3/wm999G8CE=";

      vicPkgs = pkgs
        // (import vicPkgs { inherit pkgs; }).extra
        // (import vicPkgs { inherit pkgs; }).winePkgs
        ;

    } // (c2vi-config.lib.flakeAddCross { inherit system; } ({ crossSystemFullString, ... }: 
    ############## cross compilable packages
    let
      craneLib = crane.lib.${system}.overrideToolchain fenix.packages.${system}.targets.${crossSystemFullString}.latest.toolchain;
    in
    {

      inherit craneLib;
      vic = craneLib.buildPackage {
        VIC_BUILD_CONFIG = pkgs.writeTextFile {
          name = "vic-build-config";
          text = builtins.toJSON defaultVicConfig;
        };
        src = self;
        nativeBuildInputs = with pkgs; [ pkg-config openssl libelf ];
      };

    }));

    devShells.default = pkgs.mkShell {
      buildInputs = with pkgs; [ pkg-config libelf openssl cargo rustc nodePackages.npm gnumake protobuf_29 runc crun ];
      shellHook = ''
        export VIC_BUILD_CONFIG=${pkgs.writeTextFile {
          name = "vic-build-config";
          text = builtins.toJSON defaultVicConfig;
        }}
        export MIZE_BUILD_CONFIG=${pkgs.writeTextFile {
          name = "vic-build-config";
          text = builtins.toJSON {};
        }}
        export SNIX_BUILD_SANDBOX_SHELL=bash
        echo "set VIC_BUILD_CONFIG"
        echo "set MIZE_BUILD_CONFIG"
        echo "set SNIX_BUILD_SANDBOX_SHELL"
      '';
    };
  });
}

