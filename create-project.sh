#!/usr/bin/env bash

PROJECT_NAME="${1:?}"
export PROJECT_NAME

if ! cargo init --bin "$PROJECT_NAME"; then
  echo "Could not run cargo init"
  exit 1
fi

(
cd "$PROJECT_NAME" || exit 42
cargo add --path ../aoc2023lib || exit 43
cargo add anyhow --features backtrace || exit 44
cargo add tracing --features valuable || exit 45
cargo add indicatif --features rayon,unicode-segmentation,improved_unicode,futures,vt100 || exit 46
)
