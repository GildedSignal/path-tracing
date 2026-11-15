# 3.2d-3 slice B (R1): one role-class codec

**The spec** is `D:\path-tracing-wt\sets\jobs\3.2d-3\last.md`. Read all of it. Your slice is section 7's row B and section 2 (R1). The design behind it is `C:\Users\PASCAL\Desktop\path tracing\jobs\zones-design\design.md`, with its red-team in `redteam-opus.md` in the same folder.

**Why:** four parts of the code each answer "what does the law read here", each in its own way:
- footprints: `engine/law/src/footprint.rs`;
- pattern keys: `engine/law/src/pattern.rs`;
- the proposal class: `engine/words/src/class.rs`, `table.rs`;
- `law_digest`: on the unmerged branch `wt/plain-path`, in `D:\path-tracing-wt\plain\engine\law\src\pattern.rs`.

The plain-path judge failed `law_digest` for accepting a 128-bit digest as equality without comparing bytes (`D:\path-tracing-wt\plain-judge\jobs\plain-path-judge\verdict.md`, finding 4).

Julien (S134): classes depend on the kit. Grass and stone are the same to legs, "unless i have silk touch pickaxe", and a flower is air "unless i need to place a block and it bothers me".

## Decisions already made (do not reopen them)

1. **One interner** owns `ClassId` and `LabelId`, per section 2.
   - A class = intern(verb or kit query signature, role/mask, canonical answer bytes).
   - A digest lookup always compares bytes before it returns an existing id.
2. **The legs projection is section 2's four roles,** with masks from `footprint::Query`:
   - Support02 and Support05: Hooks|Loaded;
   - BodyFeet: Shape|Fluid|Hooks|Loaded;
   - Inside: Fluid|Hooks|Loaded.

   Use the omissions exactly as section 2 lists them, and nothing more.
3. **Break and place projections.** The law does not answer them yet (mining is isolated in `mining.rs`; there is no placement law), so they return `Unknown` and keep the fine block-state identity. Do not invent their answers. Where `mining.rs` already answers a query, use it.
4. **The old APIs** (footprints, pattern keys, class/table) become thin wrappers over the codec. There is no fifth encoding and no second terrain copy. A kit change swaps a projection table.
5. **The proof spine** (`Answer<Q>`, private constructors), the section role planes and the edit invalidation are as section 2 specifies. Build them in `engine/law` and `engine/words`, where the codec lives.
6. **`LabelId`** = intern(law/schema, body/kit projection, support-class tuple, medium-class tuple). Nothing consumes it yet: slice C's generator store will.

## Gate (section 7, row B), machine-checked

- A forced digest collision cannot alias unequal bytes in any of the four consumers.
- Air, buttons and plates are equal under legs: this is the butoonpressureplates non-issue (HANDOFF).
- A distant cobweb is harmless; a cobweb in the tube is hard.
- Verb distinctions and `Unknown` are retained.
- A proof for the wrong question fails to compile. A wrong state or version refuses.
- Edits invalidate exactly: a read cell, same-content edits, unrelated ticks, and both un-evented reads.
- `cargo test --offline` from `engine/` passes.
- `jobs\1.2-footprints\verify.ps1` catches all its mutants, and its mutant list is read from its pinned call sites.

## Do not touch

- `bot/`, `engine/plan/src/flight.rs`, `engine/plan/src/bin/`, any new `engine/native` crate, `engine/law/src/state.rs`: slice A (the resident channel) is being built beside you.
- `movement.rs`'s arithmetic (pinned; L0 unchanged).

No new search code, no new crates, no push, nothing live.

**Done means:** the gate passes, and `jobs\3.2d-3-B\last.md` lists every file changed, the gate's commands and outputs, and what slice C needs from the codec.
