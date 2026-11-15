# Julien's shortcut (S149–S152): T 428 against the witness's 579

Recording: `julien-428.jsonl` (copied from the ignored `recordings/track-hardtrack-Brain_Forge-1.jsonl`; header `sprintWindow` 7, so the law's replay refuses it at `engine/law/tests/replay.rs:366` until double-tap sprint is ported). Runs: 469, 434, 428 in `lab/tracks/runs/hardtrack.jsonl`.

His route, from the per-tick positions:
1. t 0–260: the initial ladder (x −105.49, y −58.6 → −28), as intended.
2. t 264–312: the **southern** blue-ice lane (z ≈ −102, fork 1's decoy), sprinting east to x −85.6.
3. t 324–336: off the lane's east end, before the decoy pocket, falling south onto the **top of the west alignment wall** (`/fill -85 -37 -99 -85 -32 -96`, top face y −32; feet −31.00 at z −95.66).
4. t 348–360: a jump north from the wall top onto the final ladder (`-84 -37 -92 -83 -25 -92`), caught at y −30, far above its bottom.
5. t 360–428: the ladder to B.

It skips the long jump, the chute, the slime well and the exit deck.

**What was false:** `report.md:32` ("the only combination of committed arms that completes is north accelerator, south rebound well, west suspended ladder"). It was argued, never checked by a machine. `reach_and_decoy_certificates` checks the inside of each pocket, not the supports reachable *before* commitment. The alignment wall's top is a support reachable from the decoy lane's end, and it was in no certificate.

**What a true certificate needs:** reach between every top face in the box under the law's jump and fall envelopes, not prose about the intended route.
