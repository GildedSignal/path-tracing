# Post-mortem, 18 November: how flat ground came to re-learn its gait on every command

Julien (S231): "why did you had to reread the docs to not miss it? this is a very fundamental issue. make a post-mortem, this should have never happened i make sure it doesn't yet it does."

## What happened

On flat ground a new command costs 68 ms after Astra's run this morning (236 ms before it). About 59 ms of that goes into running about 4,700 law ticks to learn gaits that are already known: roughly 83 small runs per command. Each run copies the whole player state every tick and records every block read into a hash map, which comes to 80 MB. The code files each gait under its exact yaw and its starting speed's exact bits, so a new direction never finds anything in the store. It also tries all three words as rival routes, and it pre-builds every way of stopping from every tick of every cycle.

Four rules in the living docs forbid this. Each was written before the code:

| Rule | Where | In force since |
|---|---|---|
| Store the generator, not the generated: the recurrence and its parameters, keyed by those parameters, shared by every place and bot; arcs derived when looked at, never stored tick by tick; states that differ below `aiStep`'s thresholds merge | [engine.md](../engine.md), "Continuous where it helps" | 5 November (`e0274ef`) |
| Words live in velocity space: the tick is `v ← d·v + a·u(yaw)`, a contraction onto the gait | [representation.md](../representation.md) section 10 | 16 November (`36cbfee`) |
| The biggest word first; one corridor query gives its count; the remainder goes to the next word | [learning.md](../learning.md) section 0, step 3; [vocabulary.md](../vocabulary.md) section 5 | 12 November (S93), counts 16 November |
| The aim is an argument, never part of an entry key | vocabulary.md section 5 (S202) | 17 November (`9c07ea7`) |

## Why the integrator had to re-read the docs to see it

1. **This session skipped the re-read that AGENTS.md's first line requires (S164).** At the start I read PRINCIPLES (in part), HANDOFF, plan.md, clan.md and the first 40 lines of learning.md. I did not read representation.md, engine.md, vocabulary.md, scales.md or ladder.md. The four rules above live in the files I skipped.
2. **I explained the design from the code.** Asked what goes on in the bot's head, I read `solve.rs` and `store.rs` and described what they do as if that were the design. The question the docs ask first is what the design says. Only then comes how far the code is from it.

Those two are why it took this session three of his turns to see it. Why it was there at all is older.

## How it got into the code, and why every check passed it

| When | What |
|---|---|
| 16 Nov 20:11 | **The integrator's own spec puts the yaw into the key.** `jobs/u1/spec.md:7`: "the inner key `(MotorKeyId, program, yaw_index, pitch_bits)`". It also calls preparation "cold" and reports it beside a "warm" figure (`jobs/u1/spec.md:18`), so its cost reads as a one-time warm-up. That violated engine.md's "keyed by those parameters", in force since 5 November. Nobody checked the spec against engine.md. |
| 16 Nov | Astra builds exactly that. The u1 judge checks it **against the spec**, passes it, and even describes the cost structure ("cycle_period × 3 terminal jobs", `jobs/u1-judge/last.md:29`), but only as advice: it measured the repeated question, and the spec allowed the rest. |
| 17 Nov | S202 and S203 write "the aim is an argument, never part of an entry key" into vocabulary.md. **The docs changed and the code was never checked against them.** The u2b judge confirms the rule for the joints' sets (`pattern.rs:318`) and notes "a one-ulp velocity change is still distinguished" (`jobs/u2b-judge/last.md:25`), but the gait store's yaw key goes unexamined. |
| 17 Nov | The idiot-index report (`jobs/hundred-ns`) and the warm-floor spec take the code as given and optimise it in "layers" (a yaw-free generator, data shaped like the read, one walk per axis). The spec lists engine.md under "Read also" but never names its rule as one the code breaks. It asks for bit-equality on 65,536 yaws and the same end bits on every goal: precision on the widest ground there is, against ES 21 and SR-71. |
| 18 Nov | Astra spends a whole Codex 5-hour window and about 15% of the week in 45 minutes, inside that framing, and stops on two bit-level contradictions. One of them is the `aiStep` threshold, which engine.md already handles in one sentence. |

## The causes, by mechanism

1. **Rules are prose; gates are numbers.** No living-doc rule became a gate. "A new direction on known ground prepares zero law ticks" would have failed on the first day. The kill tests measured times, and the time measured was the repeated question (S223).
2. **Judges judge the diff against the spec, and the spec was the integrator's.** When the spec breaks a doc, the judge certifies the violation. This is the same failure as 16 November: the fork was in the integrator's change and was passed downstream.
3. **A doc change is never followed by a code audit.** S202 changed the rule; the code that broke it was already built and stayed as it was. Argon's lint checks our docs against his words (S211), not our code against our docs.
4. **Optimising instead of re-deriving.** A cost three to four orders above its Fermi floor is a signal of the wrong representation (AGENTS.md, "No patches"; ES 19), not a list of speed-ups. The integrator wrote layers of speed-ups, so a builder spent a window inside the wrong representation.
5. **Labels that hide cost.** "Cold" and "warm" made a per-command cost look like a once-per-world one. Without that, cause 1 would have been seen days earlier.
6. **The integrator's re-read is a rule without a check.** S164 says re-read every doc after a compaction or a new session. Nothing records that it was done, and this session did not do it.

## What it cost

- A Codex 5-hour window and about 15% of the Codex week (Astra 6.2, 18 November). Codex's week is at 87%.
- The 16–18 November work on flat-ground speed (u1, u2b, the idiot-index report, warm-floor) optimised a store the docs forbid. The certified pieces stay: joints and sets, families, points and holds, the exact integer maps. The gait store and the solve's candidate structure have to be rebuilt to the docs.
- Three of his turns today to get the integrator back to its own docs.

## What changes (proposed; his to approve)

1. **Every spec opens with a conformance section:** each living-doc rule it touches, quoted with file and line, and each turned into a gate that fails when the code breaks it. For flat ground: a new direction or a new start speed on known ground prepares zero law ticks; the aim is not in any key; the count comes from one corridor query; no word is tried that is not the biggest one that fits.
2. **Judges get the living docs' rules as well as the spec,** and their verdict has a line "conformance to the docs", which can FAIL a diff that matches its spec.
3. **A rule changed in a doc is followed the same day by a code audit:** a grep or a test for the old form, with each violation written into HANDOFF's anomalies.
4. **Argon's lint gains a second pass, code against the docs,** after each phase, beside docs against his words.
5. **A cost more than one order of magnitude above its Fermi floor goes back to the integrator as a representation question,** never as a list of optimisations.
6. **The integrator's re-read is recorded:** after every compaction or new session, HANDOFF's "Start here" names the docs read in full, and no spec or explanation is given before it.
7. **Important prompts are shown to him in chat before launch** (S230, already in force).
