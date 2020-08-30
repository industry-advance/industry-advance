{ pythonXXPackages, jdk }:
pythonXXPackages.buildPythonPackage rec {
  pname = "py2jdbc";
  version = "0.0.6";
  src = pythonXXPackages.fetchPypi {
    inherit pname version;
    sha256 = "cdef2517b18e56f64460443016d31767e0962e2528de0987b3d49e16777b8bbb";
  };
  buildInputs = with pythonXXPackages; [ six pytest jdk ];
  # Needed because the JNI is dynamically linked at runtime on lib init
  propagatedBuildInputs = [ jdk ];
}
