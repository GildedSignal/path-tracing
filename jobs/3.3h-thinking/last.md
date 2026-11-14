The job is blocked by a measured contradiction in deliverable 2:

> “if the states never rejoin, it replays to the end”

versus its gate:

> “the law’s ticks spent in `slide` grow linearly with length”

None of the first pass’s yaw-slide candidates rejoined bit for bit; all reached the goal. Their mandatory suffix replays alone cost:

| Length | Law ticks |
|---:|---:|
| 29 | 2,400 |
| 100 | 26,568 |
| 300 | 237,168 |
| 800 | 1,666,680 |

That work grows quadratically. Prefix reuse cannot remove it under the required fallback rule.

The pay-rule change remains implemented, but its performance gate still fails. Local replay and `--serve` remain unfinished under the required deliverable order.

`cargo test --offline` passed. The footprint gate caught **12/12 mutants**. Net change: **+142 lines**, including the reproducible witness and report. Details and the before/after measurements are in [last.md](/D:/path-tracing-wt/thinking/jobs/3.3h-thinking/last.md).

STATUS: BLOCKED: The mandatory full replay for nonrejoining trajectories contradicts the linear slide-work gate on the supplied straight tracks.