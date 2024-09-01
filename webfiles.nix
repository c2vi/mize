{ stdenv
, pkgs
, osCrane
, self
, defaultMizeConfig
, mize_modules
, localSystem
, ...
}:
let

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
    crossSystem = if builtins.isString system then { config = system; } else system;
    craneLib = osCrane;

    mkSelString = attrs: builtins.toJSON (attrs // {
      toolchain_version = toolchain_version_string;
      mize_version = main.version;
    });

    buildModule = path: (pkgs.callPackage path {
      inherit mkSelString;
      craneLib = osCrane;
      toolchain_version = toolchain_version_string;
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

    modules = map buildModule module_list;
  };



  mkInstallPhase = mize: ''
    # install mize
    mkdir -p $out/mize/${mize.system.config}
    cp ${mize.main}/bin/mize $out/mize/${mize.system.config}/

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



  # list of systems, we build mize for
  systems = [ "x86_64-linux-gnu" ];

  list_of_mizes = map buildMizeForSystem systems;

in 

stdenv.mkDerivation {
  name = "mize-webfiles";
  dontUnpack = true;

  # so that /bin/sh does not get patched to a nix store path in victorinix-s
  dontPatchShebangs = true;

  buildPhase = "";

  InstallPhase = ''
    mkdir -p $out
    mkdir -p $out/mize
    mkdir -p $out/mize/dist

  '' + pkgs.lib.concatStringsSep "\n" (map mkInstallPhase list_of_mizes);
    

	nativeBuildInputs = [
	];
}

