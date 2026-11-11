#!/usr/bin/env bash
# Source before starting agents, or run directly before artifact upload.
scrub_results() {
  local dir=$1 file status hits
  [[ -n ${ZAI_API_KEY:-} ]] || { echo 'Cannot scrub: ZAI_API_KEY is missing' >&2; return 2; }
  [[ -d $dir && ! -L $dir ]] || { echo 'Cannot scrub: results directory is missing or a symlink' >&2; return 2; }
  # Never upload symlinks or special files. Five MB means five binary megabytes.
  find "$dir" -mindepth 1 ! -type d ! -type f -delete || return
  find "$dir" -type f -size +5242880c -delete || return
  # One grep for the whole tree: a process per file took over an hour on 110,000 files.
  # -r skips symlinks below $dir (none remain), -a searches binary files too, -F treats
  # key characters literally, -l -Z lists each matching file once, NUL-separated.
  hits=$(mktemp) || return
  grep -rlaFZ -- "$ZAI_API_KEY" "$dir" > "$hits" && status=0 || status=$?
  # 0: some files hold the key; 1: none do; anything else: a file could not be scanned.
  [[ $status == 0 || $status == 1 ]] || { rm -f -- "$hits"; echo 'Cannot scrub: a file could not be scanned' >&2; return 1; }
  while IFS= read -r -d '' file; do
    rm -f -- "$file" || { rm -f -- "$hits"; return 1; }
  done < "$hits"
  rm -f -- "$hits"
}
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  set -euo pipefail
  scrub_results "${1:?Usage: scrub.sh <results-directory>}"
fi
