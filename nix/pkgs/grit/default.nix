{ stdenv, freeimage, libtool, autoconf, automake, which }:
stdenv.mkDerivation {
  name = "grit";
  version = "0.8.15";
  src = builtins.fetchTarball {
    url = "https://github.com/devkitPro/grit/archive/v0.8.15.tar.gz";
    sha256 = "100b9nn2h2hjivjwzh7w83ighd6ww8wbfiffyyzr7xidnz0bg327";
  };

  preConfigure = ''
    ./autogen.sh --prefix=$out
  '';
  buildInputs = [ freeimage ];
  nativeBuildInputs = [ libtool autoconf automake which ];
}
