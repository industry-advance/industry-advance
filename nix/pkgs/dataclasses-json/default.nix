{ pythonXXPackages, callPackage }:
let
  pname = "dataclasses-json";
  version = "0.5.1";
in pythonXXPackages.buildPythonPackage {
  pname = pname;
  version = version;
  src = pythonXXPackages.fetchPypi {
    inherit pname version;
    sha256 = "6e38b11b178e404124bffd6d213736bc505338e8a4c718596efec8d32eb96f5a";
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
