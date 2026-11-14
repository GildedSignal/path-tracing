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
  # Incremental (S104): the tree held 271,584 files and 1.9 GB, and a full pass before and after every
  # batch took longer than the batch's start. A stamp marks the last clean pass; only files modified
  # since are scanned. The stamp is taken BEFORE the scan, so a file written during it is rescanned next
  # time. Delete the stamp to force a full pass.
  local stamp="$dir/.scrubbed" next list
  next=$(mktemp) || return; touch "$next"
  hits=$(mktemp) || return
  if [[ -f $stamp ]]; then
    list=$(mktemp) || return
    find "$dir" -type f -newer "$stamp" ! -name .scrubbed -print0 > "$list" || return
    if [[ -s $list ]]; then
      # A grep that cannot read a file exits 2; turn that into 255 so xargs stops with 124, which
      # stays an error. Otherwise the hits file alone says whether any file held the key.
      K="$ZAI_API_KEY" xargs -0 sh -c 'grep -laFZ -- "$K" "$@"; r=$?; [ $r -le 1 ] || exit 255' _ < "$list" > "$hits" && status=0 || status=$?
      if [[ $status != 124 ]]; then [[ -s $hits ]] && status=0 || status=1; fi
    else
      status=1
    fi
    rm -f -- "$list"
  else
    grep -rlaFZ -- "$ZAI_API_KEY" "$dir" > "$hits" && status=0 || status=$?
  fi
  # 0: some files hold the key; 1: none do; anything else: a file could not be scanned.
  [[ $status == 0 || $status == 1 ]] || { rm -f -- "$hits" "$next"; echo 'Cannot scrub: a file could not be scanned' >&2; return 1; }
  while IFS= read -r -d '' file; do
    rm -f -- "$file" || { rm -f -- "$hits" "$next"; return 1; }
  done < "$hits"
  rm -f -- "$hits"
  mv -f -- "$next" "$stamp"
}
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  set -euo pipefail
  scrub_results "${1:?Usage: scrub.sh <results-directory>}"
fi
