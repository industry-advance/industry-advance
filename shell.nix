let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  grit = import ./nix/grit.nix { inherit sources; };
  nixpkgs = import sources.nixpkgs { };

  nixPackages = [
    rust

    # Build tooling
    nixpkgs.cargo-make
    nixpkgs.cargo-xbuild
    nixpkgs.gcc-arm-embedded
    nixpkgs.cacert
    nixpkgs.clippy
    grit

    # Needed to run MGBA headlessly for tests
    nixpkgs.xvfb_run
    nixpkgs.xorg.xauth
    nixpkgs.mesa
    nixpkgs.bash

    # Reference implementation
    nixpkgs.mindustry

    # Execution and maintenance of python glue
    nixpkgs.python37
    nixpkgs.python37Packages.black
    nixpkgs.python37Packages.flake8
    nixpkgs.python37Packages.pydocstyle
    nixpkgs.python37Packages.mypy

    # Debugging and testing
    nixpkgs.gdb-multitarget
    nixpkgs.mgba
    nixpkgs.xwayland
    nixpkgs.lldb_9
  ];
in nixpkgs.mkShell { buildInputs = nixPackages; }
