{ pkgs ? import <nixpkgs> {} }:
let
  overrides =  pkgs.defaultCrateOverrides // {
      libsodium-sys = attrs: {
        nativeBuildInputs = [ pkgs.pkgconfig ];
        buildInputs = [ pkgs.libsodium ];
      };
      openssl-sys-extras = attrs: {
        nativeBuildInputs = [ pkgs.pkgconfig ];
        buildInputs = [ pkgs.openssl ];
      };
  };
in
(pkgs.callPackage ./Cargo.nix {defaultCrateOverrides=overrides;}).rootCrate.build
