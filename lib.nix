{ pkgs
, osCrane
, self
, defaultMizeConfig
, mize_modules
, localSystem
, nixpkgs
, rust-overlay
, crane
, fenix
, stdenv
, ...
}:
rec {

  # get a list of all modules
  # aka folders with a mize_module.nix in them
  module_list_drv = path: pkgs.stdenv.mkDerivation {
    name = "mize_module_list";
    dontUnpack = true;
    configurePhase = "";
    buildPhase = ''
      mkdir -p $out
      touch $out/modules_to_build

      for d in ${path}/modules/*; do
        [[ -f $d/mize_module.nix ]] && (echo "$d/mize_module.nix" >> $out/modules_to_build; echo found module $d/mize_module.nix)
      done

      exit 0
    '';
  };
  findModules = path: let
    module_list_string = builtins.readFile "${module_list_drv path}/modules_to_build";
    module_list = pkgs.lib.lists.remove "" (pkgs.lib.strings.splitString "\n" module_list_string);
    in module_list;


  findModulesNix = path: let
    dirs = builtins.readDir path;
    filtered_dirs = map (dir: builtins.trace "dir: ${dir}" (dir));
    module_list = filtered_dirs;
    in module_list;


  getCrossSystem = system: let
    crossSystem = (if builtins.isString system then (pkgs.lib.systems.parse.mkSystemFromString system) else system)
      // rec {
        name = if crossSystem.vendor != "unknown" 
          then "${crossSystem.cpu.name}-${crossSystem.kernel.name}-${crossSystem.abi.name}"
          else "${crossSystem.cpu.name}-${crossSystem.vendor.name}-${crossSystem.kernel.name}-${crossSystem.abi.name}";
        nameFull = "${crossSystem.cpu.name}-${crossSystem.vendor.name}-${crossSystem.kernel.name}-${crossSystem.abi.name}";
        config = nameFull;
      };
  in crossSystem;


  buildMizeForSystem = system: let
    crossSystem = getCrossSystem system;

    pkgsCross = import nixpkgs { inherit localSystem crossSystem; overlays = [ rust-overlay.overlays.default ]; };

    pkgsNative = import nixpkgs { inherit localSystem; overlays = [ rust-overlay.overlays.default ]; };

    craneLib = 
      if crossSystem.kernel.name == "windows"
        then 
          (crane.mkLib pkgs).overrideToolchain (fenix.packages.${localSystem}.combine [
            #fenix.packages.${localSystem}.minimal.rustc
            #fenix.packages.${localSystem}.minimal.cargo
            fenix.packages.${localSystem}.stable.toolchain
            fenix.packages.${localSystem}.targets."${crossSystem.cpu.name}-pc-windows-gnu".stable.toolchain
          ])

      else if crossSystem.cpu.name == "wasm32"
        then
          (crane.mkLib pkgs).overrideToolchain (fenix.packages.${localSystem}.combine [
            fenix.packages.${localSystem}.targets.wasm32-unknown-unknown.latest.toolchain
            fenix.packages.${localSystem}.latest.toolchain
          ])
      else if crossSystem.name == "x86_64-linux-gnu"
        then
          builtins.trace "x86_64-linux-gnu hackfix...."
          (crane.mkLib pkgs).overrideToolchain (fenix.packages.${localSystem}.combine [
            fenix.packages.${localSystem}.stable.toolchain
          ])

      else
          (crane.mkLib pkgsCross).overrideToolchain (p: p.rust-bin.stable.latest.default)
      ;

  toolchain_version_drv = pkgs.stdenv.mkDerivation {
    name = "toolchain_version_drv";
    dontUnpack = true;
    configurePhase = "";
    buildPhase = ''
      mkdir -p $out
      touch $out/rustc-version

      echo hiiiiiiiiiiiiiiiiiiiiiiii
      #echo 'rustc 1.80.1 (3f5fd8dd4 2024-08-06)' > $out/rustc-version
      ${craneLib.cargo}/bin/rustc --version > $out/rustc-version

      exit 0
    '';
  };



    mkSelString = attrs: builtins.toJSON (attrs // {
      inherit toolchain_version;
      system = crossSystem.nameFull;
      mize_version = main-default.version;
    });


    toolchain_version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile "${toolchain_version_drv}/rustc-version");

    mkMizeModule = attrs: pkgsCross.stdenv.mkDerivation (attrs // {
      selector_string = mkSelString (attrs.select or {} // {
        inherit (attrs) modName;
      });
    });



    ########## build Rust Module
    mkMizeRustModule = attrs: craneLib.buildPackage (attrs // {
      MIZE_BUILD_CONFIG = mizeBildConfig;
      selector_string = mkSelString (attrs.select or {} // {
        inherit (attrs) modName;
      });
    }
    # linux specific stuff
    // (if crossSystem.kernel.name == "linux" then {
      "CARGO_TARGET_${builtins.replaceStrings ["-"] ["_"] (pkgsCross.lib.strings.toUpper crossSystem.nameFull)}_LINKER" = "${pkgsCross.stdenv.cc.targetPrefix}cc";
      HOST_CC = "${pkgsCross.stdenv.cc.nativePrefix}cc";
      TARGET_CC = "${pkgsCross.stdenv.cc.targetPrefix}cc";
      CARGO_BUILD_TARGET = crossSystem.nameFull;
      nativeBuildInputs = attrs.nativeBuildInputs or [] ++ [
        pkgsCross.stdenv.cc
      ];
    } else {})

    # add wasm stuff
    // (if crossSystem.cpu.name == "wasm32" then {
      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

      # checks fail on wasm
      doCheck = false;
    } else {})

    # add windows stuff
    // (if crossSystem.kernel.name == "windows" then {
      strictDeps = true;
      doCheck = false;

      CARGO_BUILD_TARGET = "${crossSystem.cpu.name}-pc-windows-gnu";

      # fixes issues related to libring
      TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

      #fixes issues related to openssl
      OPENSSL_DIR = "${pkgs.openssl.dev}";
      OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
      OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

      depsBuildBuild = [
        pkgs.pkgsCross.mingwW64.stdenv.cc
        pkgs.pkgsCross.mingwW64.windows.pthreads
      ];
    } else {})
    );



    buildModule = path: extraArgs: let
      pkg = (pkgs.callPackage path {
        inherit mkSelString craneLib toolchain_version;
        inherit mkMizeModule mkMizeRustModule buildModule findModules crossSystem pkgsCross pkgsNative mkMizeRustShell;
        mize_version = main-default.version;
      } // extraArgs );
    in pkg // {
    };


    mizeBildConfig = let
      settingsFormat = pkgs.formats.toml { };
    in settingsFormat.generate "mize-build-config.toml" {
      config = {
        namespace = "mize.buildtime.ns";
        module_url = "c2vi.dev";
        selector = builtins.fromJSON (mkSelString {});
      };
    };

    ####### build mize
    main-default = craneLib.buildPackage ({
      src = "${self}";
      cargoExtraArgs = "--bin mize --features os-target";
      strictDeps = true;

      nativeBuildInputs = [ 
        pkgsCross.buildPackages.pkg-config
        #pkgsCross.buildPackages.nasm
        #pkgsCross.buildPackages.cmake
      ];

      buildInputs = [
          (if crossSystem.nameFull == "x86_64-unknown-linux-gnu" then pkgs.openssl else pkgsCross.openssl)
      ];

      MIZE_BUILD_CONFIG = mizeBildConfig;

      # patch the interpreter to run on most linux-gnu distros
      postBuild = 
        if crossSystem.cpu.name == "x86_64"
          then "patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 /build/source/target/release/mize"
        else if crossSystem.cpu.name == "aarch64"
          then "patchelf --set-interpreter /lib64/ld-linux-aarch64.so.2 /build/source/target/${crossSystem.nameFull}/release/mize"
          else ""
      ;

    }
    // (if crossSystem.name == "x86_64-linux-gnu" then builtins.trace "another x86_64-linux hackfix..." {} else {
      CARGO_BUILD_TARGET = crossSystem.nameFull;
      "CARGO_TARGET_${builtins.replaceStrings ["-"] ["_"] (pkgsCross.lib.strings.toUpper crossSystem.nameFull)}_LINKER" = "${pkgsCross.stdenv.cc.targetPrefix}cc";
      HOST_CC = "${pkgsCross.stdenv.cc.nativePrefix}cc";
      TARGET_CC = "${pkgsCross.stdenv.cc.targetPrefix}cc";
    })
    );

    main-win = craneLib.buildPackage {
      src = craneLib.cleanCargoSource ./.;

      #nativeBuildInputs = [ pkgsCross.buildPackages.nasm pkgsCross.buildPackages.cmake ];

      strictDeps = true;
      doCheck = false;

      cargoExtraArgs = "--bin mize --features os-target";

      CARGO_BUILD_TARGET = "${crossSystem.cpu.name}-pc-windows-gnu";

      MIZE_BUILD_CONFIG = mizeBildConfig;

      # fixes issues related to libring
      TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

      #fixes issues related to openssl
      OPENSSL_DIR = "${pkgs.openssl.dev}";
      OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
      OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

      depsBuildBuild = [
        pkgs.pkgsCross.mingwW64.stdenv.cc
        pkgs.pkgsCross.mingwW64.windows.pthreads
      ];
    };

    main-wasm = craneLib.buildPackage {
      src = "${self}";
      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
      doCheck = false;
      cargoExtraArgs = "--features wasm-target --no-default-features";
      MIZE_BUILD_CONFIG = mizeBildConfig;

      # env vars
      #CC = "${stdenv.cc.nativePrefix}cc";
      #AR = "${stdenv.cc.nativePrefix}ar";
      #CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_14.clang-unwrapped}/bin/clang-14";
      #CFLAGS_wasm32_unknown_unknown = "-I ${pkgs.llvmPackages_14.libclang.lib}/lib/clang/14.0.6/include/";
      #AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_14.llvm}/bin/llvm-ar";
    };



  ################# dev Shels ################### 

    mkMizeRustShell = attrs: mkMizeModuleShell (attrs // {
      #_shell_type = "rust";
      nativeBuildInputs = attrs.nativeBuildInputs or [] ++ [
        (fenix.packages.${localSystem}.combine [ 
          fenix.packages.${system}.stable.toolchain
          fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.toolchain
          fenix.packages.${system}.stable.toolchain
        ])
      ];

    });

    mkMizeModuleShell = attrs: pkgs.mkShell (attrs // {
      MIZE_BUILD_CONFIG = mizeBildConfig;
    });


    mainDevShell = pkgs.mkShell {
        buildInputs = with pkgs; [

          wasm-pack #pkg-config openssl #cargo rustc
          cargo-generate
          (fenix.packages.${localSystem}.combine [ 
            fenix.packages.${system}.stable.toolchain
            fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.toolchain
            fenix.packages.${system}.stable.toolchain
          ])
          openssl
        ];

        nativeBuildInputs = with pkgs; [
         pkg-config
        ];

        MIZE_BUILD_CONFIG = mizeBildConfig;

        shellHook = ''
          export MIZE_CONFIG_FILE=${self}/test-config.toml
        '';
    };



  ################# output attrset ################### 

    in rec {
      inherit toolchain_version_drv pkgsCross craneLib;

      devShell = mainDevShell;

      main = 
        if crossSystem.kernel.name == "windows"
          then main-win
        else if crossSystem.cpu.name == "wasm32"
          then main-wasm
        else main-default
        ;
  
      system = crossSystem;

      modulesListDrv = module_list_drv mize_modules;
      modulesFileList = pkgs.lib.lists.flatten (findModules mize_modules);
      modulesList = map (mod: (buildModule mod {})) modulesFileList;

      modules = builtins.listToAttrs ( map ( mod: { name = mod.modName; value = mod; } ) modulesList );
  };


  mkInstallPhase = mize: ''
    # install mize
    mkdir -p $out/mize/${mize.system.name}

    ${ 
    if mize.system.kernel.name == "windows"
      then "cp ${mize.main}/bin/mize.exe $out/mize/${mize.system.name}/"
    else if mize.system.cpu.name == "wasm32"
      then "cp ${mize.main}/lib/mize.wasm $out/mize/${mize.system.name}/"
      else "cp ${mize.main}/bin/mize $out/mize/${mize.system.name}/"
    }

    # intall the modules
    ${mkModulesInstallPhase mize.modulesList}
  '';

  mkModulesInstallPhase = modules: pkgs.lib.concatStringsSep "\n" (map mkModuleInstallPhase modules);

  mkModuleInstallPhase = module: ''
    echo got module: ${module.name}
    echo out: $out
    hash=$(echo ${module.selector_string} | sha256sum | cut -c -32)
    cp --no-preserve=mode,ownership -r ${module}/* $out/mize/dist/$hash-${module.modName}
    ${pkgs.gnutar}/bin/tar -czf $out/mize/dist/$hash-${module.modName}.tar.gz -C ${module} .
    echo '${module.selector_string}' > $out/mize/dist/$hash-${module.modName}/selector
  '';

  webfiles = systems: stdenv.mkDerivation {
    name = "mize-webfiles";
    dontUnpack = true;

    # so that those binaries run on average linux-gnu systems
    dontPatchShebangs = true;

    buildPhase = ''
    '';

    installPhase = ''
      mkdir -p $out
      mkdir -p $out/mize
      mkdir -p $out/mize/dist

    '' + pkgs.lib.concatStringsSep "\n" (map mkInstallPhase (map buildMizeForSystem systems));
    

	  nativeBuildInputs = [
	  ];
  };

  moduleShells = system: let
    mizeFor = buildMizeForSystem system;
  in builtins.listToAttrs ( map ( mod: { name = mod.modName; value = (mod.devShell); } ) mizeFor.modulesList );

}


