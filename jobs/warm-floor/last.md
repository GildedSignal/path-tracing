# New-command floor

Base e6de417. The initial rerun used the byte-identical integrator harness (SHA256 A17F18540B381639A7D3E0554FD1CB396B4452807CDE7D075BF631855579CE17). Evidence is in D:\path-tracing-evidence\warm-floor. The expanded harness uses the same PRNG, goals and predecessor states. It records all returned times including refusals, native ticks, terminal builds, kernel time and the complete returned end bits. Raw samples and inputs are in the logs.

## Layer 0

The projected swept-box intervals now intersect on both horizontal axes through Goal's closed half-space predicates. The planted walking case (X already inside, Z enters on tick two) returned 0 under the original dominant-axis choice and 2 after the fix. See minor-old.log and approach_waits_for_minor_axis.log. Vertical eligibility remains the terminal preimage's responsibility during approach. A normal-loop test compares all bits of the native and contact-observing states on both endpoints, at each of 24 ticks, for all seven frozen fixtures (336 comparisons). It passes.

The fresh checkout had CRLF-converted fixture bytes. The judge's original copies were available in wt/u2b-astra. Before restoring those original bytes, the normalized Git object of each copy and checkout was checked against e6de417. The replay checks the original physical-file hashes, including pin_gate.rs and premises.rs. There is no Git content diff for these fixtures.

## Layer 1

The byte-identical base harness returned all 60 standing commands and refused 33 of 60 moving commands. Its success-only moving summary drops all 33 timings. The expanded baseline logged 24 missing word preimages and nine corridor label mismatches with no scar. A separate route timer runs 60 batches of 16 public solves against the learned shortcut and retains the native 158-tick replay. Per-command tables are regenerated from the logs.

## Layer 2

One native tick falsifies fixed yaw-only contact pieces for arbitrary starting velocity. See engine/plan/tests/warm_floor.rs and warm_floor.log. Both starts have label 0, the same phase, the same gait, yaw index 0 and X velocity 2^-25. With starting Z velocity 0 the native X contact is Free and the outgoing X velocity bits are 0000000000000000. With Z velocity 0.25 X contact is Hold and the bits are 3e5178d520000000. Both observed traces equal try_tick bit for bit. LivingEntity.aiStep tests the squared norm of the horizontal velocity before applying the input. The same yaw would have to carry two distinct X operators. A velocity-dependent law guard can distinguish them, but a fixed partition of the yaw indices cannot. The proposed exhaustive seal and its one-ULP mutant have no passing generator to seal.

The inherited u1_new_bearing_learns_subnormal_switch also requires a positive native-tick Learn stage on a second bearing of a prepared flat label, while the new gate requires zero. The positive-tick assertion is retained.

## Layers 3 and 4

The solve reads 40-byte phase records and stored vertical extrema. events.rs indexes binary64 graphs by exponent, phase and parity in lazily allocated dense rows; streaming readers borrow stable graph references. The bundle stores dense kernel IDs with no duplicate kernel ownership. The codec compares owner identity, interns the four-class bare-legs label, and the terminal store indexes gait and phase directly. Existing per-kernel entry lookup still has velocity in its read signature.

The two-axis approach now saves chart crossings and reads phase cut points from its table. It is built per candidate trajectory. Bearings, envelopes and emitted word endpoints can still walk other paths. Output defers boundary event construction until the event list is actually read. terminal.rs computes a stop's horizontal face preimage from one scalar position pair. It formerly materialized both axes for both states at every bisection. Lower bounds prune impossible-to-improve stops before consulting their sets. lattice.rs eliminates one coordinate and evaluates the first exact integer slice with up to 256-bit products. The gate at all three distances records one quotient query; thin strips retain the existing floor-sum proof.

The generator source now supplies its support surface during the airborne phase. The sparse corridor still validates the support and body roles. The nine false corridor exceptions disappear. All outstanding command refusals report that no word preimage reaches the goal.

There is an independent native contradiction in the prescribed phase readout. The moving command to [-388,-61,-317] follows the first word for [58006,-61,128405]. The original output cuts the jumping word at tick 1643 and runs the sprint word for 23 ticks. Native replay touches at 1666. Keeping the jump word produces **no horizontal touch** through its derived no-return horizon of 1852. Consequently “the phase the touch tick fixes” does not exist for this valid stop. The solve with the exact two-axis approach instead returns 2103. This violates “Same T and same end-state bits as before on every goal you time.” See warm_floor.rs and the RETARGET records. That comparison is a failing acceptance check. The original word sequence remains a native witness; deriving the switch from the terminal preimage rather than a continued-gait touch requires the integrator's decision.

The baseline's interleaved medians are below the earlier report's 112/124/167 us. All three reported medians lie within this replay's min/max; agreement of the medians themselves is not established. The historical moving median of 253 ms excludes refusals. The all-command moving distribution below has a different denominator.

Replay with `powershell -NoProfile -ExecutionPolicy Bypass -File jobs/warm-floor/verify.ps1`. The driver exports e6de417 locally for the baseline and runs the current measurements, a separate counter build, and the inherited gates including their negative controls. score.ps1 compares all previously ready end bits and T, writes the table from raw logs and rejects out-of-budget commands. No live game or network was used.
