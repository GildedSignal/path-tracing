# Event forms: every ported mechanic, classified for the non-tick system

**Why:** Julien (S148): "we'll need to improve the port after keeping the same behavior but adapting them to our non tick based system, will give massive gain, actually i suppose from now on we could start that".

**The rule:** the tick law (the port) is never replaced; it stays the judge. Each mechanic also gets an **event form**: the events at which its state changes in a way that matters, and between them a closed form or a stored table. The event form is accepted only when it matches the tick law bit for bit over the recorded runs. This is "store the generator, not the generated" and "compute only surprises".

**Consumer:**
- the integrator, who will spec event-form jobs in the order the planner needs them;
- the 3.2d-3 premise jobs P1–P4, which are the first four event forms (transient memo, binade integer maps, two-yaw lattice, food guard).

You read only, and write only your report. Read every file by its absolute path.

## Read

- **The port:** `C:\Users\PASCAL\Desktop\path tracing\jobs\port-backlog\report.md` (73 results: IN, PARTIAL, NOT IN) and `jobs\inventory-map\report.md` (the mechanism map).
- **The law as written:** `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\*.rs` and `engine\words\src\*.rs` on main.
- **The design:** `jobs\zones-design\design.md` (U1: every wall is an event; closed form between events), `jobs\zones-design\redteam-opus.md`, `jobs\3.2d-3\spec.md`, and `jobs\spec-fixes\report.md` sections 2–3 (P1–P4).
- **The lab's settled facts:** `lab\wiki\index.md` and the pages it links to.

## Write

`C:\Users\PASCAL\Desktop\path tracing\jobs\event-forms\report.md`, appending as you go.

1. **One row per mechanic** in the law (every counter, recurrence, threshold and check in `engine\law\src\`, and every port-backlog result), with:
   - the mechanic and its code (file and line);
   - **its kind**:
     - (a) an integer counter or phase, whose next event is arithmetic;
     - (b) a float recurrence fixed by its starting bits, stored once per start (name the key that makes two starts share a table);
     - (c) a wall: a condition that cuts segments (block edge, binade crossing, threshold, yaw switch, chunk loaded);
     - (d) none of these, saying why;
   - its events: what ends a segment, and how the next one is found without ticking;
   - **its form between events**, written out: for (a) the formula; for (b) the table's key and contents; for (c) the lookup;
   - **the certificate:** which recorded runs or tests would prove it bit-exact against the tick law (name them: `lab\tracks\runs\*.jsonl`, the L0 kept runs, `engine\*\tests\*.rs`);
   - **its consumer:** which track or word needs it (straight lines, the hard track `lab\tracks\hardtrack.json`, a named word), or "none yet".
2. **The order:** the mechanics on the straight-line path first (legs on flat ground, food, sprint), then the hard track's, then the rest. Mark which are already covered by P1–P4.
3. **Couplings:** pairs of mechanics whose events interact (food and sprint, freeze and movement speed), where one's event moves the other's next event. Say how the combined next event is found.
4. **Refusals:** any mechanic whose event form you could not derive from the code, with what is missing.

Cite every claim by file and line. Never guess: write "not found in" with the files checked. Invent no thresholds or constants: every number comes from the code or the jar offsets the code cites.
