#!/usr/bin/env bash
# workflow_dispatch host. No publishing, queue, schedule or git credentials.
set -euo pipefail

finalize() {
  local status=$? i source
  trap - EXIT
  for i in "${!ids[@]}"; do
    source=$host/worktree-$i/lab/results/${ids[$i]}
    if [[ -d $source && ! -L $source ]]; then
      cp -a -- "$source" lab/results/ || status=1
    fi
  done
  # Fail closed: upload is enabled only after every collected file was scanned.
  if bash "$host/scrub.sh" lab/results; then
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
  exit "$status"
}
main() {
  # Julien allowed research to run on his PC without isolation (S43); scripts/glm-local.ps1 sets GLM_LOCAL.
  [[ ${GITHUB_ACTIONS:-} == true || ${GLM_LOCAL:-} == S43 ]] || { echo 'GLM runs only on GitHub Actions or through scripts/glm-local.ps1' >&2; exit 2; }
  [[ -n ${ZAI_API_KEY:-} ]] || { echo 'ZAI_API_KEY Actions secret is missing; set it before running glm-lab' >&2; exit 2; }
  [[ ${LAB_MINUTES:-60} =~ ^[1-9][0-9]{0,2}$ ]] && (( ${LAB_MINUTES:-60} <= 180 )) || {
    echo 'minutes must be an integer from 1 to 180' >&2; exit 2;
  }
  local -a lenses pids
  local lens i result=0
  # Input is data, never interpolated into a shell program or evaluated.
  [[ ${LAB_LENSES:-} != *$'\n'* && ${LAB_LENSES:-} != *$'\r'* ]] || { echo 'lenses must be space-separated' >&2; exit 2; }
  read -r -a lenses <<< "${LAB_LENSES:-}"
  (( ${#lenses[@]} >= 1 && ${#lenses[@]} <= 3 )) || { echo 'Choose one to three space-separated lenses' >&2; exit 2; }
  for lens in "${lenses[@]}"; do
    [[ $lens =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]*$ && $lens != README && -f lab/lenses/$lens.md ]] || {
      echo 'Each lens must name an existing lab/lenses/<lens>.md file' >&2; exit 2;
    }
  done
  host=$(mktemp -d "$RUNNER_TEMP/glm-lab.XXXXXXXX")
  ids=()
  cp lab/scrub.sh "$host/scrub.sh"
  # Make all worktrees before launching any process that can alter a checkout.
  for i in "${!lenses[@]}"; do
    ids+=("${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}-${i}-${lenses[$i]}")
    git worktree add --quiet --detach "$host/worktree-$i" HEAD
  done
  trap finalize EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  mkdir -p lab/results
  timeout --signal=TERM --kill-after=1m 1m omp usage > lab/results/quota.txt 2>&1 || {
    printf '\nQuota command failed or timed out; runs will still be attempted.\n' >> lab/results/quota.txt
  }
  bash "$host/scrub.sh" lab/results
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
