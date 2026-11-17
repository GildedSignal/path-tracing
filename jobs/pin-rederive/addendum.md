# Addendum to the pin re-derivation: the disc frame (S200, S202)

**Julien (S200, verbatim):** "why are we doing a x y for heaving when we could use complex numbers? the player acts as a ball of influence not as a cube and it does so at each step"
**Julien (S202, verbatim):** "it doesn't matter since we can change it instantly unlike momentum so now i get it ok now you can pitch opus"

**The integrator's reading, for you to test, not adopt:**
- Between contacts the horizontal tick is one complex affine map, `v ← d·v + a·e^{iθ}`: d (friction × drag) is the same on both axes, and the input vector is normalized. Its reach after n ticks is a disc, so free motion, the light cone, the bracket and a gait's count are rotation-invariant. The oblique million line should then cost about what the axis line costs, except for the binade crossings, which are truly per axis.
- The law's real asymmetry is the box on the grid: an axis-aligned body, collision axis by axis in `axisStepOrder` (`engine\law\src\shapes.rs:87`: y first, then the faster of x and z), and two separately rounded doubles with a sine table. Astra's 1-ULP pin is that cube showing through.
- The aim (yaw and pitch) carries no memory: the law writes it from the input each tick before reading it (`movement.rs:140`). So it is an argument of a word, never state. On legs, pitch isn't read at all; only the swim reads it (`movement.rs:410-420`).

**The question it adds to yours:** should free motion be represented in the disc frame, with the per-axis exact maps used only where the swept box meets a face (a defect), so that the box and the grid enter only at contacts? If so:
- Does that change where the pin's image should be cut, or what it is?
- What does it cost at the two extremes?
- Does it remove the oblique line's extra cost (169 µs axis, 261 µs oblique, warm, in Astra's record)?

If it's wrong or adds nothing, say why, from the law. Answer it inside the same proposal.
