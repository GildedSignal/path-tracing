# Audit job: do we respect ES, or do we run the world on ticks?

You read and judge; you edit nothing but your report. Every finding cites file and line in our code and the ES principle it breaks (by number, from ES's own `docs/PRINCIPLES.md`).

## Julien's rule (S118, verbatim)

> we will need a full pass on if we respect the ES principles we don't run on ticks for the world model, ticks are just an execution layer that's premeditated and only breaks when the certificate does we're not baby sitting here. otherwise we'll do some cache dance for no reason and not using the generative functions and it's just worse in so many aspects, von neumann bottleneck is no joke.

And the one before it (S117): re-testing tick by tick "should never happen anyway since we're doing things ES style".

## Read first

- **ES itself**, at `D:\path-tracing-deps\emergent-space-engine\docs\`: `PRINCIPLES.md`, `representation.md`, `law.md`, `fermi.md`, `method.md` (skip `HISTORY.md` except to look up a principle's origin). It's Julien's engine: worldtubes, certificates ("compute only surprises"), generators ("store the generator, not the generated", ES 6), light cones, min-plus causality, the allocator.
- **How we carried it over:** `C:\Users\PASCAL\Desktop\path tracing\docs\PRINCIPLES.md` ("From Emergent Space"), `docs/representation.md` (sections 1–5, especially "Events: kinetic data structures, time advances when a certificate breaks"), `docs/engine.md` ("How it computes: a causal chain", "Local repair", "The contract", "The machine"), `docs/scales.md` 4.7.

## The code to audit

- **Main** (`C:\Users\PASCAL\Desktop\path tracing\engine\`): `law` (the tick, `pattern.rs`, footprints), `words` (`class.rs`, `segment.rs`, `program.rs`, `table.rs`), `world` (`snapshot.rs`), `probes`.
- **The planner and the first-flight stack**, on `D:\path-tracing-wt\play\engine\` (14 commits ahead of main): `plan/src/` (`search.rs`, `arith.rs`, `reach.rs`, `polish.rs`, `flight.rs`) and the play binaries (`plan-track`, `deviation`), plus `scripts/play-track.ps1` there.
- One violation is already known, as a calibration: `plan/src/arith.rs:32-55` (`free_tick`) runs the law one tick at a time and scans every cell of the tick's swept box, for every arc the search tries, though the arc certificate (`lab/lenses/arc-certificate.md`, ~700 ns per arc) answers the same arc once.

## What counts as a violation

The tick is legitimate in exactly three places: **the law itself** (bit-exact, it is nature's copy); **verification** (gates, L0, the final replay of a chosen sentence, the deviation stream against the server); and **compiling a word once** (the law integrated from one event to the next, then held as a generator). Everything else that steps ticks, or re-reads cells, to answer a question a generator or a certificate already answers is a finding:
- planning or search that simulates tick by tick where a word's closed form, cycle increments or certificate exists;
- execution that re-checks every tick instead of only where a certificate can break (it must be premeditated: inputs known in advance, a check only at a certificate's edge);
- caches that store generated output (states, trajectories, per-tick snapshots) where the generator and its arguments would do ("cache dance");
- memory traffic for nothing: whole `PlayerState` clones, per-tick hashing, per-tick allocations, cell scans on a hot path (the von Neumann bottleneck: count bytes moved per decision, against `docs/scales.md` 4.7's 24-byte hot state).

## For each finding

File and line; what the tick loop or the cache answers; the ES principle it breaks; the generator or certificate that should replace it, and whether that already exists in our code (name it) or must be built; a Fermi estimate of the gain (ticks or bytes per decision, before and after); *Killed if*: what would show the tick loop is in fact needed there (e.g. a certificate that breaks every tick, which `docs/representation.md` 7 sends to the dense lockstep path).

## The GPU, briefly

`docs/engine.md` "The machine" decides: Vulkan for integer min-plus over ticks and bitboard light cones; float work on the GPU advisory only; the GPU unqualified until a measured crossover (ES `fermi.md`). The machine is an AMD Radeon RX 7900 XTX beside a Ryzen 7 7800X3D (8 cores, 96 MB L3). Say whether any code that exists today has the data shape that would qualify, and what measurement decides the crossover. Do not design GPU code.

## Your report (under 150 lines)

1. A verdict in three lines: where we respect ES, where we don't, and how much it costs us.
2. The findings, ranked by what they cost the scoreboard (thinking time, T, scale to 10⁴ bots).
3. What we do right, with file and line, so nobody undoes it.
4. The GPU paragraph.
5. What you could not read or verify, by name.
