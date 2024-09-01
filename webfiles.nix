{ stdenv
, pkgs
, mkInstallPhase
, buildMizeForSystem
, ...
}: let

  # list of systems, we build mize for for webfiles
  systems = [ "x86_64-linux" "aarch64-linux" "wasm32" ];

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

