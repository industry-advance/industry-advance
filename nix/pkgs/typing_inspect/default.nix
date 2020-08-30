{ pythonXXPackages }:
let
  pname = "typing_inspect";
  version = "0.6.0";
in pythonXXPackages.buildPythonPackage {
  pname = pname;
  version = version;
  src = pythonXXPackages.fetchPypi {
    inherit pname version;
    sha256 = "8f1b1dd25908dbfd81d3bebc218011531e7ab614ba6e5bf7826d887c834afab7";
  };

  buildInputs = with pythonXXPackages; [ mypy-extensions typing-extensions ];
}
