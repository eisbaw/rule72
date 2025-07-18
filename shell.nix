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

    pkgs.less
    pkgs.colordiff
    pkgs.just
    pkgs.hyperfine

    # Rust source code static analysis
    pkgs.cargo-audit
    pkgs.cargo-deny
  ];
  # RUSTFLAGS left empty for default glibc target; enable musl separately if needed
} 