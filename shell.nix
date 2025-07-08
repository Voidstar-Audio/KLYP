{
  pkgs ? import <nixpkgs> { },
}:
pkgs.callPackage (
  {
    stdenv,
    mkShell,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = with pkgs; [
      rustc
      cargo
      pkg-config
      python314
    ];
    buildInputs = with pkgs; [
      wayland
      
      libjack2
      alsa-lib

      libGL
      xorg.libX11
      xorg.libXcursor
      xorg.libxcb
      xorg.xcbutilwm
    ];
  }
) { }

