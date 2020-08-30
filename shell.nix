let
  sources = import ./nix/sources.nix;
  packageList = import ./nix/packagelist.nix;
  niv = import sources.niv { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
  shellPackages = packageList.nixPackages ++ [
    # Dev tools for python scripts
    nixpkgs.python38Packages.black
    nixpkgs.python38Packages.flake8
    nixpkgs.python38Packages.pydocstyle
    nixpkgs.python38Packages.mypy
    nixpkgs.python38Packages.ipython

    # Debugging
    nixpkgs.gdb-multitarget

    # Nix maintenance
    niv.niv
  ];
in nixpkgs.mkShell { buildInputs = shellPackages; }
