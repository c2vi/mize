{ stdenv
, pkgs
, ... }:
let

in

stdenv.mkDerivation {
  name = "mize-webfiles";
  dontUnpack = true;

  # so that /bin/sh does not get patched to a nix store path in victorinix-s
  dontPatchShebangs = true;

  buildPhase = ''
    mkdir -p $out
  '';

	nativeBuildInputs = [
	];
}

