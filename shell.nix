let
  sources = import ./nix/sources.nix;
  packageList = import ./nix/packagelist.nix;
  niv = import sources.niv { inherit sources; };
  nixpkgs = import sources.nixpkgs { };
  shellPackages = packageList.nixPackages ++ [
    # Dev tools for python scripts
    nixpkgs.python37Packages.black
    nixpkgs.python37Packages.flake8
    nixpkgs.python37Packages.pydocstyle
    nixpkgs.python37Packages.mypy
    nixpkgs.python37Packages.ipython

    # Debugging
    nixpkgs.gdb-multitarget

    # Nix maintenance
    niv.niv
  ];
in nixpkgs.mkShell { buildInputs = shellPackages; }
