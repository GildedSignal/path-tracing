#!/usr/bin/env bash
# Print one GLM window's run commands, one per line (`start <lens> <min>` or `rung <file> <min>`): at most DIRECTED
# entries of lab/queue.txt with no published verdict (VOID and DISPUTED are retried, twice at most), then free lenses
# (each picks its own question) up to SLOTS. Julien, 6 October: "abuse GLM more, more diversity and usefulness".
SLOTS=${SLOTS:-4}; DIRECTED=${DIRECTED:-3}; SECONDS_UP=${SECONDS_UP:-1}   # five spent GLM's 5-hour window in about 4 h 20 (6 October): four fit it
cd "$(git rev-parse --show-toplevel)"
git fetch -q origin '+refs/heads/claude/lab-*:refs/remotes/origin/claude/lab-*' 2>/dev/null || true
state() {  # state <slug>: "done" if a published result is not VOID or DISPUTED, else how many were
  local v=0 b
  for b in $(git for-each-ref --format='%(refname:short)' "refs/remotes/origin/claude/lab-*-$1-????"); do
    git log -1 --format=%s "$b" | grep -qE "VOID|DISPUTED" && v=$((v + 1)) || { echo done; return; }
  done
  echo "$v"
}
n=0
while read -r kind arg min; do
  case $kind in rung) slug=rung-$(basename "$arg" .md) ;; start) slug=$arg ;; *) continue ;; esac
  s=$(state "$slug"); { [ "$s" = done ] || [ "$s" -ge 2 ]; } && continue
  echo "$kind $arg $min"; n=$((n + 1)); [ $n -eq "$DIRECTED" ] && break
done < lab/queue.txt
# Second authors: the newest silvers of the last four days that no run has followed yet.
m=0
for b in $(git for-each-ref --sort=-committerdate --format='%(refname:short)' 'refs/remotes/origin/claude/lab-*' | head -120); do
  [ $n -lt $SLOTS ] && [ $m -lt "$SECONDS_UP" ] || break
  r=${b#origin/claude/lab-}
  case $r in *second-*) continue ;; esac
  [ "$(git log -1 --format=%ct "$b")" -gt $(( $(date +%s) - 345600 )) ] || continue
  git log -1 --format=%s "$b" | grep -q "Grade: silver" || continue
  [ -n "$(git for-each-ref --format=x "refs/remotes/origin/claude/lab-*-second-$r-????")" ] && continue
  echo "start second-$r 90"; n=$((n + 1)); m=$((m + 1))
done
lenses=(dynamics system components scout quant spacex taleb library redteam inversion fermi)   # autoresearch and constraint retired 12 October: 0 gold in 71 runs (docs/archive/README.md)
k=$(( $(date -u +%s) / 3600 ))
[ $n -lt "$SLOTS" ] && for j in $(seq $n $((SLOTS - 1))); do echo "start ${lenses[$(( (k * SLOTS + j) % ${#lenses[@]} ))]} 60"; done
