{ sources ? import ./sources.nix }:

with import sources.nixpkgs { };
with pkgs.python37Packages;

buildPythonPackage rec {
  pname = "typing_inspect";
  version = "0.6.0";
  src = fetchPypi {
    inherit pname version;
    sha256 = "8f1b1dd25908dbfd81d3bebc218011531e7ab614ba6e5bf7826d887c834afab7";
  };

  buildInputs =
    [ python37Packages.mypy-extensions python37Packages.typing-extensions ];
}
