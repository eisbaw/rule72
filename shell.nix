{ pkgs ? (import (builtins.fetchTarball {
  url = "https://github.com/nixos/nixpkgs/tarball/25.05";
  sha256 = "1915r28xc4znrh2vf4rrjnxldw2imysz819gzhk9qlrkqanmfsxd";
}) {}) }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
    pkgs.clippy
    pkgs.rustfmt

    pkgs.cargo-audit
    pkgs.cargo-tarpaulin

    pkgs.less
    pkgs.colordiff
    pkgs.just
    pkgs.hyperfine
  ];
  # RUSTFLAGS left empty for default glibc target; enable musl separately if needed
} 
