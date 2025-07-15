#!/usr/bin/env python3
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]

# Parse version from rule72/Cargo.toml
cargo_toml = root / "rule72" / "Cargo.toml"
with cargo_toml.open() as f:
    cargo_ver = None
    for line in f:
        m = re.match(r'version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"', line)
        if m:
            cargo_ver = m.group(1)
            break
if not cargo_ver:
    print("Could not find version in Cargo.toml", file=sys.stderr)
    sys.exit(1)

# Parse version from default.nix
nix_file = root / "default.nix"
with nix_file.open() as f:
    nix_ver = None
    for line in f:
        m = re.search(r'version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"', line)
        if m:
            nix_ver = m.group(1)
            break
if not nix_ver:
    print("Could not find version in default.nix", file=sys.stderr)
    sys.exit(1)

# Parse latest version from CHANGELOG.md
changelog = root / "CHANGELOG.md"
with changelog.open() as f:
    changelog_ver = None
    for line in f:
        m = re.match(r'## \[([0-9]+\.[0-9]+\.[0-9]+)\]', line)
        if m:
            changelog_ver = m.group(1)
            break
if not changelog_ver:
    print("Could not find version in CHANGELOG.md", file=sys.stderr)
    sys.exit(1)

# Parse version from CLI implementation (main.rs)
main_rs = root / "rule72" / "src" / "main.rs"
with main_rs.open() as f:
    main_ver = None
    for line in f:
        # Accept either a hard-coded version string or env!("CARGO_PKG_VERSION")
        m = re.search(r'\.version\("([0-9]+\.[0-9]+\.[0-9]+)"\)', line)
        if m:
            main_ver = m.group(1)
            break
        if 'env!("CARGO_PKG_VERSION")' in line:
            main_ver = cargo_ver
            break
if not main_ver:
    print("Could not find version in main.rs", file=sys.stderr)
    sys.exit(1)

versions = {
    "Cargo.toml": cargo_ver,
    "default.nix": nix_ver,
    "CHANGELOG.md": changelog_ver,
    "main.rs": main_ver,
}

if len(set(versions.values())) == 1:
    print(f"Version {cargo_ver} is consistent across files.")
    sys.exit(0)
else:
    print("Version mismatch detected:")
    for k, v in versions.items():
        print(f"  {k}: {v}")
    sys.exit(1)

