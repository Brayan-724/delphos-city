{
  inputs.nixpkgs.url = "github:nixos/nixpkgs";

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    devShells.${system}.default = pkgs.mkShell rec {
      packages = with pkgs; [
        pkg-config
        libudev-zero
        wayland
        seatd
        libinput
        pixman
        libxkbcommon
        libgbm
        libGL
        egl-wayland
      ];

      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
    };
  };
}
