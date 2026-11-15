# Port backlog: every GLM result against the Rust law

**Consumer:** Terra's port batches (engine.md, "How the law grows": GLM writes a rule with a witness, an independent GLM checks it, Terra integrates in batches) and the zones design's later slices (tools, edits, pearls, portals: `C:\Users\PASCAL\Desktop\path tracing\jobs\zones-design\redteam-opus.md`, finding 14).

The inventory (`C:\Users\PASCAL\Desktop\path tracing\jobs\inventory-map\report.md`, rows 7, 8, 13, 27) found the non-movement law isolated or missing: mining, resources, pushing, boats, item use. Nobody has mapped which checked GLM results are in the Rust law and which wait.

You read only. Read every file by its absolute path.

## Read

- **The lab's settled index:** `C:\Users\PASCAL\Desktop\path tracing\lab\wiki\index.md`, then every page it lists under `C:\Users\PASCAL\Desktop\path tracing\lab\wiki\`.
- **Results from 12 November on:** under `C:\Users\PASCAL\Desktop\path tracing\lab\results\`, each folder named `local-2026111[2-5]-*` that holds a `DONE` file. In each, read `dossier.md` (the claim) and `check.md` (the independent checker's grade, and anything it weakened or refused). Skip folders ending in `-zai-limit`: those runs hit a usage limit.
- **The Rust law:** every file under `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\`, plus `C:\Users\PASCAL\Desktop\path tracing\engine\world\src\items.rs`.

## For each result, give

- **lens**, and the checker's grade (reproduced, weakened, refuted, or no check);
- **the rule in one line**, with the jar class and offsets the dossier cites;
- **in the law?** One of:
  - IN: the file and line that implements it;
  - PARTIAL: what part is missing;
  - NOT IN: the file it belongs in.
- **wired?** Whether `movement::tick` or `Input`/`PlayerState` reaches it, or whether it sits isolated (inventory row 8);
- **consumer:** the track, word, rung or anomaly that needs it, from the dossier or the docs. Mark it "none named" if there is none.

## Output

Write `C:\Users\PASCAL\Desktop\path tracing\jobs\port-backlog\report.md`, appending every few results. Its parts:

1. **Counts:** reproduced or not; IN, PARTIAL or NOT IN.
2. **The table.**
3. **Proposed Terra batches.** Group the NOT IN and PARTIAL rules that touch the same law file and serve the same consumer. Order the batches by what they unblock. Today's priority is the plain path and 3.2d-3; then the five tracks that need place or interact words; then L0's freeze damage (`port-freeze-damage`, reproduced).
4. **Conflicts:** two results that disagree, or a result that disagrees with the Rust law.

Cite every claim by file and line. When something cannot be found, write "not found in" with the files checked.
