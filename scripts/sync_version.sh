#!/usr/bin/env bash
set -euo pipefail

version="${1:?usage: sync_version.sh <version>}"

apply() {
  local file="$1"
  shift
  local tmp
  tmp="$(mktemp)"
  sed "$@" "$file" > "$tmp"
  if cmp -s "$file" "$tmp"; then
    echo "error: no version replacement matched in $file" >&2
    rm -f "$tmp"
    exit 1
  fi
  cp "$tmp" "$file"
  rm -f "$tmp"
}

apply package.json "s|\"version\": \"[0-9][0-9.]*\"|\"version\": \"$version\"|"
apply src-tauri/tauri.conf.json "s|\"version\": \"[0-9][0-9.]*\"|\"version\": \"$version\"|"
apply src-tauri/Cargo.toml "s|^version = \".*\"|version = \"$version\"|"
apply src-tauri/Cargo.lock "/^name = \"FeedMee\"/{n;s|^version = \".*\"|version = \"$version\"|}"
apply PKGBUILD "s|^pkgver=.*|pkgver=$version|"
apply README.md "s|version-[0-9.]*-blue|version-$version-blue|"