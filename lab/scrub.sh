#!/usr/bin/env bash
# Source before starting agents, or run directly before artifact upload.
scrub_results() {
  local dir=$1 file status
  [[ -n ${ZAI_API_KEY:-} ]] || { echo 'Cannot scrub: ZAI_API_KEY is missing' >&2; return 2; }
  [[ -d $dir && ! -L $dir ]] || { echo 'Cannot scrub: results directory is missing or a symlink' >&2; return 2; }
  # Never upload symlinks or special files. Five MB means five binary megabytes.
  find "$dir" -mindepth 1 ! -type d ! -type f -delete || return
  find "$dir" -type f -size +5242880c -delete || return
  while IFS= read -r -d '' file; do
    # -a searches binary files too; -F treats key characters literally.
    if grep -aFq -- "$ZAI_API_KEY" "$file"; then
      rm -f -- "$file" || return
    else
      status=$?
      [[ $status == 1 ]] || { echo 'Cannot scrub: a file could not be scanned' >&2; return 1; }
    fi
  done < <(find "$dir" -type f -print0)
}
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  set -euo pipefail
  scrub_results "${1:?Usage: scrub.sh <results-directory>}"
fi
