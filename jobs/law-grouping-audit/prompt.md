# The law's grouping table: same dynamics, different coats

**Goal:** one table that groups every behaviour in `engine/law/src/` by its transition shape, maps each group to a generator or a named kernel, and gives the honest Fermi floor of the rebuilt law, with zero code edits.

Builder: Opus. Read-only on `D:\path-tracing-wt\port-fold` (branch `wt/port-fold`, `38a49808`); write only `C:\Users\PASCAL\Desktop\path tracing\jobs\law-grouping-audit\table.md`. Read `map/` first. The Rust law is the oracle: it is right, it is not re-derived from the jar.

## His words (verbatim, 23 November)

"You group dynamics and shapes together and you find their common generators and then make them from that a generator approach by definition nearly is like a decade reduction."

## The table

1. Inventory every type, tick routine and block/entity family.
2. Group by transition shape (the same recurrence or state machine wearing a different coat), never by Mojang class or file.
3. Map each group to `body`, `sweep`, `ray`, `cell`, `clock` or `contact`, with its parameter columns and named scars, or to an explicit kernel (elytra, pistons, wire graph, portals, fluids), with why it is not a generator. Split, merge or add a generator when the shapes demand a cleaner basis. An Exactness column per group: bit-exact, server-exact or behaviour-exact (`map/architecture.md`).
4. Per group: the source files and their code lines (`scripts/law-lines.pl`), and the lines it costs rebuilt. Conservation: the groups plus one Glue/Unclassified row sum to the full `law-lines.pl` total; no line left out.
5. Witnesses: which are boundary rows (`state_in -> state_out`) and which tests pin internal types.
6. The floor: summed bottom-up from the groups' rebuilt estimates (generators + kernels + world/state plumbing, tests excluded), with no target to aim at.
7. The hot player core (tick state, movement, collision, clocks) and the cold world layer (`cell` and kernels): what is in each, and the field-by-field byte split of the 296 B `PlayerState`: what a plain movement tick touches, and what is lukewarm or cold.

Rows are inert data (constants, shapes, flags, tags): no rule scripts or opcodes in a table to make the Rust look smaller. Respect the exactness tiers in `map/architecture.md`: behaviour-exact systems keep the behaviour, not Mojang's helper classes.

Stop after the table. Your final message: the floor, the group count, and the five biggest groups.
