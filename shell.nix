
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustnightly = (nixpkgs.latest.rustChannels.nightly.rust.override { extensions = [ "rust-src" "rls-preview" "rust-analysis" "rustfmt-preview" ];});

  nixPackages = [
	rustnightly
	nixpkgs.cargo-make
	nixpkgs.cargo-xbuild
	nixpkgs.gcc-arm-embedded
	nixpkgs.cacert

	nixpkgs.mindustry

    nixpkgs.python3

    nixpkgs.gdb
	nixpkgs.mgba
  ];
in
nixpkgs.mkShell {
  buildInputs = nixPackages;
}
