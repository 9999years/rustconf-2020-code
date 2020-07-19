{ pkgs ? import <nixpkgs> { } }:
let inherit (pkgs) stdenv lib;
in stdenv.mkDerivation rec {
  pname = "rustconf-code";
  version = "1.0.0";

  nativeBuildInputs = with pkgs; [ openssl pkgconfig ];
}
