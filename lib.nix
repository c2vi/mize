{ pkgs
, osCrane
, self
, defaultMizeConfig
, mize_modules
, buildSystem
, nixpkgs
, rust-overlay
, crane
, fenix
, stdenv
, lib ? pkgs.lib
, ...
}:
rec {


  

  getCrossSystem = system: let
    hostSystem = (if builtins.isString system then (pkgs.lib.systems.parse.mkSystemFromString system) else system)
      // rec {
        name = if hostSystem.vendor != "unknown" 
          then "${hostSystem.cpu.name}-${hostSystem.kernel.name}-${hostSystem.abi.name}"
          else "${hostSystem.cpu.name}-${hostSystem.vendor.name}-${hostSystem.kernel.name}-${hostSystem.abi.name}";
        nameFull = "${hostSystem.cpu.name}-${hostSystem.vendor.name}-${hostSystem.kernel.name}-${hostSystem.abi.name}";
        nameRust = nameFull;
        config = nameFull;
        isCross = system != buildSystem;
        isNative = system == buildSystem;
      };
  in hostSystem;


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

  findModules = path: let
    mize_module_no_externals = builtins.getEnv "MIZE_MODULE_NO_EXTERNALS";
    module_list_string = builtins.readFile "${module_list_drv path}/modules_to_build";
    module_list = pkgs.lib.lists.remove "" (pkgs.lib.strings.splitString "\n" module_list_string);
    getExternals = path: map findModules ((pkgs.callPackage ((import path).externals or (args: [])) {}));
    in  module_list ++ (if mize_module_no_externals != "" then [] else (map getExternals module_list));


  modulesFileList = let
    from_mize_modules_repo = findModules mize_modules;
    mize_module_no_repo = builtins.getEnv "MIZE_MODULE_NO_REPO";
    in pkgs.lib.lists.flatten (from_env_var ++ (if mize_module_no_repo != "" then [] else from_mize_modules_repo) ++ (findModules self));

  mize_module_path = builtins.getEnv "MIZE_MODULE_PATH";
  dirs_from_path = pkgs.lib.lists.remove "" (pkgs.lib.strings.splitString ":" mize_module_path);
  dirs_in_nix_store = map (path: builtins.fetchGit {
      url = path;
    }) dirs_from_path;
  from_env_var = map findModules dirs_in_nix_store;



  buildMizeForSystem = system: let
    hostSystem = getCrossSystem system;

    pkgsCross = import nixpkgs { inherit buildSystem hostSystem; localSystem = buildSystem; overlays = [ rust-overlay.overlays.default ]; };

    pkgsNative = import nixpkgs { inherit buildSystem; overlays = [ rust-overlay.overlays.default ]; };




    craneLib = 
      if hostSystem.kernel.name == "windows"
        then 
          (crane.mkLib pkgs).overrideToolchain (fenix.packages.${buildSystem}.combine [
            #fenix.packages.${buildSystem}.minimal.rustc
            #fenix.packages.${buildSystem}.minimal.cargo
            fenix.packages.${buildSystem}.stable.toolchain
            fenix.packages.${buildSystem}.targets."${hostSystem.cpu.name}-pc-windows-gnu".stable.toolchain
          ])

      else if hostSystem.cpu.name == "wasm32"
        then
          (crane.mkLib pkgs).overrideToolchain (fenix.packages.${buildSystem}.combine [
            fenix.packages.${buildSystem}.targets.wasm32-unknown-unknown.latest.toolchain
            fenix.packages.${buildSystem}.latest.toolchain
          ])
      else if hostSystem.name == "x86_64-linux-gnu"
        then
          builtins.trace "x86_64-linux-gnu hackfix...."
          (crane.mkLib pkgs).overrideToolchain (fenix.packages.${buildSystem}.combine [
            fenix.packages.${buildSystem}.stable.toolchain
          ])

      else
          (crane.mkLib pkgsCross).overrideToolchain (p: fenix.packages.${buildSystem}.stable.toolchain)
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
      system = hostSystem.nameFull;
      mize_version = "0.0.1";
    });


    toolchain_version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile "${toolchain_version_drv}/rustc-version");


    mkMizeModule = attrs: (if builtins.hasAttr "drvFunc" attrs then attrs.drvFunc else pkgsCross.stdenv.mkDerivation) ( rec {
      name = attrs.modName;
      inherit (attrs) modName;
      selector_string = mkSelString (attrs.select or {} // {
        inherit (attrs) modName;
      });
      hash = builtins.substring 0 32 (builtins.hashString "sha256" selector_string);
    } 
    // (lib.attrsets.removeAttrs attrs ["drvFunc"] )
    );



    ########## build a Rust Module
    mkMizeRustModule = attrs: mkMizeModule (
      # general stuff
      {
        drvFunc = craneLib.buildPackage;
        MIZE_BUILD_CONFIG = mizeBuildConfigStr;
        mizeInstallPhase = attrs.mizeInstallPhase or ''
          mkdir -p $out/lib/
          cp $build_dir/target/${hostSystem.nameRust}/$debugOrRelease/libmize_module_${attrs.modName}.so $out/lib/
        '';
        mizeBuildPhase = attrs.mizeBuildPhase or ''
          cargo --color always build --target ${hostSystem.nameRust} ${if builtins.hasAttr "cargoExtraArgs" attrs then attrs.cargoExtraArgs else ""} --manifest-path $build_dir/Cargo.toml --lib
        '';
        selector_string = mkSelString (attrs.select or {} // {
          inherit (attrs) modName;
        });
        devShell = if builtins.hasAttr "devShell" attrs then attrs.devShell else mkMizeRustShell {};

      }


      # linux specific stuff
      // (lib.attrsets.optionalAttrs (hostSystem.kernel.name == "linux") {
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
      })
      

      # linux cross stuff
      // (lib.attrsets.optionalAttrs (hostSystem.kernel.name == "linux" && hostSystem.isCross) {
        "CARGO_TARGET_${builtins.replaceStrings ["-"] ["_"] (pkgsCross.lib.strings.toUpper hostSystem.nameFull)}_LINKER" = "${pkgsCross.stdenv.cc.targetPrefix}cc";
        HOST_CC = "${pkgsCross.stdenv.cc.nativePrefix}cc";
        TARGET_CC = "${pkgsCross.stdenv.cc.targetPrefix}cc";
        CARGO_BUILD_TARGET = hostSystem.nameFull;
        nativeBuildInputs = attrs.nativeBuildInputs or [] ++ [
          pkgsCross.stdenv.cc
        ];
      })

      # wasm stuff
      // (lib.attrsets.optionalAttrs (hostSystem.cpu.name == "wasm32") {
        CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

        # checks fail on wasm
        doCheck = false;
      })


      # add windows stuff
      // (lib.attrsets.optionalAttrs (hostSystem.kernel.name == "windows") {
        strictDeps = true;
        doCheck = false;

        CARGO_BUILD_TARGET = "${hostSystem.cpu.name}-pc-windows-gnu";

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
      })


      # add the attrs passed to mkMizeRustModule
      // attrs

    );



    buildModule = path: extraArgs: pkgs.callPackage ((import path).module or (attrs: null)) ({
        inherit buildMizeForSystem mizeBuildConfig mizeBuildConfigStr;
        inherit mkSelString craneLib toolchain_version;
        inherit mkMizeModule mkMizeRustModule buildModule findModules hostSystem pkgsCross pkgsNative mkMizeRustShell;
        mize_version = "0.0.1";
      } // extraArgs );

    buildLib = path: extraArgs: pkgs.callPackage ((import path).lib or (args: {})) {
        inherit mkSelString craneLib toolchain_version;
        inherit mkMizeModule mkMizeRustModule buildModule findModules hostSystem pkgsCross pkgsNative mkMizeRustShell;
        mize_version = "0.0.1";
    };


    mizeBuildConfig = {
        namespace = "mize.buildtime.ns";
        module_url = "c2vi.dev";
        selector = builtins.fromJSON (mkSelString {});
    };

    mizeBuildConfigStr = let
      settingsFormat = pkgs.formats.toml { };
    in settingsFormat.generate "mize-build-config.toml" {
      config = mizeBuildConfig;
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
      MIZE_BUILD_CONFIG = mizeBuildConfigStr;
    });


    mainDevShell = pkgs.mkShell {
        buildInputs = with pkgs; [

          wasm-pack #pkg-config openssl #cargo rustc
          cargo-generate
          (fenix.packages.${buildSystem}.combine [ 
            fenix.packages.${system}.stable.toolchain
            fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.toolchain
            fenix.packages.${system}.stable.toolchain
          ])
          openssl
        ];

        nativeBuildInputs = with pkgs; [
         pkg-config
        ];

        MIZE_BUILD_CONFIG = mizeBuildConfigStr;

        shellHook = ''
          export MIZE_CONFIG_FILE=${self}/test-config.toml
        '';
    };



  ################# output attrset ################### 

    in rec {

      devShell = mainDevShell;

      main = modules.mize;
  
      system = hostSystem;


      modules = builtins.listToAttrs ( map ( mod: { name = mod.modName; value = mod; } ) modulesList );

      # for debugging

      modulesLibList = map (mod: buildLib mod {}) modulesFileList;
      modulesLib = pkgs.lib.lists.foldr (a: b: b // a) {} modulesLibList;
      modulesList = pkgs.lib.lists.remove null (map (mod: (buildModule mod modulesLib)) modulesFileList);

      inherit toolchain_version_drv pkgsCross craneLib;

      inherit mize_module_path dirs_from_path dirs_in_nix_store from_env_var;

  };


  mkInstallPhase = mize: ''
    # install mize
    mkdir -p $out/mize/${mize.system.name}

    cp -r ${mize.main}/* $out/mize/${mize.system.name}/

    # intall the modules
    ${mkModulesInstallPhase mize.modulesList}
  '';

  mkModulesInstallPhase = modules: pkgs.lib.concatStringsSep "\n" (map mkModuleInstallPhase modules);

  mkModuleInstallPhase = module: let
   hash = module.hash; 
  in ''
    echo got module: ${module.name}
    echo out: $out
    echo hash: ${hash}
    cp --no-preserve=mode,ownership -r ${module}/* $out/mize/dist/${hash}-${module.modName}
    ${pkgs.gnutar}/bin/tar -czf $out/mize/dist/${hash}-${module.modName}.tar.gz -C ${module} .
    echo '${module.selector_string}' > $out/mize/dist/${hash}-${module.modName}/selector
  '';

  dist = systems: stdenv.mkDerivation {
    name = "mize-dist";
    dontUnpack = true;

    # so that those binaries run on average linux-gnu systems
    dontPatchShebangs = true;

    buildPhase = ''
    '';

    installPhase = ''
      mkdir -p $out
      mkdir -p $out/mize
      mkdir -p $out/mize/dist

    '' ;# + pkgs.lib.concatStringsSep "\n" (map mkInstallPhase (map buildMizeForSystem systems));
    

	  nativeBuildInputs = [
	  ];
  };

  moduleShells = system: let
    mizeFor = buildMizeForSystem system;
  in builtins.listToAttrs ( map ( mod: { name = mod.modName; value = (mod.devShell); } ) mizeFor.modulesList );

  random = systems: rec {

    distInstallPhase = map mkInstallPhase (map buildMizeForSystem systems);
    mizeList = map buildMizeForSystem systems;
    mize-x86_64-linux = buildMizeForSystem "x86_64-linux";
    mize-x86_64-linux-installPhase = mkInstallPhase mize-x86_64-linux;
    modulesListDrv = module_list_drv mize_modules;
    inherit modulesFileList;

  };

}


