# Finish the decision-relevant Minecraft 26.2 port

## Completion

The Rust engine can predict the parts of 26.2 that change what a bot can do, how long it takes, what it costs and what it changes. Use the exactness classes in [engine.md](engine.md). Player motion is bit-exact where specified. Server resources and damage agree with authoritative outcomes. Dynamic world behavior agrees at the resolution decisions need. A packet, random choice or state outside that contract is an explicit input or refusal. Rendering and cosmetic effects are outside this port.

L0 is the walking and swimming foundation. Its current successes do not close vehicle, equipment or dynamic world coverage. The first headless bot follows the port and uses the same law interfaces. Pathfinding and the choice of representation remain separate work.

## Work in dependency order, run independent work together

| Parallel lane | First useful result | Next boundary |
|---|---|---|
| Block collision and contact | Group registry states by collision context, friction, shape and executable hooks. Verify one uncovered rule against the jar. | Exercise shape and pose transitions, simultaneous contacts, chunk borders and interactions that change damage or access. |
| Player movement modes | Inventory dispatch and state transitions for grounded, airborne, climbing, swimming, riding and equipment-driven motion. Reproduce the first missing transition. | Cover mode changes and their interaction with input, collision, damage and resources. |
| Fluid state and propagation | Verify one shared neighbor-to-height, flow or update rule. Separate fluid evolution from player contact. | Cover scheduled propagation, source changes, waterlogging, falling fluids and fluid hazards across chunks. |
| Entity mechanics | Verify a deterministic contact or impulse from fixed initial state. Separate it from AI and random choices. | Cover competing contacts, riders and moving support, projectiles, and entity changes that affect player decisions. |
| Non-fluid world updates | Verify one consequential scheduled or neighbor-driven state change. | Add falling blocks, pistons, relevant redstone and other update chains that change access or timing. |
| Actions and resources | Verify one missing legality, timing or resource boundary from use, breaking, placement or damage. | Compose actions with updates and movement. Verify inventory and terrain restoration against the server. |

A named block, entity or vehicle is a probe of a rule. Track the rule's callers and state variants, then choose representatives by coverage and risk. Treat a new rule, context or ordering as a new case even when it occurs in a familiar block.

Use the existing law, world-read interfaces, server event order and vanilla recorder as the common interfaces. A stateful delta has one owner. Independent lane work happens in isolated scratch copies. The integrator sequences changes to shared Rust files and adjudicates failures. A new interface starts with a counterexample the current one cannot represent.

## How GLM is used

Give each job one concrete question, the local 26.2 class and method when known, the current Rust entry point, the smallest counterexample, the exact run command and a file boundary. Ask it to save a runnable witness as soon as it has one. The next deliverable is a candidate patch in the run folder with before/after evidence. A partial artifact is evidence to recover, even if the session ends before writing a dossier. A separate checker reproduces the result and tries an adversarial neighbor case. Preserve refusals and reproducible no-change results. Integrate only checked deltas.

Run six isolated GLM lanes. Recover useful prior witnesses when they match a current rule question. Split an unfinished job at its earliest observable transition and keep independent work in separate scratch copies. Read verdicts and the first divergent values before reading transcripts. Track independently reproduced deltas per batch.

## Where to spend tests

1. **Expose risk quickly.** Run the real 100× lab player against the port with frozen inputs. Begin with a small mixed route whose intended contacts are verified. Cross from ice into a flowing-water exit, make a low-ceiling step across a chunk seam and land or reverse at the transition. Later add boat dismount, elytra landing, live entities and toolkit edits as their laws enter.
2. **Keep two ladders.** Push one extreme at a time through orders of magnitude or adjacent representable values. Compose mechanisms in a separate chaotic lane. A passing easy case becomes a cheap control. It frees the next run to push a new boundary. Stop escalating a red case until its first divergence has an owner and a minimal witness.
3. **Reduce failures.** Preserve the raw trace, find the first differing field and tick, then remove inputs or terrain until the failure flips. Use stock speed for reductions and a small audit sample. When a failure only occurs at 100×, compare the server send, client application and resource order before assigning it to the port or the accelerated player. Keep the new boundary as a generator and a regression.
4. **Verify acceptance.** Freeze manifests, seeds, comparison fields and red-team controls before live use. Keep the existing L0 multi-seed gate and its four mutations. Give each added capability an independent positive witness and a planted fault. Check a held-out mixed corpus before calling the scoped port complete. Report distribution and worst case by capability, including unknowns and refusals.

## Integrator scoreboard

Rank the next delta by the decisions it can change, the shared mechanism it unblocks and the cost of an independent check. Track each capability as missing, partial, independently verified or mixed-tested. Keep the witness, patch, check, first divergence, and owner together. Run cheap isolated checks in parallel, then spend the live player on the transitions that combine them. Use the phase totals and the blocker list to update the rough estimate, while keeping the actual gate verdicts separate.

The next integration pass is the verified high-impact deltas plus the mixed 100× capture. Once the port clears the declared scope and review, build one scripted headless player through the normal server-visible movement path. Scale is measured separately after that single player matches the reference.
