{ pkgs ? import <nixpkgs> {} }:
let
  cargo2nix = import (
    pkgs.fetchFromGitHub {
      owner = "kolloch";
      repo = "crate2nix";
      rev = "0fd65ca7def611ad9d0ee532be0954d4b0bbb4d4";
      sha256 = "17mmf5sqn0fmpqrf52icq92nf1sy5yacwx9vafk43piaq433ba56";
      fetchSubmodules = false;
    }
    ) {pkgs=pkgs;};
in
pkgs.mkShell {
  name = "Eventbot";
  buildInputs = with pkgs; 
  [ rustc 
    cargo 
    rls 
    rustup 
    openssl 
    pkgconfig
    aws-sam-cli
    docker-compose
  ];
}
