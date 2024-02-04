{
  inputs = {
    # nixpkgs 35.11 still contains rust 1.73
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

 		flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { flake-utils, nixpkgs, ... }@inputs: flake-utils.lib.eachDefaultSystem (system: let pkgs = nixpkgs.legacyPackages.${system}; in {

      
    #packages.default =
    devShells.default = pkgs.mkShell {
      buildInputs = with pkgs; [ cargo rustc ];
    };
  });
}

