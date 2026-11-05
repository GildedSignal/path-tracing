#!/usr/bin/env bash
# Install omp (GLM's command-line agent), pinned by version and SHA-256. Linux only (agonbox as glmlab, or GitHub's machines).
set -euo pipefail
V=18.1.18
case $(uname -m) in
  x86_64)  A=x64;   S=45421f9a5f112bc47cb9f77c4b4d7927631f8ff859624f821287ec854eb239fc ;;
  aarch64) A=arm64; S=1ae8273c231ceb88cebc9971901cf7f5d97ed4149cdc740a814f629cd2b4dcb2 ;;
  *) echo "unsupported machine $(uname -m)"; exit 2 ;;
esac
D=$HOME/.local/bin; mkdir -p "$D"
curl -fsSL -o "$D/omp" "https://github.com/can1357/oh-my-pi/releases/download/v$V/omp-linux-$A"
echo "$S  $D/omp" | sha256sum -c - >/dev/null
chmod +x "$D/omp"
echo "omp $V installed in $D"
