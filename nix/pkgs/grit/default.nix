{ stdenv, freeimage, libtool, autoconf, automake, which }:
stdenv.mkDerivation {
  name = "grit";
  version = "0.8.15";
  src = builtins.fetchTarball {
    url = "https://github.com/devkitPro/grit/archive/v0.8.16.tar.gz";
    sha256 = "08pmxrn8accm32qcbj17csb28j4fnxjdvjs358ryzh19qs1gsdw9";
  };

  preConfigure = ''
    ./autogen.sh --prefix=$out
  '';
  buildInputs = [ freeimage ];
  nativeBuildInputs = [ libtool autoconf automake which ];
}
