The measurement is **incomplete**: it identifies a run-up mismatch and a distinction between anchors and actual positions, but it does not establish the full cause of the stalls. The records are in [joints](D:/path-tracing-evidence/probes/joints/inputs.txt), including the [flight traces and collision boxes](D:/path-tracing-evidence/probes/joints/flight.txt) and [instrumentation patch](D:/path-tracing-evidence/probes/joints/probe.patch).

**fences** — anchor `[21, -61, 43]` → `[22, -61, 42]`; walk `FLSjnsn`, aim 47, three ticks, one run-up contact.

| Starting state | Replay result |
|---|---|
| Centre, at rest | Did not arrive. Its path differs from the stored path on tick 1; no horizontal collision was reported then. |
| 20 expansions spread across the run | None arrived after three ticks. Most sampled states had inherited the anchor while standing or moving elsewhere. |

The stored hop begins at a pre-accelerated contact, so the centre-at-rest replay does not reproduce it; this does **not** establish why the forward search stalled or whether a valid run-up could realize the hop.

**fencemaze** — anchor `[-7, -61, -12]` → `[-8, -61, -13]`; walk `FLSjnsn`, aim 31, three ticks, one run-up contact.

| Starting state | Replay result |
|---|---|
| Centre, at rest | Did not arrive. Its path differs from the stored path on tick 1; no horizontal collision was reported then. |
| 20 expansions spread across the run | One arrived after three ticks; the other 19 did not. Most had inherited the anchor while elsewhere. |

The same pre-accelerated-contact mismatch appears here, but one sampled state can realize the claimed hop; the measurement does **not** show why the search failed to proceed from that replay, because I did not analyze the next joint.

**redfakemaze**

| Starting state | Replay result |
|---|---|
| Centre, at rest | No outgoing anchored hop was available to replay. |
| Forward expansions | All 13,549 had no anchor, so there was no anchored joint from which to select the requested samples. |

This run did not stall at a claimed anchored joint: the field had no chain from A and the search never acquired an anchor. It does **not** explain why the unanchored search failed to find a route.

The traces retain each sampled position, velocity and per-tick state, but I did not compute the first departure and collision face for every sample or perform the required next-joint analysis. The worktree retains only the two pre-existing walked-tube check removals. No live run, commit, push, or merge was performed.