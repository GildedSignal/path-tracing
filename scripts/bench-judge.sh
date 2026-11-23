#!/usr/bin/env bash
# The bench as a judge: base and candidate built alike (from archived trees at paths of one
# length, functions and branch targets aligned to 64 bytes, so code layout stops moving the
# numbers), then run interleaved A B B A on one pinned core at high priority. Prints, per
# scenario, each side's median ns/tick with its interquartile range over every sample, and the
# candidate/base ratio of paired rounds (median, then least and greatest).
#   scripts/bench-judge.sh [base-ref] [rounds]     (defaults: c392524, 12; the candidate is the
#   working tree, uncommitted edits included)
set -euo pipefail
base=${1:-c392524}; rounds=${2:-12}
root=$(git rev-parse --show-toplevel); cd "$root"
cand=$(git stash create); cand=${cand:-HEAD}
out=D:/path-tracing-bench; mkdir -p "$out"
export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo
export RUSTFLAGS="-C llvm-args=-align-all-functions=6 -C llvm-args=-align-all-nofallthru-blocks=6"
build() {
  local sha; sha=$(git rev-parse --short=10 "$1")
  if [ ! -f "$out/bench-$sha.exe" ]; then
    rm -rf "$out/src-$sha"; mkdir -p "$out/src-$sha"
    git archive "$sha" engine | tar -x -C "$out/src-$sha"
    (cd "$out/src-$sha/engine" && CARGO_TARGET_DIR="$out/target" cargo build -q --release -p law --bin bench 2>>"$out/build.log")
    cp "$out/target/release/bench.exe" "$out/bench-$sha.exe"; rm -rf "$out/src-$sha"
  fi
  echo "$out/bench-$sha.exe"
}
a=$(build "$base"); b=$(build "$cand")
run() { cmd //c start '""' //b //wait //high //affinity 10 "$(cygpath -w "$1")" | tr -d '\r'; }
: > "$out/samples.txt"
for ((i = 0; i < rounds; i++)); do
  if ((i % 2)); then order="cand base"; else order="base cand"; fi
  for side in $order; do
    if [ "$side" = base ]; then exe=$a; else exe=$b; fi
    run "$exe" | sed "s/^/$i $side /" >> "$out/samples.txt"
  done
done
perl -ne '
  next unless /^(\d+) (\w+) ([\w-]+): .*samples=\[([^\]]*)\]/;
  my ($r, $side, $name, @s) = ($1, $2, $3, split /,\s*/, $4);
  push @{$all{$name}{$side}}, @s; my @o = sort { $a <=> $b } @s; $med{$name}{$r}{$side} = $o[@o / 2];
  END {
    sub pct { my ($p, @v) = @_; @v = sort { $a <=> $b } @v; $v[int($p * $#v + 0.5)] }
    for my $n (sort keys %all) {
      my @ratio = map { $med{$n}{$_}{cand} / $med{$n}{$_}{base} } grep { $med{$n}{$_}{cand} && $med{$n}{$_}{base} } keys %{$med{$n}};
      printf "%-12s base %6.1f [%6.1f..%6.1f]  cand %6.1f [%6.1f..%6.1f]  cand/base %.3f [%.3f..%.3f] over %d rounds\n", $n,
        pct(0.5, @{$all{$n}{base}}), pct(0.25, @{$all{$n}{base}}), pct(0.75, @{$all{$n}{base}}),
        pct(0.5, @{$all{$n}{cand}}), pct(0.25, @{$all{$n}{cand}}), pct(0.75, @{$all{$n}{cand}}),
        pct(0.5, @ratio), pct(0, @ratio), pct(1, @ratio), scalar @ratio;
    }
  }' "$out/samples.txt"
