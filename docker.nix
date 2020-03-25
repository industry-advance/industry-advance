{ system ? builtins.currentSystem }:

let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  grit = import ./nix/grit.nix { inherit sources; };
  packageList = import ./nix/packagelist.nix;
  nixpkgs = import sources.nixpkgs { };

  callPackage = nixpkgs.lib.callPackageWith nixpkgs;

in nixpkgs.dockerTools.buildImage {
  name = "gba/industry-advance";
  tag = "latest";

  contents = packageList.nixPackages;

  config = {
    Cmd = [ "/bin/bash" ];
    WorkingDir = "/";
  };
}
