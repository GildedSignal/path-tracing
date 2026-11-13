1. **All 42 replays differ from the stored centre-based path on tick 1, and all have a collision on that tick:** some meet only the floor, while the other sampled states also meet a horizontal face; the coincidence does not establish that collision caused the departure.

2. **A valid run-up to the stored pre-accelerated contact is not determined by this measurement.** A replayed witness from an actual search state would establish a yes; a complete or otherwise proven reachability analysis would be needed for a no.

3. **Yes:** the one arriving `fencemaze` replay continues from `[-8, -61, -13]` to the next claimed set, `[-8, -61, -14]`, in two ticks; this continuation was replayed, not observed as a search expansion.

Every entry below departs on **tick 1**. The signs indicate the body’s attempted motion toward the contacted face: `y−` is floor contact, `x+` a face opposing positive x, and `z−` a face opposing negative z. “Rest” is the centre-at-rest replay; expansion numbers identify the original saved samples.

| `fences` sample | Contact | `fencemaze` sample | Contact |
|---|---|---|---|
| rest | y− | rest | y− |
| 4 | y− | 2 | y− |
| 779 | x+, y− | 578 | z− |
| 1555 | x+ | 1155 | z− |
| 2331 | x+ | 1731 | z− |
| 3106 | x+, y− | 2308 | z− |
| 3882 | x+ | 2885 | z− |
| 4658 | x+ | 3461 | z− |
| 5434 | x+, y− | 4038 | y−, z− |
| 6209 | x+, y− | 4615 | y−, z− |
| 6985 | x+ | 5191 | y−, z− |
| 7761 | x+ | 5768 | z− |
| 8537 | x+, y− | 6344 | z− |
| 9312 | x+ | 6921 | z− |
| 10088 | x+ | 7498 | z− |
| 10864 | x+, y− | 8074 | z− |
| 11640 | x+ | 8651 | z− |
| 12415 | x+ | 9228 | y−, z− |
| 13191 | x+ | 9804 | y−, z− |
| 13967 | x+, y− | 10381 | y−, z− |
| 14743 | x+ | 10958 | z− |

Many samples inherited an anchor while physically elsewhere, and the stored hop begins at a pre-accelerated contact. The table therefore does not by itself identify the cause of the stalls. For `redfakemaze`, the earlier run recorded **0 anchored and 13,549 unanchored expansions**; it was not rerun.

The [records and replay command](D:/path-tracing-evidence/probes/joints-b/inputs.txt), [full traces](D:/path-tracing-evidence/probes/joints-b/flight.txt), and [per-sample table](D:/path-tracing-evidence/probes/joints-b/table.csv) are saved. All replayed positions matched the original 42 traces; `cargo test --offline` passed on the unchanged checkout, which has no remaining diff.