let
  sources = import ./nix/sources.nix;
  packageList = import ./nix/packagelist.nix;
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
  ];
in nixpkgs.mkShell { buildInputs = shellPackages; }
