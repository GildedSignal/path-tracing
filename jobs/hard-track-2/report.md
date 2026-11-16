# Hard track 2: whole-box certificate and offline race witness

The rebuilt course passes the whole-box certificate. Its witness has **race T 939 ticks** (943 from settled A), full health, no edits, and no items used. This is an existence certificate, not an optimality claim. No game, server, client or RCON was launched. The integrator still judges and places the build.

A remains (-107, -60, -107), B remains (-84, -25, -88), and `hardtrack.json` is unchanged. The initial ladder, northern blue-ice head-hitter and gap, slime rebound, final suspended ladder and all three forks remain.

## Old course: the counterexample and every hole

The old graph has 3013 supports and 3548489 edges. It retains the recorded route: southern lane end → alignment-wall top → final ladder. The assertions also connect A to that lane without the intended northern ice or main rebound, and connect the final ladder to B. The old local pocket checks still pass; they are retained explicitly as old-build controls, not as the rebuilt course's proof.

[old-graph.md](old-graph.md) gives witnesses and every missing-element subgraph. [old-graph.tsv](old-graph.tsv) lists every old node and successor using inclusive ID runs. [old-holes.tsv](old-holes.tsv) lists each complete A/B core after deleting an intended element. Their union describes **every skipping walk**, including arbitrarily many cycles, without pretending that a finite list of representative routes is exhaustive. These are conservative graph possibilities; only Julien's recording and the law-replayed sentences are execution evidence.

## What the certificate proves

The revised graph has **5300 supports and 4787613 edges**. It inventories all exposed full-block top faces, every climbable cell and climbable top face, and all slime blocks; even outside roof faces and inaccessible candidates are retained. Completely covered full faces cannot support a body. New partial shapes, lift hooks or friction regimes fail closed rather than disappearing from the inventory.

Every absent edge has a bound: vertical jump/rebound height, horizontal speed times the last possible airborne tick, or disconnected free space in the flight's altitude slab. Horizontal rectangles include the body's overhang. Endpoint rectangle arrangements include every free region and exact-width boundary; they are not center samples used to prune. The shortest law pose is used to over-approximate body clearance.

The ordinary apex is 1.252203340253724 blocks. The maximum horizontal movement envelope is **1.533322324673778 blocks/tick**, derived from law coefficients and the blue-ice friction recurrence. A ground jump must be followed by an airborne tick, even on a ceiling hit; the bound allows an impulse every other tick, faster than the actual head-hitter. The sine-table rotation enclosure is 1.024286818910270 for **every f32 yaw**, including large-angle index rounding and saturation. Arithmetic bounds round outward; no edge tolerance is used.

Slime impact energy is bounded using every ordinary launch in the same free-space component, including unreachable internal launches. The uncollided fall recurrence plus a full gravity tick bounds partial impact correction. Climbing uses the whole cell height. Between vertical-impulse/support events, the feet cannot descend below both endpoints and then regain height; a point body disconnected inside that altitude slab is therefore a sound geometric exclusion. A free outside ring retains routes that might leave and return. The final roof closes the box after all removals.

The step-up premise is explicit: every collision Y plane is asserted integral, and context-dependent collision shapes are refused. The law measures a step from the grounded or downward-collision base. These bases remain integral from settled A; the smallest positive candidate is one block, exceeding the law's 0.600000023841858-block step height. Thus a falling transfer cannot regain height through an omitted step-up in this fixture. Fractional-height terrain would require a different envelope.

The graph's ordered-state traversal has no A-to-B path missing the initial ladder → northern ice → main slime → final ladder sequence. Committed decoy regions contain 100, 12 and 92 supports, with **no edge out**. Commitment is the drop below an entry sill; returning on an approach before dropping is allowed. The final false ladder's upper cavity is also enclosed, so its unreachable high supports cannot supply a graph escape. The alignment-column regression is false on the rebuilt graph.

## Changed geometry and the separating edges

| Change | Why it closes edges |
|---|---|
| Full roof, initial shaft casing, raised divider and alignment-wall tops | Removes low exits from the initial climb and standable wall-top stepping stones. Interior and exterior roof components are separated. The final roof fill runs after every carving operation. |
| Southern false accelerator lowered below its entry platform, with a closed west sill and side walls | Its ground, pocket and gap floor form a committed region. Ordinary jumping cannot regain the entry or the northern landing. Its head-hitter tunnel remains visible from the fork's drop. |
| Main well moved to the southeast, reached by an east-boundary high approach | The initial deck cannot reach that pad in one unsupported flight, even with the global ice-speed envelope. The intended northern ice is required before the landing or approach is reachable. The northern false slime well remains a closed decoy. |
| Rebound divider extends below the exit support; low corridor returns north then west | A falling body cannot pass underneath and then regain the exit height. A rising rebound can. This excludes direct falling transfers from all high supports, not just the old lane endpoint. |
| Solid glass cover and upper closure around the final false ladder | Keeps the false fork and its pocket, while separating the high false ladder/backing supports from the route. The lower pocket cannot jump back to its entry. |

[critical-exclusions.md](critical-exclusions.md) gives the exact separating support pairs, bounds and positive deficits for these changes. The rebuilt witness's rebound clears the exit by 0.379788496764789 blocks.

## Every excluded edge near the old shortcut

The audit neighborhood is x=-86…-82, z=-107…-87, at **every inventoried height**. [old-shortcut-exclusions.csv](old-shortcut-exclusions.csv) contains all 10043 excluded ordered pairs there before the rebuild; [new-shortcut-exclusions.csv](new-shortcut-exclusions.csv) contains all 42106 afterwards. Nothing is sampled or truncated. Each row names both support cells, the proof, its upper bound, the required value and the positive deficit. For topology, zero common free components fails the necessary condition of at least one; its unit deficit is categorical, not a distance tolerance.

The former alignment top is covered by solid blocks and ceases to be a support. Its replacement roof face and every surviving nearby support are in the revised audit. Thus a removed face is not silently kept as an edge endpoint. Across the entire revised graph, minimum exclusion deficits (height in blocks, horizontal distance in blocks, common-component count) are [0.024629606984539976, 0.00013208007280951506, 1.0]. [new-graph.md](new-graph.md) records the invariant results and decoy counts.

## Witness, timing and replay

The [input sentence](../../lab/tracks/hardtrack.witness.txt) reaches B at its last tick. A second replay compares every state bit; the frozen complete-state digest is `b9fce6d6b6563d41`. The sentence is also certified through the existing tiny-program planner interface with the same endpoint and race clock. [witness.md](witness.md) records event states, health and margin.

[timing.md](timing.md) shifts each named boundary by ±1 tick with every later boundary fixed. Successful perturbations: n=18, min=939, median=943, p90=943, p99=943, max=943 ticks; failed perturbations: 3. Duplicate zero shifts are intentional positive controls. No mean is used. These are local perturbations, not a random-run distribution or a maximal-window claim. The rebound turn is more forgiving in the rebuilt geometry; the early ice and ladder takeoffs still fail. [controls.md](controls.md) removes the initial ladder, blue ice, low ceiling, main slime or final ladder separately; each breaks the frozen sentence.

Use the [ordered vanilla build](../../lab/tracks/hardtrack.build.txt). Its first command clears the specified box, preserving the grass floor. Query/save `minecraft:max_block_modifications`, temporarily set it to 106641 for construction, then restore the saved value. The largest later fill has 2401 blocks. The list applies the base layout followed by the certified revisions, ending with the complete roof; execute it in order.

From the worktree root:

```powershell
powershell -NoProfile -File jobs/hard-track-2/check.ps1
```

Or from `engine/`, with `CARGO_TARGET_DIR=D:\path-tracing-target\hard-track-2` and the staged D: Rust toolchain:

```powershell
cargo test --offline -p plan --test hard_track
cargo test --offline --workspace
```

Both required gates pass: hard-track **5 passed, 1 ignored**; workspace **191 passed, 19 ignored**. The planner diagnostic stays ignored. [gates.json](gates.json) records exits and wall times; [inputs.sha256](inputs.sha256) pins the law, registry, fixtures and delivered inputs. `check.ps1` regenerates every reported measurement and refuses to publish if inputs change during the gates. No global fastest-time claim or live audit is made; the earlier human shortcut time belongs to the old build.
