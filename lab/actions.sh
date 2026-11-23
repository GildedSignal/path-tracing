#!/usr/bin/env bash
# workflow_dispatch host. No publishing, queue, schedule or git credentials.
set -euo pipefail

finalize() {
  local status=$? i source scrub_ok=true
  trap - EXIT
  for i in "${!ids[@]}"; do
    source=$host/worktree-$i/lab/results/${ids[$i]}
    if [[ -d $source && ! -L $source ]]; then
      cp -a -- "$source" lab/results/ || status=1
    fi
  done
  # Fail closed: scrub only the new run folders produced by this batch (never walk all of lab/results/).
  for i in "${!ids[@]}"; do
    if [[ -d lab/results/${ids[$i]} ]]; then
      if ! bash "$host/scrub.sh" "lab/results/${ids[$i]}"; then
        scrub_ok=false
      fi
    fi
  done
  if [[ $scrub_ok == true ]]; then
    for i in "${!ids[@]}"; do
      if [[ -f lab/results/${ids[$i]}/DONE ]]; then
        cat "lab/results/${ids[$i]}/DONE" >> "$GITHUB_STEP_SUMMARY"
        printf '\n' >> "$GITHUB_STEP_SUMMARY"
      fi
    done
    printf 'sanitized=true\n' >> "$GITHUB_OUTPUT"
  else
    echo 'Artifact upload refused: results could not be scrubbed' >&2
    status=1
  fi
  for i in "${!ids[@]}"; do
    for _ in 1 2 3 4; do
      git worktree remove --force "$host/worktree-$i" >/dev/null 2>&1 && break
      sleep 1
    done
  done
  rm -rf -- "$host" 2>/dev/null || true
  exit "$status"
}
main() {
  # Julien allowed research to run on his PC without isolation (S43); scripts/glm-local.ps1 sets GLM_LOCAL.
  [[ ${GITHUB_ACTIONS:-} == true || ${GLM_LOCAL:-} == S43 ]] || { echo 'GLM runs only on GitHub Actions or through scripts/glm-local.ps1' >&2; exit 2; }
  [[ -n ${LAB_KEY:-${ZAI_API_KEY:-}} ]] || { echo "the model's key is missing (LAB_KEY, or the ZAI_API_KEY Actions secret)" >&2; exit 2; }
  [[ ${LAB_MINUTES:-60} =~ ^[1-9][0-9]{0,2}$ ]] && (( ${LAB_MINUTES:-60} <= 180 )) || {
    echo 'minutes must be an integer from 1 to 180' >&2; exit 2;
  }
  local -a lenses pids
  local lens i result=0
  # Input is data, never interpolated into a shell program or evaluated.
  [[ ${LAB_LENSES:-} != *$'\n'* && ${LAB_LENSES:-} != *$'\r'* ]] || { echo 'lenses must be space-separated' >&2; exit 2; }
  read -r -a lenses <<< "${LAB_LENSES:-}"
  local max_lenses=3
  if [[ ${GLM_LOCAL:-} == S43 ]]; then max_lenses=24; fi
  (( ${#lenses[@]} >= 1 && ${#lenses[@]} <= max_lenses )) || {
    echo "Choose 1 to $max_lenses space-separated lenses" >&2; exit 2;
  }
  for lens in "${lenses[@]}"; do
    [[ $lens =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]*$ && $lens != README && -f jobs/$lens/prompt.md ]] || {
      echo 'Each name must be a job with jobs/<id>/prompt.md' >&2; exit 2;
    }
  done
  host=$(mktemp -d "$RUNNER_TEMP/glm-lab.XXXXXXXX")
  ids=()
  cp lab/scrub.sh "$host/scrub.sh"
  # Make all worktrees before launching any process that can alter a checkout.
  for i in "${!lenses[@]}"; do
    ids+=("${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}-${i}-${lenses[$i]}")
    for _ in 1 2 3 4; do
      git worktree add --quiet --detach "$host/worktree-$i" HEAD && break
      sleep 1
    done
    [[ -d $host/worktree-$i ]] || { echo "Failed to add worktree for ${lenses[$i]}" >&2; return 1; }
  done
  trap finalize EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  mkdir -p lab/results "$host/quota"
  timeout --signal=TERM --kill-after=1m 1m omp usage > "$host/quota/quota.txt" 2>&1 || {
    printf '\nQuota command failed or timed out; runs will still be attempted.\n' >> "$host/quota/quota.txt"
  }
  bash "$host/scrub.sh" "$host/quota"
  cp -f -- "$host/quota/quota.txt" lab/results/quota.txt || true
  [[ ! -f lab/results/quota.txt ]] || cat lab/results/quota.txt
  for i in "${!lenses[@]}"; do
    (
      cd "$host/worktree-$i"
      export CARGO_TARGET_DIR="$host/target-$i"
      bash lab/run.sh "${lenses[$i]}" "${ids[$i]}" "${LAB_MINUTES:-60}"
    ) > "$host/run-$i.log" 2>&1 &
    pids+=("$!")
  done
  # A failed run must not skip waiting for its peers, or prevent their artifact collection.
  for i in "${!pids[@]}"; do
    wait "${pids[$i]}" || result=1
    mkdir -p "lab/results/${ids[$i]}"
    cp -- "$host/run-$i.log" "lab/results/${ids[$i]}/host.log"
  done
  return "$result"
}
main "$@"
