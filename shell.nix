{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
    pkgs.clippy
    pkgs.rustfmt
    pkgs.just
  ];
  # RUSTFLAGS left empty for default glibc target; enable musl separately if needed
} 