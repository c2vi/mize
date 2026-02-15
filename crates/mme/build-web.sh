
set -e

wasm-pack build --features wasm-target --no-default-features

cd /home/me/work/mme-presenters/presenters/mme-js/

npm run build -- --mode development
