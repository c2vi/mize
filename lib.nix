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

    # get a list of all modules
    # aka folders with a mize_module.nix in them
    module_list_drv = path: pkgs.stdenv.mkDerivation {
      name = "mize_module_list";
      dontUnpack = true;
      configurePhase = "";
      buildPhase = ''
        mkdir -p $out
        touch $out/modules_to_build

        find ${path} -not -type d -name mize_module.nix | while read p; do
          (echo "$p" >> $out/modules_to_build; echo "found module $p")
        done

        exit 0
      '';
    };

    mizeMdules = let
      from_mize_modules_repo = findModules mize_modules;
      mize_module_path = builtins.getEnv "MIZE_MODULE_PATH";
      mize_module_no_repo = builtins.getEnv "MIZE_MODULE_NO_REPO";
      dirs_from_path = pkgs.lib.lists.remove "" (pkgs.lib.strings.splitString ":" mize_module_path);
      dirs_in_nix_store = map (path: builtins.fetchGit {
          url = path;
        }) dirs_from_path;
      from_env_var = map findModules dirs_in_nix_store;
      in pkgs.lib.lists.flatten ((if mize_module_no_repo != "" then [] else from_mize_modules_repo) ++ from_env_var);


    findModules = path: let
      module_list_string = builtins.readFile "${module_list_drv path}/modules_to_build";
      module_list = pkgs.lib.lists.remove "" (pkgs.lib.strings.splitString "\n" module_list_string);
      getExternals = path: map findModules ((pkgs.callPackage ((import path).externals or (args: [])) {}));
      in ( builtins.trace "module_list: ${pkgs.lib.strings.concatStringsSep " --- " module_list}" module_list) ++ (map getExternals module_list);




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

      # rename the so library, in case the lib.name in your Cargo.toml is not like mize_module_mylib
      postInstall = (attrs.postInstall or "") + ''
        if [ ! -f "$out/lib/libmize_module_${attrs.modName}" ]; then
          echo RENAMING $out/lib/${attrs.modName}.so to $out/lib/libmize_module_${attrs.modName}.so
          mv $out/lib/${attrs.modName}.so to $out/lib/libmize_module_${attrs.modName}.so || true

          echo RENAMING $out/lib/${attrs.modName}.a to $out/lib/libmize_module_${attrs.modName}.a
          mv $out/lib/${attrs.modName}.a to $out/lib/libmize_module_${attrs.modName}.a && true # ignore if there is no .a file || true
          echo "$FILE"
        fi
      '';
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



    buildModule = path: extraArgs: pkgs.callPackage ((import path).module or (attrs: null)) ({
        inherit mkSelString craneLib toolchain_version;
        inherit mkMizeModule mkMizeRustModule buildModule findModules crossSystem pkgsCross pkgsNative mkMizeRustShell;
        mize_version = main-default.version;
      } // extraArgs );

    buildLib = path: extraArgs: pkgs.callPackage ((import path).lib or (args: {})) {
        inherit mkSelString craneLib toolchain_version;
        inherit mkMizeModule mkMizeRustModule buildModule findModules crossSystem pkgsCross pkgsNative mkMizeRustShell;
        mize_version = main-default.version;
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
        (fenix.packages."x86_64-linux".combine [ 
          fenix.packages."x86_64-linux".stable.toolchain
          fenix.packages."x86_64-linux".targets.wasm32-unknown-unknown.stable.toolchain
          fenix.packages."x86_64-linux".stable.toolchain
        ])

        # the shell script, to mk the dist folder, for a standard rust module
        (pkgs.writeShellApplication {
          name = "mkdist";
          # should we use the folder, that cargo locate-project gives us??? or just cp ./target/debug/
          text = ''
            mkdir -p ./dist
            mkdir -p ./dist/lib
            
            cp ./target/debug/libmize_module*.so ./dist/lib
            cp ./target/debug/libmize_module*.a ./dist/lib || true

          '';
        })
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
      modulesFileList = mizeMdules;
      modulesLibList = map (mod: buildLib mod {}) modulesFileList;
      modulesLib = pkgs.lib.lists.foldr (a: b: b // a) {} modulesLibList;
      modulesList = pkgs.lib.lists.remove null (map (mod: (buildModule mod {})) modulesFileList);

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

  mkModuleInstallPhase = module: let
   hash = builtins.substring 0 32 (builtins.hashString "sha256" module.selector_string);
  in ''
    echo got module: ${module.name}
    echo out: $out
    echo hash: ${hash}
    cp --no-preserve=mode,ownership -r ${module}/* $out/mize/dist/${hash}-${module.modName}
    ${pkgs.gnutar}/bin/tar -czf $out/mize/dist/${hash}-${module.modName}.tar.gz -C ${module} .
    echo '${module.selector_string}' > $out/mize/dist/${hash}-${module.modName}/selector
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


