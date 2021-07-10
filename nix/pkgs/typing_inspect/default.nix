{ pythonXXPackages }:
let
  pname = "typing_inspect";
  version = "0.7.1";
in pythonXXPackages.buildPythonPackage {
  pname = pname;
  version = version;
  src = pythonXXPackages.fetchPypi {
    inherit pname version;
    sha256 = "sha256-BH1Al9mxf0ZTG/bwFDVhEaG2+4IaJP56yQmFPKKngqo=";
  };

  buildInputs = with pythonXXPackages; [ mypy-extensions typing-extensions ];
}
