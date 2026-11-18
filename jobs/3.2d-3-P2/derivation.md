# P2: exact binary64 word maps and detector preimages

Consumer: U1's kernel, used by every zone (S165). This is a test-local arithmetic prototype, not an adapter or another planner. No engine source is changed.

## Law and scope

`PlayerState::bounding_box` (`engine/law/src/state.rs:196-197`) calls `Aabb::player` (`engine/law/src/shapes.rs:111-113`). That function owns the horizontal radius; `Pose::height` (`state.rs:27-32`) owns the pose height. P2 calls those functions. Eye height is not a read of `Goal::touched`; no eye dimension is introduced. `Goal::zone` and `Goal::touched` (`engine/plan/src/goal.rs:15-26`) own detector expansion, before/after hull, and strict intersection (`shapes.rs:203-204`). The integrator's correction replaces the spec's decimal radius shorthand with this law.

The witness kernel is P1's word 30 on the straight100 stone model, prepared with native table direction +x, not a rotated +z trajectory. P1's native +z direction and the native +x direction are separate bound operands. Preparation recovers the measured transient/cycle and asserts the mandated 263/12. `tick_with_collision` (`movement.rs:67-70`) calls the same movement implementation as `try_tick` (`movement.rs:56-57`), returning the actual before-addition moved operand. `movement_this_tick` stores wanted at `movement.rs:748`; position adds moved at :749. Preparation proves horizontal equality on this word before retaining the operands. Position differences are never operands.

The world is a Peaceful, loaded, uniform stone layer copied from straight100, covering every streamed query. Vertical phase, pose and the motor cycle are retained from native preparation. Resource numeric values remain outside the P1 motor projection; food/health guards are retained. This does not prove a resource map or any other bearing/medium. P4 owns resource recurrence. No game process runs.

## Number ownership and rounding

A finite binary64 is decoded into sign, integer significand and binary exponent, including both signed zeros and subnormals. The represented number is signed significand times 2^exp. Normal significands have the hidden leading bit; subnormals do not. Reconstruction writes the bits directly.

Within a signed normal binade let U=2^exp and p=kU with signed integer k. Subnormals use their own signed region at the same minimum spacing; zero, sign changes and normal/subnormal boundaries remain explicit. Integer carry is retained in full, not truncated to the parity state.

Decode each operand d exactly. Euclidean division gives d/U=N+r with 0<=r<1 even for negative d. Comparing twice the integer remainder with its power-of-two denominator determines below-half, above-half or exact tie. The increment is N, N+1, or N+((k+N) mod 2) respectively. This is nearest-even rounding on the lattice. A result strictly within the current region has the same representable neighbours and midpoint ownership as native addition. Boundary results are excluded from that claim and use the native binary64 addition once.

Only parity can affect an increment. Compose a fixed word for each of the two input parities, retaining every phase offset and full-word displacement. Following the parity transition reaches a repeated node after at most two states. The graph's prefix, cycle and cumulative displacement give any requested number of words by quotient/remainder and a bounded sum over those states. No loop enumerates the requested count.

For a periodic graph node i, phase j and graph repetition r, the intermediate coordinate is base_i + r*D + offset_ij. The legal strict region is lo<coordinate<hi. Solve its first violation by signed integer division, then take the earliest violating word across all nodes/phases. Jump every preceding word, use stored offsets for the safe prefix of the boundary word, and execute only the crossing addition. Repeat at the new region. An operand whose scaled integer exceeds the prototype's shift domain necessarily leaves the region immediately; its single boundary addition does not refuse or approximate a gate instance.

The qualified word has fixed period, finite operands and finite outputs, with significands/count products inside i128. General unbounded-period/overflow/nonfinite inputs are outside this test prototype's API contract. No claim about all binary64 programs is made.

## Exact stop inversion

Each event segment emits one affine family per reachable parity node/word phase. A family stores coordinate base+r*D, before/after phase offsets, tick origin, tick stride and count. Thus even a long segment has a bounded number of records, not future tick positions.

Vertical and stationary lateral eligibility comes from the hull of the two native phase boxes. Positive moved operands make the after box's right edge and before box's left edge the swept x extremes. Let g be a detector comparison edge, a the signed radius obtained by calling the law, and M the exact dyadic midpoint of g and nextUp(g). RN(kU+a)>g holds when kU+a>M if g has an even significand, and when kU+a>=M if g has an odd significand. Thus the first integer is floor((M-a)/U)+1 or ceil((M-a)/U), respectively. Before.min>=zone.max is expressed as Before.min>nextDown(zone.max). Every midpoint, subtraction and division is integer/dyadic, preserving the width addition's own rounding and the strict slab edge. The integer interval is clipped to the region before inversion through a family. Integer-domain bisection with native additions is retained only as an independent qualification oracle, never in the timed inverse.

Goal cells are i32, their actual slab edges are nonzero finite binary64s, and the body's native radius has bounded magnitude. Their midpoint subtraction fits i128. Scaling far outside a region is saturated beyond every possible mantissa/phase offset; it cannot select a count in that region. The qualification compares these integer thresholds to native-addition bisection over both signs, tiny/ordinary/huge exponents and extreme Goal cells.

Pull that integer interval through base+r*D by exact ceil/floor division, intersect with the family count domain, and select the earliest tick across eligible phases/families and explicit boundary ticks. The result includes its word remainder. There is no native tick tail, velocity replay, candidate count enumeration or epsilon. A None result means this finite word prefix does not touch, not that the mission is impossible.

## Witnesses and measurement contract

The test compares streaming `try_tick` with the event map at arbitrary cycle/remainder queries across positive and negative binades and sign/zero crossings. It also checks native first touches for three detector heights, including missed floor detectors, and compares the motor/position endpoint at every detected inverse stop. Binade and detector neighbours are exercised separately. Synthetic exact half-ULP ties, negative operands, parity prefixes/cycles, a trillion-count jump and subnormal/normal boundaries isolate arithmetic ownership.

The poison environment flag is consumed only inside this test and flips the exact-half parity decision in a temporary mode. The verifier requires a running test to fail at that parity assertion and then requires the unpoisoned test to pass.

Distance measurements start at x=0.5 with the prepared cyclic motor and stationary z. The detector cell y=-59 intersects all vertical phases, ensuring a horizontal stop exists; y=-61 and -60 are independently checked against the native fold. Reported ticks exclude the transient owned by P1 and are not live click-to-arrival T. Segment counts are event-map regions through the exact stop; a crossing operation belongs to its exiting segment. The finite query bound comes from the detector's exact no-return edge and a downward displacement bound derived from the smallest law operand and the coarsest traversed U. One U is subtracted from the operand's integer floor as a proved rounding lower bound, including crossing additions; it is not a geometric tolerance. The measurement guards that its last addition cannot enter a coarser binade. Timings include deriving that bound, constructing the compressed families, their inverse, and direct map application to the stop, with native preparation and streaming validation outside the timed interval. Every sample is logged; min/median/max are measured at Idle priority on the current shared machine, not a latency promise.

Replay `jobs/3.2d-3-P2/verify.ps1`. Its output, input hashes, exactness assertion summaries and all duration samples are written under `D:/path-tracing-evidence/3.2d-3-P2`. A failing comparison remains red; it is never relaxed.

P3 can consume the test-scoped `Binary64` and `fold` API and its `Fold::pieces` affine families (region exponent, integer base/stride, phase offsets and count domain); there is no duplicate float displacement sum to invert.
