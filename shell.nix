{ pkgs ? import <nixpkgs> {
   config={
      allowUnfree=true;
      cudaSupport=true;
      packageOverrides = pkgs: {
         ollama = pkgs.ollama.override {
            acceleration = "cuda";
         };
         unstable = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {
          config.allowUnfree = true;
        };
      };
   };
} }:

pkgs.mkShell {
   buildInputs = with pkgs; [
      ollama
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
      rustup
   ]  ;
   nativeBuildInputs = with pkgs; [
      pkg-config
   ] ;
   shellHook = ''
   # add to PATH
   export PATH="$HOME/.cargo/bin:$PATH"
   rustup target add x86_64-unknown-linux-musl
   '';
}