Continue the same job. Your failed-premise finding is right, and the fault is in the spec, not the plan. The integrator rules:

1. **The premise was framed wrong.** The two-yaw lattice chooses the **bearing**: how many cycles of each of the two neighbouring yaws. It was never meant to land on B by itself. The plan's loop already says how the goal is reached (`jobs\unified-plan\plan.md` section 2, steps 3–4; the old plain path's "the word to its touch of B, or from its last landing before that the remainder run"):
   - B is an event that breaks the big word's certificate, like any other;
   - `solve` cuts there and extends the next biggest word the kit holds over the remainder. Your positive control did exactly this: the sprint word without the jump, from the carried state, touching B at tick 2806325.

   This is the same `extend` step applied again, not a special case.
2. **Amend `t3_two_yaw`:** the acceptance is that the two-yaw counts bring the body to a carried state from which a **terminal word**, chosen by the same `extend` over the store's words, touches B. Assert the touch with the unchanged `Goal::touched`, bit-exact against the native replay. Keep your counterexample as a test: it proves the two-yaw counts alone miss B, and it must stay a test.
3. **Then build the kernel in full,** per `jobs\u1\spec.md`, and run every gate, including kill test 8's planted shortcut and the 13 footprint mutants.
   - The terminal word is found by preimage over the store's words and phases, by exact arithmetic, never a tick loop in `solve` when warm.
   - Report the million line's thinking time as a distribution, cold and warm. Say what dominates the warm time; your diagnostic's 1.16 ms is far above the plan's floor of about 5 µs.
4. **A failed premise stops only what depends on it.** Build everything else, and record what is red and why. Never run `launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
