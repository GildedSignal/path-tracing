# Job 3.2d-redteam: break this spec before it freezes

You are red-teaming a **specification**, not code. It has not been built yet, and that is the point: once a builder starts, a flaw here costs hours and a judge's FAIL. Its predecessor was judged FAIL and its own first draft was sent back for a rewrite, so this is the third attempt and the bar is high.

**The spec:** `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-sets\prompt.md`. Read it whole, twice.

**What it is for.** A planner for a Minecraft pathfinding bot. The project's one currency is **T**, server ticks from the user's click until the bot arrives with the world restored, thinking time included. The law is a Rust port of Mojang's 26.2 code and is the oracle; a plan is certified by replaying it tick by tick from the exact start state. The gate this spec feeds is one bot racing 18 hand-made parkour tracks.

**Read, so your attack is grounded and not generic:**
- `C:\Users\PASCAL\Desktop\path tracing\AGENTS.md` — especially "No patches" and "No new search code".
- `C:\Users\PASCAL\Desktop\path tracing\docs\PRINCIPLES.md`, the goal and philosophy sections.
- `C:\Users\PASCAL\Desktop\path tracing\jobs\planner-premises\last.md` — the audited constraint list the spec was written from, with 19 hard constraints, the settled measurements, the open questions and the contradictions between sources. **Check the spec against this list**: a constraint the audit raised and the spec silently dropped is your highest-value finding.
- `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2b-judge\last.md` — the FAIL whose four fixes the spec's step 0 claims to carry.
- `C:\Users\PASCAL\Desktop\path tracing\docs\learning.md` section 1 and `docs\vocabulary.md` sections 3, 7 and 12.
- `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2f-b-joints\last.md` and `jobs\3.2e-probes\last.md` — what was measured about the stalled joints, and what was explicitly **not determined**.

**Attack it on these axes, hardest first:**

1. **A way to pass while being wrong.** The spec freezes a kill table of 9 tracks, 4 expected to move. Find every way a builder could satisfy this spec, report movement on those 4, and still have built something that does not generalise — or that moves them for a reason the spec did not intend. This is the most valuable thing you can find.
2. **A dropped constraint.** Nineteen hard constraints were audited. Which does the spec fail to impose, impose ambiguously, or contradict? Cite the audit's number and the spec's line.
3. **An assumed-away open question.** The spec lists four things it must not assume. Does its own text assume one of them anyway, somewhere else?
4. **An invitation to patch.** Where would a competent builder, trying to satisfy this spec, reach for a tolerance, an epsilon, a retry, a clamp or a special case? Name the place in the spec that pushes them there. Note the spec forbids clamping the `acos` ratio at `movement.rs:802-803` while requiring corner checks to stay sound despite it — check whether that pairing is actually achievable, or whether it quietly demands the impossible.
5. **Underspecification.** A builder working unattended for hours needs nothing left open. Name every place where two readings are possible and would lead to different code.
6. **Overspecification.** The opposite failure, and it matters here: where does the spec dictate a mechanism where it should state a requirement, foreclosing a better representation the builder might have found? This project's rule is that a wrong representation must be re-derived, never patched, so a spec that over-constrains is a real cost.
7. **The measurement.** Would the spec's own reported numbers actually distinguish success from failure? Is the baseline comparison fair? Could the thinking-time accounting be gamed while technically complying?

**Rules.** Do not rewrite the spec and do not propose a planner design of your own — your product is a list of defects with the fix each one needs, and the integrator decides. Do not soften a finding because the spec explains itself well. Do not pad: a generic observation that would apply to any spec is noise and counts against your report. If you believe the spec is sound on an axis, say so in one line and move on — that is a real answer.

**Return**, at most 50 lines: the defects ranked by what they cost if built, each with the spec's line, the axis above it falls under, and the smallest change that fixes it; then one line per axis you found clean; then a single verdict word — `FREEZE` (build it as written), `AMEND` (fix the named items first, no re-review needed), or `REWRITE` (the spec's shape is wrong).
