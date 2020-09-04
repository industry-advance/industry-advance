let
  sources = import ./sources.nix;
  rust = import ./rust.nix { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
in {
  nixPackages = [
    rust

    # Build tooling
    nixpkgs.gcc-arm-embedded
    nixpkgs.cargo-make
    (nixpkgs.callPackage ./pkgs/grit { })
    (nixpkgs.callPackage ./pkgs/gba-tools { })

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
