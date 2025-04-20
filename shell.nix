{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustPlatform.bindgenHook
    pkgs.pkg-config
    pkgs.diesel-cli
  ];
  buildInputs = [
    pkgs.cmake
    pkgs.libmysqlclient
    pkgs.libopus
    pkgs.ffmpeg
  ];
}

