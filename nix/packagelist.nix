let
  sources = import ./sources.nix;
  rust = import ./rust.nix { inherit sources; };
  grit = import ./grit.nix { inherit sources; };
  gba-tools = import ./gba-tools.nix { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
in {
  # List of packages which are needed both for testing and CI.
  nixPackages = [
    rust

    # Build tooling
    nixpkgs.cargo-make
    nixpkgs.cargo-xbuild
    nixpkgs.gcc-arm-embedded
    nixpkgs.cacert
    nixpkgs.clippy
    grit
    gba-tools

    # For running tests headlessly
    nixpkgs.mgba
    nixpkgs.xvfb_run
    nixpkgs.xorg.xauth
    nixpkgs.mesa
    nixpkgs.pulseaudio
    nixpkgs.libpulseaudio
    nixpkgs.bash

    # Execution of python glue
    nixpkgs.python37
    nixpkgs.python37Packages.pillow
  ];
}
