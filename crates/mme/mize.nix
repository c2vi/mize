{


module = { buildMizeForSystem, mizeBuildConfig, mkMizeRustModule, hostSystem, pkgsCross, mkMizeRustShell, pkgs, pkgsNative, buildNpmPackage, mkSelString, mizeBuildConfigStr, fenix, ... }: let

  selector_string = mkSelString {
    modName = "mme";
  };
  hash = builtins.substring 0 32 (builtins.hashString "sha256" selector_string);
  url = "file:///home/me/work/mme/dist/mize.js";
  html = ''
    <html>
      <head>
        <!-- script src="https://c2vi.dev/mize/wasm32-none-unknown/mize.js"></script --!>
        <!-- script src="${url}"></script --!>
        <script>
          console.log("from script")
        </script>
      </head>
      <body>
        hello world...............
      </body>  
    </html>
  '';

in mkMizeRustModule ({
  modName = "mme";
  src = ./.;
  cargoExtraArgs = "--no-default-features --lib";

  ## add the index.html
  postInstall = ''
    echo -n '${html}' > $out/index.html
  '';

}


// (if hostSystem.name == "wasm32-none-unknown" then {
  mizeBuildPhase = ''
    cd $build_dir
    RUSTFLAGS="-C link-args=--import-memory" RUST_LOG=off wasm-pack build --target no-modules --dev --out-dir $out -- --features wasm-target --no-default-features
    echo -n 'export function get_wasm_bindgen() { return wasm_bindgen }' >> $out/mize_module_mme.js
    echo -n '${html}' > $out/index.html
  '';
  mizeInstallPhase = "";
} else {})

// (if hostSystem.kernel.name == "linux" then builtins.trace "adding linux stuff" {
  nativeBuildInputs = with pkgsCross.buildPackages; [
    pkg-config
  ];
  buildInputs = with pkgsNative; [
    webkitgtk_4_1
  ];
} else {})


# add the devShell
// {
  devShell = pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
      #emscripten
      wasm-pack
      pkg-config
      webkitgtk_4_1
      libsForQt5.full
      cmake
      nasm
      pkg-config
      nodejs

      (fenix.packages.${system}.combine [
        fenix.packages.${system}.targets.wasm32-unknown-unknown.latest.toolchain
        fenix.packages.${system}.latest.toolchain
        fenix.packages.${system}.latest.rust-src
      ])

    ];

    MIZE_BUILD_CONFIG = mizeBuildConfigStr;

    buildInputs = with pkgs; [
      openssl
      #pango
      #libsoup_3
      webkitgtk_4_1
      # gobject-introspection gtk4 atkmm
    ];

    shellHook = ''
      echo hiiiiiiiiiiiiii
      export LD_LIBRARY_PATH=${pkgs.webkitgtk_4_1}/lib:${pkgs.libsoup_3}/lib:${pkgs.glib.out}/lib:${pkgs.gtk3}/lib:${pkgs.cairo}/lib:${pkgs.gdk-pixbuf}/lib:${pkgs.libxkbcommon}/lib:${pkgs.fontconfig.lib}/lib:${pkgs.libsForQt5.full}/lib:${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.libsForQt5.qt5.qtwebengine}/lib

      export CPLUS_INCLUDE_PATH=${pkgs.libsForQt5.full}/include:${pkgs.libsForQt5.qt5.qtwebengine}/include

      export MME_QT_LIB=${pkgs.libsForQt5.full}/lib

      # i found that this is the env war to set where QT looks for platform plugins
      # at: https://forums.fedoraforum.org/showthread.php?326508-How-to-set-QT_QPA_PLATFORM_PLUGIN_PATH
      export QT_QPA_PLATFORM_PLUGIN_PATH=${pkgs.libsForQt5.full}/lib/qt-5.15.14/plugins/platforms/
      
      alias run="${./.}/run.sh"
    '';

  };
}

);



lib = { mkMizeModule, buildNpmPackage, pkgsCross, pkgsNative, ... }: rec {
  mkMmePresenter = attrs: mkMizeModule (attrs // {
    src = attrs.src;
    modName = attrs.name;
    select = {
      mme_presenter = true;
    };
  });

  mkMmeNpmPresenter = attrs: mkMizeModule (attrs // {
    drvFunc = buildNpmPackage;
    mizeBuildPhase = ''
      cd $build_dir
      npm i
      npm run build
      mv dist tmp
      mkdir -p $out
      mv tmp/* $out
      rm -rf tmp
    '';
    mizeInstallPhase = "";
    select = {
      mme_presenter = true;
    };

    devShell = pkgsNative.mkShell {
      buildInputs = with pkgsNative; [ nodejs_20 ];
    };

  });

  mkMmeHtmlPresenter = attrs: mkMmePresenter ({
    dontUnpack = true;
    dontPath = true;
    buildPhase = "";
    installPhase = ''
      mkdir -p $out
      cp -r ${attrs.src}/* $out
    '';
  } // attrs);
};




externals = { fetchFromGitHub, ... }: [

  (fetchFromGitHub {
    owner = "c2vi";
    repo = "mme-presenters";
    rev = "master";
    hash = "sha256-FeMBDCJBkw9XOLXC1rfedNk71uyg4OTCHaaO1jAAGto=";
  })

];


}

