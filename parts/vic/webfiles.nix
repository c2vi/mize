{ stdenv
, pkgs
, url
, nixpkgs
, self
, system
, c2vi-config
, vicConfig
, getTarballClosure
, getVicorinix
, ... }:
let
  closure-x86_64-linux = getTarballClosure pkgs "x86_64-linux";

  closure-aarch64-linux = getTarballClosure pkgs "aarch64-linux";

  #victorinix-l = let pkgs = nixpkgs.legacyPackages.x86_64-linux; in pkgs.rustPlatform.buildRustPackage rec {
  victorinix-l = getVicorinix pkgs "x86_64-linux" "l" "sha256-0kAb+sieN+Ipnr8E3CS3oy+9+4qvUQU3rXrhpJyGTIM=";
  victorinix-la = getVicorinix pkgs "aarch64-multiplatform" "la" "sha256-eB/+tcI5+pWSMq2fIKI3qPcuRKOg0r1C3/wm999G8CE=";

  victorinix-s = pkgs.writeTextFile {
    name = "victorinix-s";
    executable = true;

    text = ''
      #!/bin/sh
      # This is a quick script that downloads the correct victorinix binary to ./vic
      # Programms needed in $PATH or /bin:
      # - sh (at /bin/sh)
      # - uname
      # - chmod
      # - wget or curl or python with urllib

      ##########################
      # check for needed things

      # add /bin to path, so that even if there is no path specified we can run if /bin/uname and /bin/chmod exist
      PATH=$PATH:/bin
      dev_null_replacement=$(command -v uname)
      if [[ "$?" == "0" ]]; then echo uname found; else echo uname not found; exit 1; fi
      dev_null_replacement=$(command -v chmod)
      if [[ "$?" == "0" ]]; then echo chmod found; else echo chmod not found; exit 1; fi


      ##########################
      # determine right executable
      arch=$(uname -m)
      kernel=$(uname -s)
      exepath=""
      if [[ "$arch" == "x86_64" ]] && [[ "$kernel" == "Linux" ]]; then
          exepath="l/vic"
      elif [[ "$arch" == "aarch64" ]] && [[ "$kernel" == "Linux" ]]; then
          exepath="la/vic"
      else
        echo "system (kernel: $kernel, arch: $arch) not supported"
        exit 1
      fi


      ##########################
      # get executable
      echo downloading victorinix binary at ${url}/$exepath

      function download_with_python(){
        python=$1
        $python -c '

      # i hate you python with your indents!!!
      from urllib.request import urlopen
      import sys

      with urlopen("${url}" + "/" + sys.argv[1]) as response:
        body = response.read()

      f = open("./vic", "wb")
      f.write(body)
      f.close()
        ' $exepath
      }

      function try_wget(){
        echo do we have wget??
        dev_null_replacement=$(command -v wget)
        found=$?
        if [[ "$found" == "0" ]]; then
          wget ${url}/$exepath
        else
          echo wget not found
        fi
        return $found
      }

      function try_curl(){
        echo do we have curl??
        dev_null_replacement=$(command -v curl)
        found=$?
        if [[ "$found" == "0" ]]; then
          curl ${url}/$exepath -o ./vic
        else
          echo curl not found
        fi
        return $found
      }

      function try_python(){
        echo do we have python??
        dev_null_replacement=$(command -v python)
        found=$?
        if [[ "$found" == "0" ]]; then
          download_with_python python
        else
          echo python not found
        fi
        return $found
      }

      function try_python3(){
        echo do we have python3??
        dev_null_replacement=$(command -v python3)
        found=$?
        if [[ "$found" == "0" ]]; then
          download_with_python python3
        else
          echo python3 not found
        fi
        return $found
      }

      function try_python2(){
        echo do we have python2??
        dev_null_replacement=$(command -v python2)
        found=$?
        if [[ "$found" == "0" ]]; then
          download_with_python python2
        else
          echo python2 not found
        fi
        return $found
      }

      function android_init_pre_download(){
        cd /data/local
      }

      function are_we_on_android(){
        dev_null_replacement=$(command -v getprop)
        if [[ "$?" == "0" ]]; then
          dev_null_replacement=$(getprop ro.build.version.release)
          if [[ "$?" == "0" ]]; then
            return 0
          fi
        fi
        return 1
      }

      # pre download platform specific init
      are_we_on_android && android_init_pre_download

      # download vic
      try_wget || try_curl || try_python || try_python3 || try_python2 || (echo ERROR: out of ways to download the victor binary; exit 1)

      ##########################
      # make it executable
      chmod +x ./vic

      # on android spawn a new sub-process, to keep the PWD at /data/local
      are_we_on_android && echo on ANDROID HISTORY LOST because we are in a sub-process, to keep the cd to /data/local && sh
    '';
  };
webfilesBuildPhase = closure: ''

  mkdir -p tar-tmp
  mkdir -p tar-tmp/nix

  while read path
  do
    echo path: $path
    mkdir -p tar-tmp$path
    cp -r --no-preserve=mode,ownership $path/* tar-tmp$path
  done < ${closure.info}/store-paths

  cp ${closure.proot}/bin/proot tar-tmp/nix/proot
  echo -en '${closure.nix}/bin/' > tar-tmp/nix/nix-path
  echo -en '${closure.busybox}/bin/' > tar-tmp/nix/busybox-path
  echo -en '${closure.cacert}' > tar-tmp/nix/cacert-path
  echo iiiiiiiiiiiiiiiii
  ls tar-tmp/nix
  echo iiiiiiiiiiiiiiiii

  tar -z -c -f $out/tars/${closure.system}.tar.gz -C tar-tmp --mode="a+rwx" .

  rm -rf tar-tmp
'';

in

stdenv.mkDerivation {
  name = "victorinix-webfiles";
  dontUnpack = true;

  # so that /bin/sh does not get patched to a nix store path in victorinix-s
  dontPatchShebangs = true;

  buildPhase = ''
    mkdir -p $out
    mkdir -p $out/l
    mkdir -p $out/la

    cp ${victorinix-s} $out/s

    cp ${victorinix-l}/bin/victorinix $out/l/vic
    cp ${victorinix-la}/bin/victorinix $out/la/vic

    mkdir -p $out/tars
    
    # make tars

  '' + webfilesBuildPhase closure-x86_64-linux
     + webfilesBuildPhase closure-aarch64-linux
  ;

    #${pkgs.gnutar}/bin/tar -C ./nix-store -czf $out/tars/x86_64-linux.tar.gz .

	nativeBuildInputs = [
	];
}

