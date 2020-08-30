let
  sources = import ./sources.nix;
  rust = import ./rust.nix { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
in {
  # List of packages which are needed both for testing and CI.
  nixPackages = [
    rust

    # Build tooling
    nixpkgs.cargo-make
    nixpkgs.gcc-arm-embedded
    nixpkgs.cacert
    nixpkgs.clippy
    (nixpkgs.callPackage ./pkgs/grit { })
    (nixpkgs.callPackage ./pkgs/gba-tools { })

    # For running tests headlessly
    nixpkgs.mgba
    nixpkgs.xvfb_run
    nixpkgs.xorg.xauth
    nixpkgs.mesa
    nixpkgs.pulseaudio
    nixpkgs.libpulseaudio
    nixpkgs.bash

    # Execution of python glue
    nixpkgs.python38
    nixpkgs.python38Packages.pillow
    nixpkgs.python38Packages.fonttools
    nixpkgs.python38Packages.ffmpeg-python
    (nixpkgs.callPackage ./pkgs/py2jdbc {
      pythonXXPackages = nixpkgs.python38Packages;
    })
    (nixpkgs.callPackage ./pkgs/dataclasses-json {
      pythonXXPackages = nixpkgs.python38Packages;
    })
  ];
}
