{ stdenv
, pkgs
, osCrane
, self
, defaultMizeConfig
, mize_modules
, localSystem
, nixpkgs
, rust-overlay
, crane
, ...
}:
rec {
  # get a list of all modules, that have a mize_module.nix in their folder
  module_list_drv = pkgs.stdenv.mkDerivation {
    name = "mize_module_list";
    dontUnpack = true;
    configurePhase = "";
    buildPhase = ''
      mkdir -p $out
      touch $out/modules_to_build

      for d in ${mize_modules}/modules/*; do
        [[ -f $d/mize_module.nix ]] && echo "$d/mize_module.nix" >> $out/modules_to_build
      done

      exit 0
    '';
  };
  module_list_string = builtins.readFile "${module_list_drv}/modules_to_build";
  module_list = pkgs.lib.lists.remove "" (pkgs.lib.strings.splitString "\n" module_list_string);



  toolchain_version_drv = pkgs.stdenv.mkDerivation {
    name = "toolchain_version_drv";
    dontUnpack = true;
    configurePhase = "";
    buildPhase = ''
      mkdir -p $out
      touch $out/rustc-version

      ${osCrane.rustc}/bin/rustc --version > $out/rustc-version

      exit 0
    '';
  };
  toolchain_version_string = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile "${toolchain_version_drv}/rustc-version");



  buildMizeForSystem = system: let
    crossSystem = (if builtins.isString system then pkgs.lib.systems.mkSystemFrommString system else system)
      // {
        name = if crossSystem.vendor == "unknown" 
          then "${crossSystem.cpu.name}-${crossSystem.kernel.name}-${crossSystem.abi.name}"
          else "${crossSystem.cpu.name}-${crossSystem.vendor.name}-${crossSystem.kernel.name}-${crossSystem.abi.name}";
        nameFull = "${crossSystem.cpu.name}-${crossSystem.vendor.name}-${crossSystem.kernel.name}-${crossSystem.abi.name}";
      };
    pkgsCross = import nixpkgs { inherit crossSystem localSystem; overlays = [ rust-overlay ]; };

    craneLib = (crane.mkLib pkgsCross).overrideToolchain (p: p.rust-bin.stable.latest.default);

    mkSelString = attrs: builtins.toJSON (attrs // {
      toolchain_version = toolchain_version_string;
      mize_version = main.version;
    });

    mkMizeModule = attrs: {
      a = builtins.error "todo!()";
    };

    mkMizeRustModule = attrs: craneLib.buildPackage attrs // {
      cargoExtraArgs = "--target ${crossSystem.nameFull} " + attrs.cargoExtraArgs or "";
      "CARGO_TARGET_${pkgsCross.lib.strings.toUpper crossSystem.nameFull}_LINKER" = "${pkgsCross.stdenv.cc.targetPrefix}cc";
      HOST_CC = "${pkgsCross.stdenv.cc.nativePrefix}cc";
      TARGET_CC = "${pkgsCross.stdenv.cc.targetPrefix}cc";
    };

    buildModule = path: (pkgsCross.callPackage path {
      inherit mkSelString craneLib;
      inherit mkMizeModule mkMizeRustModule;
      toolchain_version = toolchain_version_string;
      mize_version = main.version;
    });

    main = craneLib.buildPackage {
      src = "${self}";
      cargoExtraArgs = "--bin mize --features os-target";

      MIZE_BUILD_CONFIG = pkgs.writeTextFile {
        name = "mize-build-config";
        text = builtins.toJSON (defaultMizeConfig // {
          toolchain_version = toolchain_version_string;
          mize_version = main.version;
        });
      };
      # patch the interpreter to run on most linux-gnu distros
      postBuild = ''
        patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 /build/source/target/release/mize
      '';
    };
    in rec {
      inherit main;
  
      system = crossSystem;

      modules = pkgs.lib.lists.flatten (map buildModule module_list);
  };



  mkInstallPhase = mize: ''
    # install mize
    mkdir -p $out/mize/${mize.system.name}
    cp ${mize.main}/bin/mize $out/mize/${mize.system.name}/

    # intall the modules
    ${mkModulesInstallPhase mize.modules}
  '';

  mkModulesInstallPhase = modules: pkgs.lib.concatStringsSep "\n" (map (module: ''
    echo got module: ${module.name}
    hash=$(echo ${module.selector_string} | sha256sum | cut -c -32)
    mkdir -p $out/mize/dist/$hash
    cp -r ${module}/* $out/mize/dist/$hash
    echo '${module.selector_string}' > $out/mize/dist/$hash/selector_string
  '')) modules;


}


