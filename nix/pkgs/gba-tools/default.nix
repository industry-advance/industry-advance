{ stdenv, libtool, autoconf, automake, which }:
stdenv.mkDerivation {
  pname = "gba-tools";
  version = "1.2.0";
  src = builtins.fetchTarball {
    url = "https://github.com/devkitPro/gba-tools/archive/v1.2.0.tar.gz";
    sha256 = "1rlhyc9dsdxcmaih3x9qjb3ihr2xxz1rw42ijbz2ylymn9p133gh";
  };

  preConfigure = ''
    ./autogen.sh --prefix=$out
  '';
  nativeBuildInputs = [ libtool autoconf automake which ];
}
