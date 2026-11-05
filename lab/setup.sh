#!/usr/bin/env bash
# GitHub's Linux x64 runner; verify before replacing any installed executable.
set -euo pipefail
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || {
  echo 'omp setup requires Linux x64' >&2; exit 2;
}
dest=$HOME/.local/bin
mkdir -p "$dest"
download=$(mktemp "$dest/.omp.XXXXXXXX")
trap 'rm -f -- "$download"' EXIT
curl --fail --silent --show-error --location --connect-timeout 20 --max-time 120 \
  -o "$download" https://github.com/can1357/oh-my-pi/releases/download/v18.16.0/omp-linux-x64
if ! printf '%s  %s\n' 6498ed785bab00d2926e0387e0ec02362d6d45f1c3d0d1b57c1618b7f95c9264 "$download" \
  | sha256sum --check --status; then
  echo 'omp v18.16.0 SHA-256 mismatch; refusing to install' >&2
  exit 1
fi
chmod 755 "$download"
mv -f -- "$download" "$dest/omp"
echo 'Installed omp v18.16.0 in ~/.local/bin/omp'
