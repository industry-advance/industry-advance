# This is a minimal nix-shell without dependencies needed to run tests, debug and other niceties.
# Useful on extremely crappy internet connections.

let
  sources = import ./nix/sources.nix;
  packageList = import ./nix/essential-packages.nix;
  nixpkgs = import sources.nixpkgs { };
in nixpkgs.mkShell {
  buildInputs = packageList.nixPackages;
}
