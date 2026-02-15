
# Use mize in a js project

## On the Web
```bash
npm init wasm-app project_dir
cd project_dir
npm install @c2vi/mize
```



## Build npm package yourself
### with nix
```bash
nix build github:c2vi/mize#npmPackage
```

### without nix
```bash
wasm-pack build -- --features wasm-target
```
