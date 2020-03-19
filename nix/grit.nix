{ sources ? import ./sources.nix }:

with import sources.nixpkgs { };

stdenv.mkDerivation rec {
  pname = "grit";
  version = "0.8.15";
  src = fetchTarball {
    url = "https://github.com/devkitPro/grit/archive/v0.8.15.tar.gz";
    sha256 = "100b9nn2h2hjivjwzh7w83ighd6ww8wbfiffyyzr7xidnz0bg327";
  };

  preConfigure = ''
    ./autogen.sh --prefix=$out
  '';
  buildInputs = [ pkgs.freeimage ];
  nativeBuildInputs = [ pkgs.libtool pkgs.autoconf pkgs.automake pkgs.which ];
}
