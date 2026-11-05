#!/usr/bin/env python3
"""Verdict for a directed rung: verdict.py <rung.md> <checker-metrics.json> [<author-metrics.json>]. Exit 0, or 2 on usage.
The rung has a ```configs block (<name> candidate|negative|positive) and a ```criteria block (<id> behaviour|type|cost <metric> <op> <threshold>).
Metrics files: {"<config>": {"<metric>": number}}. Order: VOID (numbers incomplete, a negative passes all, or a positive control fails one), DISPUTED (author and
checker differ in a pass/fail outcome), PASS (a candidate passes all), else KILL: it kills configurations, never the method."""
import json, math, operator, re, sys

OPS = {"<=": operator.le, "<": operator.lt, ">=": operator.ge, ">": operator.gt}

def parse(text):
    def rows(name):
        m = re.search(r"^```" + name + r"[^\n]*\n(.*?)^```", text, re.S | re.M)
        return [l.split() for l in (m.group(1).splitlines() if m else []) if l.strip() and not l.lstrip().startswith("#")]
    cfgs, crit = rows("configs"), rows("criteria")
    ok = (any(r[1:] == ["candidate"] for r in cfgs) and crit and all(len(r) == 2 and r[1] in ("candidate", "negative", "positive") for r in cfgs)
          and all(len(r) == 5 and r[1] in ("behaviour", "type", "cost") and r[3] in OPS for r in crit)
          and len({r[0] for r in cfgs}) == len(cfgs) and len({r[0] for r in crit}) == len(crit))
    if not ok:
        raise ValueError("needs a candidate, and well-formed unique configs and criteria lines")
    return [tuple(r) for r in cfgs], [(i, k, m, op, float(th)) for i, k, m, op, th in crit]  # float() may raise ValueError

def num(m, cfg, metric):  # a finite-or-infinite real number, else None (bool, string, null, NaN are not numbers)
    v = m.get(cfg, {}).get(metric)
    return v if isinstance(v, (int, float)) and not isinstance(v, bool) and not math.isnan(v) else None

def load(path):
    try:
        with open(path) as f:
            d = json.load(f)
        return d if isinstance(d, dict) and all(isinstance(v, dict) for v in d.values()) else None
    except (OSError, ValueError):
        return None

def judge(text, chk, auth=None):
    """The output lines; the first is 'Verdict: X; reason'. auth None means no author file was given."""
    try:
        cfgs, crit = parse(text)
    except ValueError as e:
        return [f"Verdict: VOID; the rung file is malformed ({e})"]
    if chk is None:
        return ["Verdict: VOID; the checker's metrics file is missing or unreadable"]
    mets = sorted({m for _, _, m, _, _ in crit})
    gaps = [f"{c}: no such configuration" for c, _ in cfgs if c not in chk]
    gaps += [f"{c}: {m} is missing or not a number" for c, _ in cfgs if c in chk for m in mets if num(chk, c, m) is None]
    if gaps:
        return [f"Verdict: VOID; the checker's metrics are incomplete ({len(gaps)} gaps)"] + gaps
    failed = {c: [i for i, _, m, op, th in crit if not OPS[op](chk[c][m], th)] for c, _ in cfgs}
    lines = [f"{c} {r} " + ("fail " + ",".join(failed[c]) if failed[c] else "pass") for c, r in cfgs]
    cand = [c for c, r in cfgs if r == "candidate"]
    stuck = [c for c, r in cfgs if r == "negative" and not failed[c]]
    unpassable = [c for c, r in cfgs if r == "positive" and failed[c]]
    if unpassable:
        return [f"Verdict: VOID; a positive control fails, so the test cannot be passed: {', '.join(unpassable)}"] + lines
    if stuck:
        return [f"Verdict: VOID; negative passes every criterion, so the test cannot fail: {', '.join(stuck)}"] + lines
    if auth is not None:
        diff = []
        for c, _ in cfgs:
            for i, _, m, op, th in crit:
                a, ok = num(auth, c, m), i not in failed[c]
                if a is None or OPS[op](a, th) != ok:
                    diff.append(f"{c} {i}: checker {'pass' if ok else 'fail'}, author " +
                                ("missing" if a is None else "pass" if OPS[op](a, th) else "fail"))
        if diff:
            return [f"Verdict: DISPUTED; author and checker differ on {len(diff)} outcome(s)"] + lines + diff
    won, dead = [c for c in cand if not failed[c]], [c for c in cand if failed[c]]
    if won:
        return [f"Verdict: PASS; {', '.join(won)} passed every criterion" + (f"; killed: {', '.join(dead)}" if dead else "")] + lines
    return [f"Verdict: KILL; no candidate passed every criterion; killed: {', '.join(dead)}"] + lines

def main(argv):
    if len(argv) not in (2, 3):
        print("usage: verdict.py <rung.md> <checker-metrics.json> [<author-metrics.json>]")
        return 2
    try:
        text = open(argv[0]).read()
    except OSError as e:
        print(f"cannot read the rung file: {e}")
        return 2
    print("\n".join(judge(text, load(argv[1]), (load(argv[2]) or {}) if len(argv) == 3 else None)))
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
