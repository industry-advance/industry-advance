{ pythonXXPackages, callPackage }:
let
  pname = "dataclasses-json";
  version = "0.5.4";
in pythonXXPackages.buildPythonPackage {
  pname = pname;
  version = version;
  src = pythonXXPackages.fetchPypi {
    inherit pname version;
    sha256 = "sha256-bDl2gW/TzdjbO+K1FrZPwIOs1GrCLGgNPcJMsdauM2c=";
  };

  buildInputs = with pythonXXPackages; [
    (callPackage ../typing_inspect { pythonXXPackages = pythonXXPackages; })
    marshmallow
    stringcase
    marshmallow-enum
    mypy-extensions
    typing-extensions
  ];

  propagatedBuildInputs = with pythonXXPackages; [
    (callPackage ../typing_inspect { pythonXXPackages = pythonXXPackages; })
    marshmallow
    stringcase
    marshmallow-enum
    mypy-extensions
    typing-extensions
  ];
}
