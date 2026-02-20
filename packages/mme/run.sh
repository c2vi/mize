
set -e

# override, where mize loads the mme module from
export MIZE_MODULE_PATH=/home/me/work/mme:/home/me/work/modules:/home/me/work/presenters
export MIZE_MODULE_NO_REPO=1
export MIZE_MODULE_NO_EXTERNALS=1
export MIZE_CONFIG=$MIZE_CONFIG:module_dir.mme=/home/me/work/mme/dist:module_dir.String=/home/me/work/modules/modules/String/dist:module_dir.mme_presenter.mme_js=/home/me/work/presenters/presenters/mme_js/dist
#export MIZE_CONFIG=$MIZE_CONFIG:module_dir.mme=/home/me/work/mize/result


# build mme for the browser
echo '############################# MME for Browser #############################'
RUST_LOG=off wasm-pack build --target no-modules --dev --out-dir ./dist -- --features wasm-target --no-default-features


# build mize for the browser
echo '############################# mize for Browser #############################'
cd /home/me/work/mize
RUST_LOG=off wasm-pack build --target no-modules --dev -- --features wasm-target --no-default-features
cat /home/me/work/mize/src/platform/wasm/init.js >> /home/me/work/mize/pkg/mize.js

#--out-dir ~/work/mize/src/platform/wasm/npm_pkg/generated

#export CARGO_BUILD_RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals'
#wasm-pack build --target no-modules --dev . -Z build-std=panic_abort,std --features wasm-target --no-default-features --config 'build.rustflags = "-C target-feature=+atomics,+bulk-memory,+mutable-globals"'
#cp -r ~/work/mize/pkg/* ~/work/mize/src/platform/wasm/npm_pkg/generated
#RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' cargo build --target wasm32-unknown-unknown -Z build-std=panic_abort,std --lib --no-default-features
#cp ~/work/mize/target/wasm32-unknown-unknown/debug/mize.wasm ~/work/mize/src/platform/wasm/npm_pkg/generated/mize_bg.wasm


# build the mme-js presenter
echo '############################# mme-js presenter #############################'
cd /home/me/work/presenters/presenters/mme_js/
npm run build -- --mode development


# build the mme module
echo '############################# mme module for os #############################'
cd /home/me/work/mme
cargo build --lib
mkdir -p /home/me/work/mme/dist/lib
cp /home/me/work/mme/target/debug/libmize_module_mme.so /home/me/work/mme/dist/lib
nix eval --expr 'import /home/me/work/mme/src/implementors/html/index.nix { mize_url = "file:///home/me/work/mize/pkg/mize.js"; }' --impure --raw > /home/me/work/mme/dist/index.html


# build the String module
echo '############################# string module for os #############################'
cargo build --manifest-path ~/work/modules/modules/String/Cargo.toml --lib
mkdir -p /home/me/work/modules/modules/String/dist/lib
cp /home/me/work/modules/modules/String/target/debug/libmize_module_String.so /home/me/work/modules/modules/String/dist/lib


# run mize with gui
echo '############################# run mize for os #############################'
cargo run --manifest-path ~/work/mize/Cargo.toml -- gui


