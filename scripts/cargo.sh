#!/bin/sh
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"
# Homebrew's cargo/rustc may precede rustup shims on the host PATH.
toolchain_bin=$(dirname "$(rustup which --toolchain 1.99.0 rustc)")
export PATH="$toolchain_bin:$PATH"
exec "$toolchain_bin/cargo" "$@"
