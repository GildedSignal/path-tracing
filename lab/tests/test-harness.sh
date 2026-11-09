#!/usr/bin/env bash
# Offline only: synthetic git repositories and a fake omp. Never invokes the real agent.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
test_parent=$(cd "${TMPDIR:-/tmp}" && pwd -P)
test_root=$(mktemp -d "$test_parent/glm-harness.XXXXXXXX")
cleanup() {
  # Verify the absolute recursive-delete target, including when testing on Windows.
  case $test_root in "$test_parent"/glm-harness.*) rm -rf -- "$test_root" ;; *) exit 1 ;; esac
}
trap cleanup EXIT
export TEST_ROOT=$test_root
export REAL_TIMEOUT=$(command -v timeout)
export REAL_SHA256SUM=$(command -v sha256sum)
export ZAI_API_KEY='stub-only.[key]*value'
export STUB_MODE=normal
# Simulate the host only inside this test process, whose PATH points to the stub.
export GITHUB_ACTIONS=true
export PATH="$test_root/bin:$PATH"
mkdir -p "$test_root/bin" "$test_root/source/lab/lenses" "$test_root/source/docs"
cp "$repo/"lab/{run,actions,scrub,setup}.sh "$test_root/source/lab/"
cp "$repo/"lab/{glm,checker}.md "$test_root/source/lab/"
cp "$repo/docs/PRINCIPLES.md" "$test_root/source/docs/"
printf 'Synthetic test input, not a production lens.\n' > "$test_root/source/lab/lenses/stub.md"
printf 'lab/results/*\n!lab/results/README.md\n.cache/\n' > "$test_root/source/.gitignore"
printf 'original\n' > "$test_root/source/protected.txt"
printf 'delete me\n' > "$test_root/source/delete-me.txt"
printf '\0before\377' > "$test_root/source/binary.dat"
mkdir -p "$test_root/source/lab/results"
printf 'Fixture README\n' > "$test_root/source/lab/results/README.md"
# These commits exist only in the throwaway test repository, never in the project.
git -C "$test_root/source" init -q
git -C "$test_root/source" config core.autocrlf false
git -C "$test_root/source" add .
git -C "$test_root/source" -c user.name=stub -c user.email=stub@example.invalid commit -qm fixture
cat > "$test_root/bin/omp" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ $* == usage ]]; then
  echo quota >> "$TEST_ROOT/events"
  echo 'Synthetic quota: no API call'
  exit 0
fi
[[ $* == *'--model=zai/glm-5.3'* && $* == *'--thinking=high'* && $* == *'--approval-mode=yolo'* && $* == *'--no-session'* ]]
prompt=${!#}; prompt=${prompt#@}
[[ -f $prompt ]]
out=$(find lab/results -mindepth 1 -maxdepth 1 -type d | head -n 1)
id=${out##*/}
expected=$TEST_ROOT/preamble-$id-$$
sed -n '/^## The goal\r*$/,/^## From Emergent Space\r*$/{ /^## From Emergent Space\r*$/d; p; }' docs/PRINCIPLES.md > "$expected"
head -c "$(wc -c < "$expected")" "$prompt" | cmp - "$expected"
! grep -q '^## From Emergent Space' "$prompt"
if [[ $prompt == *author-prompt.md ]]; then
  [[ $* == *'--max-time=2m'* ]]
  echo "author $id ${CARGO_TARGET_DIR:-none}" >> "$TEST_ROOT/events"
  [[ -z ${CARGO_TARGET_DIR:-} ]] || {
    mkdir -p "$TEST_ROOT/barrier"
    touch "$TEST_ROOT/barrier/$id"
    for ((n=0; n<300; n++)); do
      [[ $(find "$TEST_ROOT/barrier" -type f | wc -l) == ${STUB_WORKERS:-3} ]] && break
      sleep 0.1
    done
    [[ $(find "$TEST_ROOT/barrier" -type f | wc -l) == ${STUB_WORKERS:-3} ]]
  }
  printf 'staged\n' > protected.txt
  git add protected.txt
  printf 'author experiment\n' > protected.txt
  rm delete-me.txt
  printf '\0after\376' > binary.dat
  mkdir -p .cache
  printf '\0ignored\377' > .cache/hidden.bin
  printf 'new evidence\n' > 'outside new.txt'
  printf 'Claim: Synthetic evidence reproduces.\nPatch needed: yes; apply work.patch before reproduce.sh.\n' > "$out/dossier.md"
  cat > "$out/reproduce.sh" <<'REPRO'
#!/usr/bin/env bash
set -euo pipefail
[[ $(cat protected.txt) == 'author experiment' ]]
[[ ! -e delete-me.txt && -f 'outside new.txt' && -f .cache/hidden.bin ]]
printf '\0after\376' | cmp - binary.dat
echo 'Synthetic evidence reproduced'
REPRO
  printf '%s\n' "$ZAI_API_KEY" > "$out/leaked key.txt"
  printf '\0%s\377' "$ZAI_API_KEY" > "$out/leaked-binary.bin"
  printf '%s\n' "$ZAI_API_KEY"  # author transcript must also be deleted
  head -c 5242881 /dev/zero > "$out/too-big.bin"
  head -c 5242880 /dev/zero > "$out/at-limit.bin"
  if [[ $STUB_MODE == author-fails ]]; then exit 7; fi
  if [[ $STUB_MODE == one-fails && $id == *-0-stub ]]; then exit 7; fi
  if [[ $STUB_MODE == author-times-out ]]; then sleep 10; fi
else
  [[ $prompt == *checker-prompt.md && $* == *'--max-time=1m'* ]]
  echo "checker $id" >> "$TEST_ROOT/events"
  [[ ! -e $out/author.log && ! -e $out/author-prompt.md ]]
  ! grep -q 'staged' protected.txt
  [[ $(cat protected.txt) == original && -f delete-me.txt && ! -e 'outside new.txt' && ! -e .cache ]]
  printf '\0before\377' | cmp - binary.dat
  git diff --quiet && git diff --cached --quiet
  git apply "$out/work.patch"
  bash "$out/reproduce.sh" > "$out/reproduced.log" 2>&1
  case $STUB_MODE in
    malformed) printf 'Explanation first\nGrade: reproduced\n' > "$out/check.md" ;;
    no-grade) : ;;
    nothing) printf 'Grade: nothing-found\nSynthetic empty inquiry.\n' > "$out/check.md" ;;
    *) printf 'Grade: reproduced\nRan reproduce.sh and checked the synthetic binary.\n' > "$out/check.md" ;;
  esac
  printf 'checker temporary experiment\n' > protected.txt
  [[ $STUB_MODE != checker-fails ]] || exit 8
  if [[ $STUB_MODE == checker-times-out ]]; then sleep 10; fi
fi
STUB
chmod +x "$test_root/bin/omp"
cat > "$test_root/bin/timeout" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ ( $STUB_MODE == author-times-out && $* == *author-prompt.md ) || ( $STUB_MODE == checker-times-out && $* == *checker-prompt.md ) ]]; then
  shift 3
  exec "$REAL_TIMEOUT" --signal=TERM --kill-after=1s 3s "$@"
fi
exec "$REAL_TIMEOUT" "$@"
STUB
chmod +x "$test_root/bin/timeout"

new_tree() {
  git -C "$test_root/source" worktree add -q --detach "$test_root/$1" HEAD
  cd "$test_root/$1"
}
assert_restored() {
  git diff --quiet && git diff --cached --quiet
  [[ $(cat protected.txt) == original && -f delete-me.txt && ! -e 'outside new.txt' && ! -e .cache ]]
  [[ -z $(git ls-files --others --exclude-standard) ]]
}
assert_scrubbed() {
  [[ ! -e $1/'leaked key.txt' && ! -e $1/leaked-binary.bin && ! -e $1/too-big.bin && ! -e $1/author.log ]]
  [[ -f $1/at-limit.bin ]]
  ! grep -arFq -- "$ZAI_API_KEY" "$1"
}
new_tree success
bash lab/run.sh stub success 2
assert_restored
assert_scrubbed lab/results/success
grep -qx 'success | stub | reproduced | Synthetic evidence reproduces.' lab/results/success/DONE
grep -q 'GIT binary patch' lab/results/success/work.patch
grep -q '.cache/hidden.bin' lab/results/success/work.patch
grep -qx 'Grade: reproduced' <(head -n 1 lab/results/success/check.md)
[[ $(wc -l < lab/results/success/DONE) == 1 ]]
printf 'PASS: both phases, prompt prefix, private transcript, all patch types, restore, DONE and scrub\n'

for mode in author-fails checker-fails author-times-out checker-times-out malformed no-grade nothing; do
  new_tree "$mode"
  export STUB_MODE=$mode
  status=0
  bash lab/run.sh stub "$mode" 2 || status=$?
  case $mode in author-fails|checker-fails|*-times-out) [[ $status != 0 ]] ;; *) [[ $status == 0 ]] ;; esac
  expected=lead
  [[ $mode != nothing ]] || expected=nothing-found
  grep -qx "Grade: $expected" <(head -n 1 "lab/results/$mode/check.md")
  grep -qx "$mode | stub | $expected | Synthetic evidence reproduces." "lab/results/$mode/DONE"
  assert_restored
  assert_scrubbed "lab/results/$mode"
done
printf 'PASS: nonzero/timed-out phases, invalid/missing checker grades, nothing-found\n'

new_tree validation
if GITHUB_ACTIONS=false bash lab/run.sh stub run 2 > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
grep -q 'only on GitHub Actions' "$TEST_ROOT/rejected.log"
for args in 'stub ../escape 2' '../stub run 2' 'missing run 2' 'stub run 0' 'stub run 181' 'stub run 1.5'; do
  if bash lab/run.sh $args > "$TEST_ROOT/rejected.log" 2>&1; then echo "Accepted invalid arguments: $args" >&2; exit 1; fi
done
if ZAI_API_KEY= bash lab/run.sh stub run 2 > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
printf 'dirty\n' >> protected.txt
if bash lab/run.sh stub run 2 > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
grep -q 'dirty worktree' "$TEST_ROOT/rejected.log"
printf 'PASS: path traversal, missing lens/key, minute bounds and dirty-tree refusal\n'

new_tree parallel
export STUB_MODE=normal LAB_LENSES='stub stub stub' LAB_MINUTES=2
export RUNNER_TEMP="$test_root/runner" GITHUB_RUN_ID=123 GITHUB_RUN_ATTEMPT=1
export GITHUB_OUTPUT="$test_root/output" GITHUB_STEP_SUMMARY="$test_root/summary"
mkdir -p "$RUNNER_TEMP"
: > "$TEST_ROOT/events"
bash lab/actions.sh
[[ $(head -n 1 "$TEST_ROOT/events") == quota ]]
[[ $(grep -c '^author ' "$TEST_ROOT/events") == 3 && $(grep -c '^checker ' "$TEST_ROOT/events") == 3 ]]
[[ $(awk '/^author /{print $3}' "$TEST_ROOT/events" | sort -u | wc -l) == 3 ]]
[[ $(grep -c ' | reproduced | ' "$GITHUB_STEP_SUMMARY") == 3 ]]
grep -qx 'sanitized=true' "$GITHUB_OUTPUT"
for id in 123-1-{0,1,2}-stub; do
  [[ -f lab/results/$id/DONE && -f lab/results/$id/host.log ]]
  assert_scrubbed "lab/results/$id"
done
printf 'PASS: quota precedes three parallel isolated worktrees/targets; collected artifact, summary and scrub\n'

# Validate host input before any quota or worktree operation.
for invalid in '' 'stub stub stub stub' '../stub' 'stub; echo injected'; do
  if LAB_LENSES="$invalid" bash lab/actions.sh > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
done
if ZAI_API_KEY= bash lab/actions.sh > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
grep -q 'Actions secret is missing' "$TEST_ROOT/rejected.log"
printf 'PASS: workflow host rejects missing key, excess lenses and shell input\n'

# One failure still waits for and collects both successful peers, then enables a scrubbed upload.
new_tree mixed
mv "$TEST_ROOT/barrier" "$TEST_ROOT/previous-barrier"
export STUB_MODE=one-fails GITHUB_RUN_ID=124
: > "$GITHUB_OUTPUT"
: > "$GITHUB_STEP_SUMMARY"
if bash lab/actions.sh; then echo 'Host hid a failed phase' >&2; exit 1; fi
[[ $(grep -c ' | reproduced | ' "$GITHUB_STEP_SUMMARY") == 2 && $(grep -c ' | lead | ' "$GITHUB_STEP_SUMMARY") == 1 ]]
grep -qx 'sanitized=true' "$GITHUB_OUTPUT"
for id in 124-1-{0,1,2}-stub; do
  [[ -f lab/results/$id/DONE ]]
  assert_scrubbed "lab/results/$id"
done
printf 'PASS: failed phase preserves completed peers, lead evidence, summary and sanitized upload eligibility\n'

# A local batch has twenty-four simultaneous isolated lanes, with the hosted cap preserved above.
new_tree local-twenty-four
mv "$TEST_ROOT/barrier" "$TEST_ROOT/three-worker-barrier"
export STUB_MODE=normal GITHUB_RUN_ID=125
: > "$TEST_ROOT/events"
: > "$GITHUB_OUTPUT"
: > "$GITHUB_STEP_SUMMARY"
trial_lenses=$(printf 'stub %.0s' {1..24})
GITHUB_ACTIONS=false GLM_LOCAL=S43 STUB_WORKERS=24 LAB_LENSES="$trial_lenses" bash lab/actions.sh
[[ $(grep -c '^author ' "$TEST_ROOT/events") == 24 && $(grep -c '^checker ' "$TEST_ROOT/events") == 24 ]]
[[ $(awk '/^author /{print $3}' "$TEST_ROOT/events" | sort -u | wc -l) == 24 ]]
[[ $(grep -c ' | reproduced | ' "$GITHUB_STEP_SUMMARY") == 24 ]]
grep -qx 'sanitized=true' "$GITHUB_OUTPUT"
for id in 125-1-{0..23}-stub; do
  [[ -f lab/results/$id/DONE && -f lab/results/$id/host.log ]]
  assert_scrubbed "lab/results/$id"
done
if GITHUB_ACTIONS=false GLM_LOCAL=S43 LAB_LENSES="$trial_lenses stub" bash lab/actions.sh > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
grep -q 'Choose 1 to 24 space-separated lenses' "$TEST_ROOT/rejected.log"
printf 'PASS: twenty-four local isolated lanes and their checker artifacts; twenty-five refused\n'

# Installer tests redirect only the destination in a copy, never changing the real HOME.
export TEST_INSTALL_HOME="$test_root/install" STUB_CHECKSUM=pass STUB_ARCH=x86_64
mkdir -p "$TEST_INSTALL_HOME/.local/bin"
sed 's/\$HOME/\$TEST_INSTALL_HOME/g' "$repo/lab/setup.sh" > "$TEST_ROOT/setup-under-test.sh"
cat > "$test_root/bin/uname" <<'STUB'
#!/usr/bin/env bash
case $1 in -s) echo Linux ;; -m) echo "$STUB_ARCH" ;; *) exit 2 ;; esac
STUB
cat > "$test_root/bin/curl" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ $* == *https://github.com/can1357/oh-my-pi/releases/download/v18.16.0/omp-linux-x64* ]]
while [[ $1 != -o ]]; do shift; done
printf 'synthetic executable\n' > "$2"
STUB
cat > "$test_root/bin/sha256sum" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ $STUB_CHECKSUM == pass ]]; then
  read -r hash file
  [[ $hash == 6498ed785bab00d2926e0387e0ec02362d6d45f1c3d0d1b57c1618b7f95c9264 && -f $file ]]
else
  exec "$REAL_SHA256SUM" "$@"
fi
STUB
chmod +x "$test_root/bin/"{uname,curl,sha256sum}
bash "$TEST_ROOT/setup-under-test.sh"
grep -qx 'synthetic executable' "$TEST_INSTALL_HOME/.local/bin/omp"
printf 'previous installation\n' > "$TEST_INSTALL_HOME/.local/bin/omp"
if STUB_CHECKSUM=fail bash "$TEST_ROOT/setup-under-test.sh" > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
grep -q 'SHA-256 mismatch' "$TEST_ROOT/rejected.log"
grep -qx 'previous installation' "$TEST_INSTALL_HOME/.local/bin/omp"
[[ -z $(find "$TEST_INSTALL_HOME/.local/bin" -name '.omp.*' -print) ]]
if STUB_ARCH=aarch64 bash "$TEST_ROOT/setup-under-test.sh" > "$TEST_ROOT/rejected.log" 2>&1; then exit 1; fi
printf 'PASS: installer version/hash checks, checksum refusal preserves old executable, wrong architecture refused (download mocked)\n'
