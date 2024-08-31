{ stdenv
, pkgs
, osCrane
, self
, defaultMizeConfig
, mize_modules
, ...
}:
let

  # build all modules
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

  buildModule = path: (pkgs.callPackage path {
    craneLib = osCrane;
    toolchain_version = toolchain_version_string;
  });
  module_drv_list = map buildModule module_list;

  test2 = builtins.trace "list: ${module_drv_list}" "hi";
  test = builtins.trace "last: ${pkgs.lib.lists.last module_drv_list}" test2;

  webfilesBuildPhase = module_drv: ''
    echo got module: ${module_drv.name}
    hash=$(echo ${module_drv.selector_string} | sha256sum | cut -c -32)
    mkdir -p $out/mize/dist/$hash
    cp -r ${module_drv}/* $out/mize/dist/$hash
    echo '${module_drv.selector_string}' > $out/mize/dist/$hash/selector_string
  '';

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

  main = osCrane.buildPackage {
    src = "${self}";
    cargoExtraArgs = "--bin mize --features os-target";

      MIZE_BUILD_CONFIG = pkgs.writeTextFile {
        name = "mize-build-config";
        text = builtins.toJSON (defaultMizeConfig // { toolchain_version = toolchain_version_string; });
      };

    # patch the interpreter to run on most linux-gnu distros
    postBuild = ''
      patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 /build/source/target/release/mize
    '';
  };


in 

stdenv.mkDerivation {
  name = "mize-webfiles";
  dontUnpack = true;

  # so that /bin/sh does not get patched to a nix store path in victorinix-s
  dontPatchShebangs = true;

  buildPhase = ''
    mkdir -p $out
    mkdir -p $out/mize
    mkdir -p $out/mize/dist

    mkdir -p $out/mize/x86_64-linux-gnu
    cp -r ${main}/bin/mize $out/mize/x86_64-linux-gnu/mize

  '' + pkgs.lib.concatStringsSep "\n" (map webfilesBuildPhase module_drv_list);
    

	nativeBuildInputs = [
	];
}

