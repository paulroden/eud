{
  description = "Eudaemon: A kind helper for managing Emacs client & server instances.";

  inputs = {
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";  # TODO: specify ref.
    };
  };

  outputs = {
    self
  , flake-utils
  , nixpkgs
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.callPackage ./shell.nix { inherit pkgs; };
      }
    );
}
