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

    # Reference implementation
    nixpkgs.mindustry

    # For testing github actions locally
    nixpkgs.act

    # Debugging
    nixpkgs.gdb-multitarget
    nixpkgs.lldb_9
  ];
in nixpkgs.mkShell { buildInputs = shellPackages; }
