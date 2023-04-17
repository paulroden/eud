{
  pkgs ? import <nixpkgs> {} 
}:

pkgs.mkShell {
  # if os is darwin tho...
  nativeBuildInputs = [
    pkgs.darwin.IOKit
  ];
}
