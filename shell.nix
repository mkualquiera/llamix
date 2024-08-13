let
  nixpkgs-unstable = builtins.fetchTarball https://github.com/NixOS/nixpkgs/archive/nixpkgs-unstable.tar.gz;
  mozillaOverlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import nixpkgs-unstable { overlays = [ mozillaOverlay ]; };
  rust = (nixpkgs.rustChannelOf { channel = "stable"; }).rust.override {
    targets = [ "x86_64-unknown-linux-musl" ];
  };
in
{ pkgs ? import <nixpkgs> {
   config={
      allowUnfree=true;
      cudaSupport=true;
      packageOverrides = pkgs: {
         unstable = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {
          config.allowUnfree = true;
          config.packageOverrides = pkgs: {
            ollama = pkgs.ollama.override {
              acceleration = "cuda";
            };
          };
        }; 
      };
   };
} }:

pkgs.mkShell {
   buildInputs = with pkgs; [
      unstable.ollama
      cudaPackages.cudatoolkit
      gcc
      grub2
      qemu_kvm
      wget
      flex
      bison
      bc
      openssl.dev
      elfutils.dev
      libelf
      ncurses.dev
      xorriso
      rust
      glibc
   ]  ;
   nativeBuildInputs = with pkgs; [
      pkg-config
   ] ;
   shellHook = ''
   # add to PATH
   export PATH="$HOME/.cargo/bin:$PATH"
   '';
}