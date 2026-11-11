#!/usr/bin/env bash
# One author and one independent checker, in a disposable, initially clean worktree.
set -euo pipefail

die() { echo "$*" >&2; exit 2; }
preamble() {
  sed -n '/^## The goal\r*$/,/^## From Emergent Space\r*$/{ /^## From Emergent Space\r*$/d; p; }' docs/PRINCIPLES.md
  printf '\n'; cat lab/glm.md; printf '\n'
}
restore_tree() {
  git restore --source="$base" --staged --worktree -- . ":(exclude)$out/" &&
    git clean -ffdx -q -e "$out/" -- .
}
capture_changes() {
  local patch=$1
  # A private index captures staged, unstaged, deleted, binary, ignored and new files.
  GIT_INDEX_FILE="$scratch/index" git read-tree "$base"
  GIT_INDEX_FILE="$scratch/index" git add --force --all -- . ":(exclude)$out/"
  GIT_INDEX_FILE="$scratch/index" git diff --cached --binary "$base" -- . ":(exclude)$out/" > "$out/$patch"
  restore_tree
}
phase() {
  local name=$1 minutes=$2 prompt
  prompt=$scratch/$name-prompt.md
  # On Windows (S43, Git Bash), omp.exe needs a Windows path; "@" stops MSYS from converting it.
  if command -v cygpath >/dev/null; then prompt=$(cygpath -w "$prompt"); fi
  timeout --signal=TERM --kill-after=1m "${minutes}m" \
    omp -p --model=zai/glm-5.3 --thinking=high --approval-mode=yolo --no-session \
    --max-time="${minutes}m" "@$prompt" > "$scratch/$name.log" 2>&1
}
finish() {
  local status=$?
  trap - EXIT
  if [[ ${active:-false} == true ]]; then
    capture_changes interrupted.patch || status=1
  fi
  # The author transcript stays out of the run folder until the checker has exited.
  cp -- "$scratch/"*-prompt.md "$scratch/"*.log "$out/" 2>/dev/null || true
  scrub_results "$out" || status=1
  rm -f -- "$scratch/"*-prompt.md "$scratch/"*.log "$scratch/index" "$scratch/index.lock"
  rmdir -- "$scratch" || status=1
  exit "$status"
}
main() {
  [[ ${GITHUB_ACTIONS:-} == true || ${GLM_LOCAL:-} == S43 ]] || die 'GLM runs only on GitHub Actions or through scripts/glm-local.ps1 (S43); local verification must use the offline stub test'
  [[ $# == 2 || $# == 3 ]] || die 'Usage: lab/run.sh <lens> <run-id> [author-minutes]'
  local lens=$1 id=$2 minutes=${3:-60} checker_minutes author_status=0 checker_status=0 grade claim first author_complete=true
  [[ $lens =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]*$ && $lens != README ]] || die 'Invalid lens name'
  [[ $id =~ ^[a-zA-Z0-9][a-zA-Z0-9._-]*$ ]] || die 'Invalid run id'
  [[ $minutes =~ ^[1-9][0-9]{0,2}$ ]] && (( minutes <= 180 )) || die 'author-minutes must be an integer from 1 to 180'
  [[ -n ${ZAI_API_KEY:-} ]] || die 'ZAI_API_KEY is missing'
  command -v omp >/dev/null || die 'omp is missing; run bash lab/setup.sh on the Actions runner'
  cd "$(git rev-parse --show-toplevel)"
  [[ -f lab/lenses/$lens.md ]] || die "Missing integrator lens: lab/lenses/$lens.md"
  [[ -z $(git status --porcelain --untracked-files=all --ignored) ]] || die 'Refusing a dirty worktree (including ignored files); use a fresh disposable worktree'
  out=lab/results/$id
  [[ ! -e $out && ! -L $out ]] || die 'Run folder already exists'
  base=$(git rev-parse HEAD)
  scratch=$(mktemp -d)
  mkdir -p "$out"
  # Load trusted cleanup code before either agent can edit the tree.
  source lab/scrub.sh
  trap finish EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  printf '%s\n' "$base" > "$out/base"
  checker_minutes=$(( (minutes + 1) / 2 ))
  {
    preamble
    cat "lab/lenses/$lens.md"
    cat <<PROMPT

## This run
Run id: $id. Lens: $lens. Author budget: $minutes minutes.
Repository: $PWD
Read lab/wiki/index.md and lab/wiki/$lens.md first. Do not re-ask questions graded reproduced or falsified; build on the leads.
Your only writable folder: $out/
Write $out/dossier.md, starting with exactly "Claim: " and a one-sentence claim.
Include the evidence, exact commands, falsifier, limits, and whether work.patch is needed.
Even a nothing-found result must explain what you checked, briefly.
Write $out/reproduce.sh to regenerate the evidence from shipped inputs; keep all other work in $out/.
Never change other paths, git metadata, commit, or push. No production code, docs, gates or fixtures.
Any outside changes will be captured as $out/work.patch and reverted before the checker starts.
The checker must run reproduce.sh itself; give it runnable commands, not your reasoning transcript.
PROMPT
  } > "$scratch/author-prompt.md"
  active=true
  phase author "$minutes" || author_status=$?
  capture_changes work.patch
  active=false
  printf 'author exit: %s\n' "$author_status" > "$out/phases.txt"
  if [[ ! -f $out/dossier.md || -L $out/dossier.md ]]; then
    author_complete=false
    rm -f -- "$out/dossier.md"
    printf 'Claim: The author produced no dossier.\n' > "$out/dossier.md"
  fi
  # Construct the checker prompt only after restoration, with no author session log.
  {
    preamble
    cat lab/checker.md
    printf '\n## Dossier\n'
    cat "$out/dossier.md"
    cat <<PROMPT

## This check
Run id: $id. Repository: $PWD. Budget: $checker_minutes minutes.
Read lab/wiki/index.md and lab/wiki/$lens.md first. Do not re-ask questions graded reproduced or falsified; build on the leads.
Run folder: $out/. Reproduction script: $out/reproduce.sh. Patch: $out/work.patch.
Author exit status: $author_status (nonzero means incomplete; do not hide it).
Run the reproduction yourself and try independent adversarial cases. Apply work.patch only if the dossier requires it.
Write $out/check.md. Keep your evidence in $out/. Never commit or push.
Do not read an author transcript or author session; only the dossier, scripts, data and patch are evidence.
PROMPT
  } > "$scratch/checker-prompt.md"
  active=true
  phase checker "$checker_minutes" || checker_status=$?
  capture_changes checker-work.patch
  active=false
  printf 'checker exit: %s\n' "$checker_status" >> "$out/phases.txt"
  # The first grade line and the first claim line, wherever they sit: the wiki reads them the same way.
  first=$(grep -m1 '^Grade: ' "$out/check.md" 2>/dev/null | tr -d '\r' || true)
  case $first in
    'Grade: reproduced'|'Grade: lead'|'Grade: nothing-found') grade=${first#Grade: } ;;
    *) grade=lead ;;
  esac
  claim=$(grep -m1 '^Claim: ' "$out/dossier.md" | sed 's/^Claim: //' | tr -d '\r' || true)
  if [[ $author_complete != true || -z $claim || ! -f $out/reproduce.sh || -L $out/reproduce.sh || ! -f $out/check.md || -L $out/check.md || $author_status != 0 || $checker_status != 0 || $first != "Grade: $grade" ]]; then
    [[ ! -e $out/check.md ]] || mv -- "$out/check.md" "$out/check-incomplete.md"
    grade=lead
    printf 'Grade: lead\nHarness: missing or invalid dossier, reproduce.sh or grade, or a phase exited nonzero. See phases.txt and check-incomplete.md if present.\n' > "$out/check.md"
  fi
  claim=${claim:-The author produced no one-sentence claim.}
  printf '%s | %s | %s | %s\n' "$id" "$lens" "$grade" "$claim" > "$out/DONE"
  [[ $author_status == 0 && $checker_status == 0 ]]
}
main "$@"
