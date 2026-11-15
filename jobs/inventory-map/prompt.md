# Inventory: what the docs adopt against what the code does, and Julien's words that were lost (S132, S133)

Julien: "how many of those important things have we not built yet? This is an extremely holistic problem we can't just omit things like that" (S132), and "let hydrogen map and also what is said ad verbatim that might have been lost" (S133).

You read only. You run nothing and change nothing except your report. Read every file by its absolute path as listed below.

## Part A: the mechanism map

**Read the docs** (all under `C:\Users\PASCAL\Desktop\path tracing\docs\`):
- `engine.md`
- `representation.md`
- `scales.md`
- `ladder.md`
- `learning.md`
- `footprints.md`
- `server.md`
- `vocabulary.md`
- `PRINCIPLES.md`: the ES principles list, and every register row marked **Adopted** or **Followed**.

From these, list every mechanism, invariant, guarantee, data structure and rule they say the system has or will have. Examples: certificate expiry, slack, local repair, portal tables with widths, the region interface, light cones for players, the medium, GPU min-plus, bitboard cones, threads, executor re-anchoring, anytime refinement, the always-on invariants in `ladder.md`, store the generator, error widths.

**Then read the code** (all under `C:\Users\PASCAL\Desktop\path tracing\`):
- `engine\law\src\*.rs`
- `engine\plan\src\*.rs` and `engine\plan\src\bin\*.rs`
- `engine\words\src\*.rs`
- `engine\world\src\*.rs`
- `bot\src\pathtracing\*.java` and `bot\src\pathtracing\planner\*.java`
- `bot\src\pathtracing\mixin\*.java`
- `scripts\common.ps1`

Also read the plain-path branch's new file `D:\path-tracing-wt\plain\engine\plan\src\plain.rs`.

**For each mechanism give:**
- **Status:** BUILT, PARTIAL or DOC ONLY.
- **Evidence:** the doc's file and line, and the code's file and line. For DOC ONLY, name the files you checked that should have held it.
- **Consumer:** the rung or word that needs it, from the docs.
- **Duplicates:** where two mechanisms in the docs, or two pieces of the code, do the same work (for example two searches, two representations of the same reads, two clocks). Say whether they could be one.

## Part B: Julien's words that were lost

**Read his recorded turns** (all under `C:\Users\PASCAL\Desktop\path tracing\docs\reference\`):
- `julien-2026-11-04.md`
- `julien-2026-11-09.md`
- `julien-2026-11-10.md`
- `research-2026-11-05.md`
- `leads.md`

Do **not** read `private.md`.

For every distinct request, idea or rule he states, check four things:
- whether the register in `PRINCIPLES.md` cites it;
- what status the register gives it;
- whether a doc or `HANDOFF.md` (`C:\Users\PASCAL\Desktop\path tracing\docs\HANDOFF.md`) carries it as truth or as a tracked item;
- whether the code does it.

**List these:**
1. His words that no register row cites.
2. Rows marked Adopted whose mechanism Part A found DOC ONLY.
3. Rows still **Open** from before 14 November.
4. Requests he made twice or more. AGENTS.md says these become a tracked item in HANDOFF with an owner, so say whether HANDOFF has one.

Quote his words verbatim, with their S-number.

## Output

Write `C:\Users\PASCAL\Desktop\path tracing\jobs\inventory-map\report.md`, appending as you go. Its parts:
- **A summary first:** counts of BUILT, PARTIAL and DOC ONLY, and the ten DOC ONLY items that block the most.
- **Part A** as a table.
- **The duplicates.**
- **Part B's four lists.**

Cite every claim by file and line. When you cannot find something, write "not found in" with the files you checked; never guess.
