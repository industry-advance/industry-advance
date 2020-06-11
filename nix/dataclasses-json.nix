{ sources ? import ./sources.nix }:
with import sources.nixpkgs { };
with pkgs.python37Packages;
let typing_inspect = import ./typing_inspect.nix { inherit sources; };
in pkgs.python37Packages.buildPythonPackage rec {
  pname = "dataclasses-json";
  version = "0.5.1";
  src = fetchPypi {
    inherit pname version;
    sha256 = "6e38b11b178e404124bffd6d213736bc505338e8a4c718596efec8d32eb96f5a";
  };

  buildInputs = [
    typing_inspect
    marshmallow
    stringcase
    marshmallow-enum
    mypy-extensions
    typing-extensions
  ];

  propagatedBuildInputs = [
    typing_inspect
    marshmallow
    stringcase
    marshmallow-enum
    mypy-extensions
    typing-extensions
  ];
}
