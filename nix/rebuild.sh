#!/usr/bin/env bash
set -euo pipefail

version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
echo " niri-sidebar v$version"

echo "Updating Cargo.lock..."
cargo generate-lockfile

echo "Updating flake.lock..."
nix flake update

echo "Building..."
nix build

echo "Done - result at ./result ($(./result/bin/niri-sidebar --version))"
