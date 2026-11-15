# GLM digest: what the newest lab results change

**Consumer:**
- the integrator, who has not read these results;
- `lab/wiki`, the lab's settled index;
- the docs and the 3.2d-3 spec, which several of them test.

You read only, and write only your report. Read every file by its absolute path.

## Read

- **The index:** `C:\Users\PASCAL\Desktop\path tracing\lab\wiki\index.md`.
- **The results:** every folder under `C:\Users\PASCAL\Desktop\path tracing\lab\results\` named `local-20261114-*` or `local-20261115-*` that holds a `DONE` file. In each, read `dossier.md` and `check.md` (the independent checker's grade; where author and checker disagree, the checker wins). Skip folders ending in `-zai-limit`.
- **Context for what each tests:**
  - `C:\Users\PASCAL\Desktop\path tracing\jobs\zones-design\redteam-opus.md` (tests T4, T7, T11);
  - `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2d-3\spec.md`;
  - `C:\Users\PASCAL\Desktop\path tracing\docs\engine.md` and `docs\representation.md`.

## Write

`C:\Users\PASCAL\Desktop\path tracing\jobs\glm-digest\report.md`, appending as you go.

1. **Per result:**
   - the lens;
   - the grade;
   - the settled fact in one or two lines, with jar class and offsets;
   - whether `lab\wiki\` already has a page for it (name the page), or which page it should have;
   - what it changes: a doc line, a spec line, an anomaly, or a port batch. Name the file and line it contradicts or extends.
2. **The three premise tests** (`trial-food-crossing`, `trial-offset-monotone`, `port-wormhole-facts`): what each confirms or refutes in the red-team and the spec. Give the exact spec or doc lines to correct, with replacement text.
3. **Wiki entries to write:** for each result that has no wiki page, a draft page in the format of the existing pages in `lab\wiki\` (read two first).

Cite every claim by file and line. Never guess.
