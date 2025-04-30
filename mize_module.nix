{

module = { mkMizeRustModule, hostSystem, pkgsCross, pkgs, lib, craneLib, ... }: 

  mkMizeRustModule (


  # general stuff
  {
    modName = "mize";
    src = ./.;
    cargoExtraArgs = "--no-default-features --lib";
  } 



  # linux stuff
  // (lib.attrsets.optionalAttrs (hostSystem.kernel.name == "linux") {
      strictDeps = true;
      cargoExtraArgs = "--bin mize --features os-target";

  })



  
  # x86_64-linux stuff
  // (lib.attrsets.optionalAttrs (hostSystem.name == "x86_64-linux-gnu") {


      nativeBuildInputs = [
        pkgsCross.buildPackages.pkg-config
      ];



      buildInputs = [
          (if hostSystem.nameFull == "x86_64-unknown-linux-gnu" then pkgs.openssl else pkgsCross.openssl)
      ];


      # patch the interpreter to run on most linux-gnu distros
      # postBuild = "patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 /build/source/target/release/mize";

  })

  # aarch64-linux stuff
  // (lib.attrsets.optionalAttrs (hostSystem.name == "aarch64-linux-gnu") {
      # patch the interpreter to run on most linux-gnu distros
      postBuild = "patchelf --set-interpreter /lib64/ld-linux-aarch64.so.2 /build/source/target/${hostSystem.nameFull}/release/mize";
  })




  #x86_64-windows stuff
  // (lib.attrsets.optionalAttrs (hostSystem.name == "x86_64-windows-gnu") {

    strictDeps = true;
    doCheck = false;

    cargoExtraArgs = "--bin mize --features os-target";

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



  #browser stuff
  // (lib.attrsets.optionalAttrs (hostSystem.name == "wasm32-none-unknown") {

    cargoArtifacts = craneLib.buildDepsOnly ({
      src = ./.;
      doCheck = false; # tests does not work in wasm
      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
      cargoExtraArgs = "--features wasm-target --no-default-features";
      RUSTFLAGS="-C linker=wasm-ld";
      buildInputs = with pkgs; [ cargo-binutils lld ];
    });

    doInstallCargoArtifacts = false;

    doCheck = false;

    buildPhaseCargoCommand = ''
        mkdir -p $out/pkg

        HOME=$(mktemp -d fake-homeXXXX) wasm-pack build --target no-modules --out-dir $out/pkg --scope=c2vi -- --features wasm-target --no-default-features
    '';

    postInstall = ''
    rm -rf $out/target.tar.zst

    cat $src/src/platform/wasm/init.js >> $out/pkg/mize.js
    '';

    buildInputs = with pkgs; [ wasm-bindgen-cli binaryen wasm-pack ];

  })

  );

}
