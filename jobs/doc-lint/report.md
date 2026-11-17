# Doc Lint Report (against Julien's words, S1–S211)

Scope: `AGENTS.md`, `docs/PRINCIPLES.md`, `docs/plan.md`, `docs/HANDOFF.md`, `docs/learning.md`, `docs/representation.md`, `docs/scales.md`, `docs/vocabulary.md`, `docs/engine.md`, `docs/clan.md`, `docs/ladder.md`, `docs/server.md`, `docs/footprints.md`, and `lab/glm.md`, linted against `docs/reference/julien-2026-11-04.md` (S1–S43, X1–X6), `docs/reference/julien-2026-11-09.md` (S44–S63), `docs/reference/julien-2026-11-10.md` (S64–S211, X7–X16), and `docs/PRINCIPLES.md`.

---

## 1. Findings (most consequential first)

### F1. `AGENTS.md` and `docs/clan.md` invert Astra and Terra's roles against S174 and S209
- **Doc and line:**
  - `AGENTS.md:23–24`
  - `docs/clan.md:41–42` and `docs/clan.md:118`
- **What it says:**
  - `AGENTS.md:23`: `| Builder | GPT gpt-6.1-terra through codex exec (S47) | all GPT building, hard tasks included (S99: Sol is not used); it does best on a spec that leaves nothing open |`
  - `AGENTS.md:24`: `| Hard tasks | GPT gpt-6-astra, sparingly | what Terra stalls on: convergent math, hard design within a spec |`
  - `docs/clan.md:41`: `| Terra | OpenAI | Checked (build) | a spec that leaves nothing open, built exactly, unattended |`
  - `docs/clan.md:42`: `| Astra | OpenAI | Frozen (convergent) | the math Terra stalls on: a bound, a monotonicity, a closed gap |`
  - `docs/clan.md:118`: `| Terra (build), Astra (hard design within a spec) | ... |`
- **His words:**
  - **S174** (`docs/reference/julien-2026-11-10.md:611–613`): `"why is terra building the core of the new algo what are you doing"` / `"opus and astra for important things, this is clear in the doc"`
  - **S209** (`docs/reference/julien-2026-11-10.md:777`): `"the goal is to maximize astra more we have other subs with access to cheaper models, terra is more there to backfill those odd 20% left for example, we need quite a bit of intelligence in this project."`
- **Why it's an issue:** `AGENTS.md:23–24` and `docs/clan.md:41–42, 118` still tell every new session that Terra does `"all GPT building, hard tasks included"` and that Astra is used `"sparingly"` only for `"what Terra stalls on"`. This directly contradicts `docs/clan.md:142` (`"The Codex window is Astra's (S209: "the goal is to maximize astra more"). Terra backfills the leftover ~20%."`) and `docs/plan.md:68–69`, and is the exact rule drift that caused the S174 mistake.
- **Smallest fix:**
  - In `AGENTS.md:23–24`, replace:
    ```markdown
    | Builder | GPT `gpt-6.1-terra` through `codex exec` (S47) | all GPT building, hard tasks included (S99: Sol is not used); it does best on a spec that leaves nothing open |
    | Hard tasks | GPT `gpt-6-astra`, sparingly | what Terra stalls on: convergent math, hard design within a spec |
    ```
    with:
    ```markdown
    | Core builder | GPT `gpt-6-astra` through `codex exec` (S174, S209) | the core algorithm, convergent math and hard design/builds; owns the Codex window (S99: Sol is not used) |
    | Backfill builder | GPT `gpt-6.1-terra` through `codex exec` (S47, S209) | well-specified pieces and tooling; backfills the ~20% of the Codex window Astra leaves |
    ```
  - In `docs/clan.md:41–42`, replace:
    ```markdown
    | Terra | OpenAI | Checked (build) | a spec that leaves nothing open, built exactly, unattended |
    | Astra | OpenAI | Frozen (convergent) | the math Terra stalls on: a bound, a monotonicity, a closed gap |
    ```
    with:
    ```markdown
    | Astra | OpenAI | Frozen / Judged (core build & math) | the core algorithm, convergent math and hard builds; owns the Codex window (S174, S209) |
    | Terra | OpenAI | Checked (build) | a spec that leaves nothing open, built unattended; backfills the ~20% of the Codex window Astra leaves (S209) |
    ```

---

### F2. `AGENTS.md`, `docs/clan.md`, and `docs/plan.md` contradict S210 and S211 on Argon and Hydrogen building and running commands
- **Doc and line:**
  - `AGENTS.md:28`
  - `docs/clan.md:44, 47–53, 85, 105, 120, 174–176`
  - `docs/plan.md:70`
- **What it says:**
  - `AGENTS.md:28`: `| Google | Argon and Hydrogen through Antigravity, from 13 November (S102) | Hydrogen takes the complex questions (S173), Argon reads: the corpus traversal, the untrusted channel, his primary sources. Hydrogen is a second wide lane. Neither builds and neither judges ([docs/clan.md](docs/clan.md)) |`
  - `docs/clan.md:44`: `| **Argon** | **Google** | **claimed: corpus traversal; untrusted reading; primary sources** | **three rows, none of them building. ...** |`
  - `docs/clan.md:47`: `**Why Argon gets three rows and none of them is building.** Its benchmarks say plainly that it should not build and should not judge (section 4b).`
  - `docs/clan.md:85`: `| wide cheap lanes | Hydrogen | unknown | no run has survived its permission denials yet |`
  - `docs/clan.md:105`: `Its row is therefore narrowed to that and nothing else`
  - `docs/clan.md:120`: `| Argon (read, traverse), Hydrogen | .\scripts\launch.ps1 <id> argon (or hydrogen) | ≤ 1 Argon or 2 Hydrogen; read-only |`
  - `docs/clan.md:174`: `- **Hydrogen** has never run a real job here: its row is still a claim with no evidence.`
  - `docs/plan.md:70`: `| Complex questions and wide reading | Hydrogen; Argon for primary sources |`
- **His words:**
  - **S210** (`docs/reference/julien-2026-11-10.md:781, 789`): `"Hydrogen also can’t run code: its sandbox refused every command in the builds. So it can’t build or test." what?` / `"we can leverage argon better than that."`
  - **S211** (`docs/reference/julien-2026-11-10.md:795–797`): `"for the linting i meant our own docs should be lintet based on what i said looking for issues from time to time."` / `"argon can build yes."`
  - **S209** (`docs/reference/julien-2026-11-10.md:777`): `"do we really need hydrogen for questions? in what way is it useful for? ... same, are we using argon right?"`
  - **S197** (`docs/reference/julien-2026-11-10.md:725`): `"wow, hydrogen's build a very good. this will be used as a benchmark for the building of the swarm for when we'll get to that part"`
- **Why it's an issue:** `docs/clan.md` contradicts both Julien (S197, S209–S211) and its own section 5c (`docs/clan.md:142`: `"Hydrogen and Argon read and build on their own Google meters"`, and `docs/clan.md:144`: `"Antigravity's "cannot run commands" was our launcher, not the models (S210)"`). `AGENTS.md:28` still states `"Neither builds"`, and `docs/clan.md:85, 174–176` still claims Hydrogen has never survived a run.
- **Smallest fix:**
  - In `AGENTS.md:28`, replace:
    ```markdown
    | Google | Argon and Hydrogen through Antigravity, from 13 November (S102) | Hydrogen takes the complex questions (S173), Argon reads: the corpus traversal, the untrusted channel, his primary sources. Hydrogen is a second wide lane. Neither builds and neither judges ([docs/clan.md](docs/clan.md)) |
    ```
    with:
    ```markdown
    | Google | Argon and Hydrogen through Antigravity, from 13 November (S102) | Argon lints our docs against his words, reads primary sources and the untrusted channel, and builds (S211). Hydrogen takes complex questions, digests and builds (S173, S197, S210). Neither judges ([docs/clan.md](docs/clan.md)) |
    ```
  - In `docs/plan.md:70`, replace:
    ```markdown
    | Complex questions and wide reading | Hydrogen; Argon for primary sources |
    ```
    with:
    ```markdown
    | Doc linting, primary sources, builds and wide questions | Argon (doc lints, primary sources, builds, S211); Hydrogen (complex questions, digests, builds, S173, S210) |
    ```
  - In `docs/clan.md:44, 47, 85, 105, 120, 174–176`, remove `"none of them building"`, `"read-only"`, and `"has never run a real job here"`, aligning sections 3, 4, 4b, 5, and 7 with section 5c (`clan.md:142, 144, 148`).

---

### F3. `AGENTS.md`, `docs/HANDOFF.md`, and `docs/clan.md` still instruct running Opus on Antigravity against S115 and S129
- **Doc and line:**
  - `AGENTS.md:29` and `AGENTS.md:37`
  - `docs/HANDOFF.md:14`
  - `docs/clan.md:119`
- **What it says:**
  - `AGENTS.md:29`: `| The spare meter | Opus 5.5 and Sonnet 5.5 **through Antigravity** | the same minds on a meter that is not Julien's Anthropic plan: prefer it for judging and red-teaming, and keep the native plan as the reserve |`
  - `AGENTS.md:37`: `Antigravity, per meter (S105): one Argon or two Hydrogen; one Opus or two Sonnet; adjust on a measured drain. scripts/launch.ps1 enforces all of these. **The native Anthropic plan is the reserve and is exhausted last** (S102): Claude-shaped work goes to Antigravity's Claude meter first...`
  - `docs/HANDOFF.md:14`: `on Antigravity (S105) one Argon or two Hydrogen, and one Opus or two Sonnet.`
  - `docs/clan.md:119`: `| Sonnet on Antigravity (judge) | .\scripts\launch.ps1 <id> sonnet [-ReadDir <worktree>] | ≤ 2 Sonnet or 1 Opus; weekly meter; inline the diff |`
- **His words:**
  - **S115** (`docs/reference/julien-2026-11-10.md:325`): `"yeah so antigravity is sonnet only you can use opus"`
  - **S129** (`docs/reference/julien-2026-11-10.md:398`): `"Opus on Antigravity spent that week's Claude meter with no output." these are very bad failures and should NOT happen again, there should be tests or something`
- **Why it's an issue:** `docs/PRINCIPLES.md:286, 300` and `docs/clan.md:82` record that `launch.ps1` and `test-antigravity.ps1` refuse Opus on Antigravity and that Opus runs only as a native sub-agent. Yet `AGENTS.md:29, 37`, `docs/HANDOFF.md:14`, and `docs/clan.md:119` still say Antigravity runs Opus (`"one Opus or two Sonnet"`).
- **Smallest fix:**
  - In `AGENTS.md:29`, replace:
    ```markdown
    | The spare meter | Opus 5.5 and Sonnet 5.5 **through Antigravity** | the same minds on a meter that is not Julien's Anthropic plan: prefer it for judging and red-teaming, and keep the native plan as the reserve |
    ```
    with:
    ```markdown
    | The spare meter | Sonnet 5.5 **through Antigravity** (S115: Sonnet only; Opus is native) | judging on a meter that is not Julien's Anthropic plan, keeping the native plan for orchestration and Opus red-teams |
    ```
  - In `AGENTS.md:37`, `docs/HANDOFF.md:14`, and `docs/clan.md:119`, replace `one Opus or two Sonnet` / `≤ 2 Sonnet or 1 Opus` with `up to two Sonnet (S115: Antigravity is Sonnet only; Opus runs native)`.

---

### F4. `lab/glm.md`, `docs/scales.md`, and `docs/clan.md` assign non-port work to GLM against S173
- **Doc and line:**
  - `lab/glm.md:13–24, 29, 41–43`
  - `docs/scales.md:59`
  - `docs/clan.md:43, 45`
- **What it says:**
  - `lab/glm.md:17–24`: Lists five GLM lenses: `Breaker`, `Worst cases`, `Jar reader`, `Pre-freeze red-team`, and `Probe` (`a measurement the plan needs (border states per chunk, table sizes)`).
  - `lab/glm.md:29`: `- Inside its lens, a run picks its own question and does real work: code, experiments, derivations.`
  - `docs/scales.md:59`: `- **Proposers, now, in the lab.** GLM isn't the scarce resource. A lens proposes programs in the frozen program language, the law scores them, the archive keeps the best and feeds them back into the next prompt (FunSearch's loop).`
  - `docs/clan.md:43`: `| GLM | Z.ai | Checked (wide) | many cheap narrow questions, each reproduced by a second session ([lab/glm.md](../lab/glm.md)) |`
  - `docs/clan.md:45`: `| **Hydrogen** | **Google** | **claimed: Checked (wide)** | **a second cheap-wide lane beside GLM, on the same lens queue, graded against GLM's own reproduced answers. ...** |`
- **His words:**
  - **S173** (`docs/reference/julien-2026-11-10.md:607`): `"use glm just for the port and hydrogen for more complex questions, glm is not very strong"`
- **Why it's an issue:** `AGENTS.md:27` states `"**the port only** (S173: "glm is not very strong"): jar-backed port deltas, each checkable by a machine ([lab/glm.md](lab/glm.md)). Questions and premise tests go to Hydrogen or Terra"`. Yet `lab/glm.md` (which `AGENTS.md:27` links to), `docs/scales.md:59`, and `docs/clan.md:43, 45` still assign open-ended questions, probes, red-teaming, and FunSearch program proposals to GLM. Also `lab/glm.md:41–43` omits `scripts/glm-queue.ps1` (which paces 1–8 lanes to the meter per S100, S123).
- **Smallest fix:**
  - In `lab/glm.md:13–25`, replace the five-lens table with the S173 rule: GLM runs **jar-backed port deltas and `javap` law extractions only**, each with a machine-checked witness reproduced by an independent checker; questions, probes, and premise tests go to Hydrogen or Terra; launch via `scripts\glm-queue.ps1 -Queue lab\queue.txt` (S100, S123).
  - In `docs/scales.md:59`, replace `GLM isn't the scarce resource. A lens proposes programs...` with `A proposer on a spare meter proposes programs... (S173: GLM is for the port only)`.
  - In `docs/clan.md:43, 45`, update GLM's row to `the port only: jar-backed port deltas reproduced by a second session (S173)` and Hydrogen's row to remove `on the same lens queue, graded against GLM's own reproduced answers`.

---

### F5. `AGENTS.md` and living docs omit rules Julien stated (and rules from the 16 November post-mortem)
- **Doc and line:**
  - `AGENTS.md:46` and `AGENTS.md:38–40`
- **What it says:**
  - `AGENTS.md:46`: `- **Julien's words.** His turns go verbatim into docs/reference/, and the register in PRINCIPLES.md cites them; models never reword them. For a new idea, think and talk with him first, and build when he says so.`
- **His words:**
  1. **S179 (before)** (`docs/reference/julien-2026-11-10.md:637`): `"ok explain me what's the next steps now when i ask to explain you explain you don't do"` and **S181** (`docs/reference/julien-2026-11-10.md:645`): `"Explain it first"`.
  2. **S161** (`docs/reference/julien-2026-11-10.md:555`): `"what we can't have two paths like that "easy" or "hard" it should be the same mechanism we're deviating here this is extremely back, read again all the doc and what i said and the plan i can sense that you don't have them in mind"` (and `docs/reference/postmortem-2026-11-16.md:28–30`: fold design turns into the living docs the same day; check every spec/judge prompt against the one-mechanism test; register statuses `"Talking it through"` and `"Open"` mean nothing built).
  3. **S177–S178** (`docs/reference/julien-2026-11-10.md:625, 629`): `"so you used 60% of the window to write it into the doc something that it had already written?"` (recorded in `HANDOFF.md:21` as `"never resume a long-context agent for a small edit (S177)"`, but missing from `AGENTS.md:39` "Resuming", which only mentions Astra's cold cache).
  4. **S179** (`docs/reference/julien-2026-11-10.md:633`): `"what's up with those made u1 p3 and so on that means nothing say names i'm over those made up names just say what they are."` (recorded in `plan.md:3`, but not in `AGENTS.md`).
  5. **S180** (`docs/reference/julien-2026-11-10.md:641`): `"ok lunch the relevent agents if not already done make sure they have the right doc in the prompt"` (`AGENTS.md:40` says prompts start with PRINCIPLES' goal and philosophy + `jobs/<id>/prompt.md`, but omits the 16 November post-mortem / S180 rule that builder, judge, and red-team prompts must include the relevant living docs so they check the one-mechanism rule).
- **Why it's an issue:** `AGENTS.md` is the always-on rule file injected into every session. Leaving these rules only in `HANDOFF.md`'s transient `"Start here"` or `postmortem-2026-11-16.md` means they vanish when `HANDOFF.md` is rewritten.
- **Smallest fix:**
  - In `AGENTS.md:39` ("Resuming"), append: `Never resume a long-context agent just to copy or format text it already wrote (S177).`
  - In `AGENTS.md:40` ("Prompts"), append: `Include the living docs the job touches (`plan.md`, `learning.md`, `representation.md`, `scales.md`); every judge and red-team checks the one-mechanism rule: no terrain kind gets its own code path (S161, S180).`
  - In `AGENTS.md:46` ("Julien's words"), replace:
    ```markdown
    For a new idea, think and talk with him first, and build when he says so.
    ```
    with:
    ```markdown
    When he asks for an explanation or next steps, explain only and do not act (S179, S181). Fold his approved design turns into the living docs the same day, with plain names before any code (S163, S179); "Open" or "Talking it through" in the register means nothing built.
    ```

---

### F6. `docs/PRINCIPLES.md` has corrupted paths with literal TAB characters (`\t`) at lines 363 and 365
- **Doc and line:** `docs/PRINCIPLES.md:363` and `docs/PRINCIPLES.md:365`
- **What it says:**
  - Line 363: ``the documents kept outside git (`D:path-tracing-evidence	hree-builds`), the repository being public``
  - Line 365: ``the 24 region files the builds created were moved aside to `D:path-tracing-evidence	hree-builds-removed-20261117` with the server stopped``
- **His words:** **S129** (`docs/reference/julien-2026-11-10.md:398`): `"these are very bad failures and should NOT happen again, there should be tests or something"` (referring to Antigravity runs killed by malformed/driveless paths).
- **Why it's an issue:** Both paths were written through an unescaped string where `\p` dropped the backslash after `D:` and `\t` became a literal ASCII TAB (`0x09`) followed by `hree-builds`. Besides being garbled text, any agent that tries to read that path will fail immediately.
- **Smallest fix:**
  - In `docs/PRINCIPLES.md:363`, replace `` `D:path-tracing-evidence	hree-builds` `` with `` `D:\path-tracing-evidence\three-builds` ``.
  - In `docs/PRINCIPLES.md:365`, replace `` `D:path-tracing-evidence	hree-builds-removed-20261117` `` with `` `D:\path-tracing-evidence\three-builds-removed-20261117` ``.

---

### F7. `docs/PRINCIPLES.md` Register is missing rows for 18 turns (including substantive turns S176, S180, S184, S186, S192, S193, S194, S195)
- **Doc and line:** `docs/PRINCIPLES.md:97–370`
- **What it says:** `docs/HANDOFF.md:8`: `New turns go there verbatim, with a register row in PRINCIPLES.`
- **Missing turns (no row in `docs/PRINCIPLES.md`):**
  1. **S186** (`docs/reference/julien-2026-11-10.md:663–667`): `"what drives most of those 500µs?"` / `"why do we need to clone? can't we have a smarter operation? do we need clone? it is important to get even the simplest case right, if we can't make them right the rest will compound badly"` — cited in `docs/plan.md:47`, but has no row in `docs/PRINCIPLES.md` between line 352 (S181–S182) and line 353 (S188).
  2. **S194** (`docs/reference/julien-2026-11-10.md:711–713`): `""What I need from you to race the new bot: nothing yet. It can't race in your game until the bot's runner exists." true but it takes time for me to race, like Elon we work massively in parallel. just remove the unused maps that's bloat. you can import all eight. just make sure the spawn points are good. oh yes too for the mod for the tracks make sure autocompletes work it's an not great to not have that feature oh yes and another quite frustrating issue is that i often have to type /track restore before it doesn't properly start them and also i have to wait for too long when on the block for it to properly load which is frustrating where you're restarting the track often or even worse finish it to then realize it did not count. don't add the ice boat madness i don't have great hopes that it'll work"` — has no row in `docs/PRINCIPLES.md` between line 356 (S191) and line 357 (S197).
  3. **S192** (`docs/reference/julien-2026-11-10.md:703–705`): `"lunch them both, what do you need from me to race? oh yes i've tested all the maps and only few were good, one is a bit too hard for a neophyte like me but interesting, we need dropper, elytra and maybe others with nice progression systems that don't get broken via the scoreboard like the others"` — no row in `docs/PRINCIPLES.md`.
  4. **S193** (`docs/reference/julien-2026-11-10.md:707–709`): `"the maps with tracks on them are the good ones"` — no row in `docs/PRINCIPLES.md`.
  5. **S195** (`docs/reference/julien-2026-11-10.md:715–717`): `"i'm curious of something, have hydrogen design a pretty pagoda in my map next to spawn i'm curious of what it can do, then report usage used for it don't let it half bake it"` — no row of its own (only mentioned parenthetically in S197 at line 357).
  6. **S196** (`docs/reference/julien-2026-11-10.md:719–721`): `"i'll see it in game and then add tracks to the other maps"` — no row.
  7. **S176** (`docs/reference/julien-2026-11-10.md:619–621`): `"sorry i meant fable"` — corrects S175 (`"hydrogen's specs? why would hydrogen write the specs for astra?"`). Because S176 is missing, `docs/PRINCIPLES.md:349` records only S175 (`"why would hydrogen write the specs for astra?"`) without his correction that he meant Fable.
  8. **S180** (`docs/reference/julien-2026-11-10.md:639–641`): `"ok lunch the relevent agents if not already done make sure they have the right doc in the prompt"` — no row.
  9. **S184** (`docs/reference/julien-2026-11-10.md:655–657`): `"if glm and gemini are fresh then use them unless you can't. what's left before more interesting things can happen in testing?"` — no row.
  10. **S185** (`docs/reference/julien-2026-11-10.md:659–661`): `"Opus re-derives it"` — no row.
  11. **S187** (`docs/reference/julien-2026-11-10.md:669–671`): `"wait and do it all at once"` — no row of its own (only parenthetically cited in S188 at line 353).
  12. **S135** (`docs/reference/julien-2026-11-10.md:437–439`): `"btw someone is on the computer so don't start things in the foreground"` — no row.
  13. **X7** (`docs/reference/julien-2026-11-10.md:209–211`): `"hey on my minecraft path finding project i made good progress look at the code on github https://github.com/GildedSignal/path-tracing and update your beliefs based on the new developments, also had my first semi emergent property"` — no row.
  14. **S31** (`docs/reference/julien-2026-11-04.md:173–175`): `"you're sure about that? check and if so make me a report on what you think the glm issue is"` — no row.
  15. **S73, S74** (`docs/reference/julien-2026-11-10.md:56–62`), **S7** (`julien-2026-11-04.md:31–33`), **S183** (`julien-2026-11-10.md:651–653`) — procedural turns with no row.
  16. **Conversational turns in the register with no verbatim entry in `docs/reference/`:** `docs/PRINCIPLES.md:108–112` and `215` cite `"6 November"`, `"7 November"`, and `"6 November chat"`, which do not exist in any `docs/reference/julien-*.md` file.
- **Smallest fix:** Add the missing register rows for S31, X7, S135, S176 (merged with S175 as `S175–S176`), S180, S184–S187, and S192–S196 in `docs/PRINCIPLES.md`.

---

### F8. `docs/PRINCIPLES.md` Register statuses that contradict current docs or code
- **Doc and line:** `docs/PRINCIPLES.md:3, 108–112, 172, 211, 215, 216–219, 231, 243, 249, 259–264, 273, 280, 288, 291–292, 295–296, 301–303, 305, 312, 315–316, 331, 367`
- **Specific false or stale statuses:**
  1. **`docs/PRINCIPLES.md:291` (S120):**
     - **Says:** `**Talking it through** (not yet built): checked that food stays 20/20.0 over the 800-block live run`
     - **Why it's an issue:** `docs/reference/postmortem-2026-11-16.md:12, 24, 30` specifically called out that leaving S120's status as `"Talking it through"` caused the 16 November fork, and stated the rule: `A register status must match the code. "Talking it through" and "Open" mean nothing built.` S120 is now folded into `docs/learning.md` section 0, `docs/vocabulary.md` section 5, and built in `wt/u1-astra` (`docs/plan.md:45`), yet line 291 still says `**Talking it through** (not yet built)`!
  2. **`docs/PRINCIPLES.md:301–303, 305, 312, 315–316` (S130, S131, S132, S134, X8, X14, X15):**
     - **Says:** S130, S131, S132, S134 are all still marked `**Open:** talked through first; nothing built until he says so` / `the design write-up waits for his go`, and X8, X14, X15 point to `zones-design`, the dropped fork (`postmortem-2026-11-16.md:14`).
     - **Why it's an issue:** All of S130–S134 were unified in `jobs/unified-plan/plan.md`, approved by Julien in S165, folded into `docs/learning.md` section 0, `docs/representation.md` section 10, and `docs/scales.md`, and built in steps 1–2 (`docs/plan.md:45–47`).
  3. **`docs/PRINCIPLES.md:367` (S208–S209):**
     - **Says:** `Astra always fed, Terra backfills, Hydrogen preps Astra's specs, Argon reads. Awaiting his go`
     - **Why it's an issue:** In the very next turns (S210 at line 368 and S211 at line 369), Julien explicitly forbade `"Hydrogen preps Astra's specs"` (`"no a smaller model don't write anything for a bigger one, never ever"`) and overruled `"Argon reads"` (`"argon can build yes"`). Leaving `"Hydrogen preps Astra's specs, Argon reads. Awaiting his go"` in line 367 is a trap.
  4. **`docs/PRINCIPLES.md:273` (S98):**
     - **Says:** `Not yet written into learning.md.`
     - **Why it's an issue:** It *is* written into `docs/learning.md:80` (section 2, step 9: `Play is a source of queue items, with skin in the game (S97, S98: "how childs get better and test themselves instead of doing nothing"; it "could also create its own puzzles")`).
  5. **`docs/PRINCIPLES.md:259–264` (S84–S91):**
     - **Says:** `**Draft:** learning.md ...` on all six rows.
     - **Why it's an issue:** `docs/learning.md` is no longer a draft; it was approved three times (S93, S165, S169; `learning.md:3`).
  6. **`docs/PRINCIPLES.md:331` (S165):**
     - **Says:** `the order of work U0–U9 in [HANDOFF](HANDOFF.md)`
     - **Why it's an issue:** The order of work was moved to `docs/plan.md` (steps 0–9, in plain names) after S179; `HANDOFF.md` no longer has U0–U9.
  7. **`docs/PRINCIPLES.md:288, 292, 295, 296, 323` (S117, S121, S124, S125, S142):**
     - **Says:** Point to the dropped fork specs `3.2d-3` (`jobs/3.2d-3-design/addendum.md`, `wt/3.2d-3`) and `wt/plain-path` (and line 296 says `Hydrogen up to three lanes beside it` where `AGENTS.md:37` and `clan.md:120` say two).
  8. **`docs/PRINCIPLES.md:280` (S107):**
     - **Says:** `Lanes launch at Idle` — superseded by S158 (`PRINCIPLES.md:337`: `jobs at Normal priority; lower to Idle before a live gate`).
  9. **`docs/PRINCIPLES.md:108–112` (6–7 November):**
     - **Says:** `**Tracked:** Codex integrator owns the three targets in HANDOFF`, `job 1.8b`, `job 1.8c` — none of these exists in `docs/HANDOFF.md`, and Claude/Fable is the integrator (S46, S63).
  10. **`docs/PRINCIPLES.md:215` (6 November chat) and `252` (S75–S77):**
      - **Says:** Link to `HANDOFF.md#tracked-requests-agentsmd-a-request-made-twice-gets-an-owner`, a broken anchor (the heading in `HANDOFF.md:71` is `## Tracked requests (a request made twice gets an owner)`), and line 215 says `Bot server design and the particle-system analogy are open`, when both are adopted in `docs/server.md`, `docs/representation.md:140`, and `docs/scales.md:102`.
  11. **`docs/PRINCIPLES.md:172` (S13), `211` (S40), `216, 218, 219` (S44, S46), `231` (S55), `243` (S66), `249` (S70):**
      - Line 172 says `Open: whether a tool use that leaves no trace (a water bucket placed and taken back) counts as a modification` (settled by S32); line 211 cites nonexistent `claim 15` and `step 2.0, awaiting him` (done 9 November); lines 216–219 say `docs to be rewritten` / `once rewritten` (done 9 November); lines 231 and 243 cite lines removed from `AGENTS.md`; line 249 cites a nonexistent `"Models"` section in `HANDOFF.md`.
  12. **`docs/PRINCIPLES.md:3`:**
      - **Says:** `Keys such as (S6) and (X2) point to [his turns, verbatim](reference/julien-2026-11-04.md).`
      - **Fix:** Name all three files: `[4 November](reference/julien-2026-11-04.md) (S1–S43, X1–X6), [9 November](reference/julien-2026-11-09.md) (S44–S63) and [10–17 November](reference/julien-2026-11-10.md) (S64–S211, X7–X16)`.

---

### F9. `docs/PRINCIPLES.md` and `docs/plan.md` misquote or reword his turns inside quotation marks
- **Doc and line:**
  - `docs/PRINCIPLES.md:368` (S210)
  - `docs/PRINCIPLES.md:234` (S59)
  - `docs/PRINCIPLES.md:195` (S18)
  - `docs/plan.md:81` (S190)
- **What it says vs his verbatim words:**
  1. **`docs/PRINCIPLES.md:368` (S210):**
     - **Says:** `"Hydrogen also can't run code … what?"`
     - **His words** (`docs/reference/julien-2026-11-10.md:781`): `"Hydrogen also can’t run code: its sandbox refused every command in the builds. So it can’t build or test." what?`
     - **Why it's an issue:** `"Hydrogen also can’t run code..."` was the *integrator's* false statement that Julien was quoting in disbelief; his word was `"what?"`. Merging them into `"Hydrogen also can't run code … what?"` attributes the integrator's false claim to Julien.
     - **Smallest fix:** Replace `"Hydrogen also can't run code … what?"` with `(quoting "Hydrogen also can’t run code…") "what?"`.
  2. **`docs/PRINCIPLES.md:234` (S59):**
     - **Says:** `"ok all of these are good besides archive, don't delete"`
     - **His words** (`docs/reference/julien-2026-11-09.md:176`): `ok all of these are good besides "archive, don’t delete, with a ledger line (our cleanup lost the GLM runs);", proceed`
     - **Smallest fix:** Quote verbatim: ``ok all of these are good besides "archive, don’t delete, with a ledger line (our cleanup lost the GLM runs);", proceed``.
  3. **`docs/PRINCIPLES.md:195` (S18):**
     - **Says:** `"because it's just jumping between pillars and guiding it"`
     - **His words** (`docs/reference/julien-2026-11-04.md:79`): `"becaus it's just jumping between pillars and guiding it"`
     - **Smallest fix:** Keep his spelling `"becaus"` inside quotes (per `julien-2026-11-04.md:3`: `verbatim, typos kept`).
  4. **`docs/plan.md:81` (S190):**
     - **Says:** `"a very powerful and cheap tool which is an alien like behavior … extremely suited to a swarm."`
     - **His words** (`docs/reference/julien-2026-11-10.md:695`): `"very powerful and cheap tool which is an alien like behavior that i love and extremely suited to a swarm."`
     - **Smallest fix:** Remove the added `"a "` inside the quote.

---

### F10. Made-up code names (`U1`–`U9`, `P1`–`P4`) used bare without plain names, violating S179
- **Doc and line:**
  - `docs/HANDOFF.md:57, 85, 89`
  - `docs/scales.md:40–42`
  - `docs/learning.md:52, 151`
  - `docs/representation.md:179`
  - `docs/PRINCIPLES.md:329, 331`
- **What it says:**
  - `docs/HANDOFF.md:57`: `(sets backward, the point forward, each worldtube checked once; steps U1–U4 below), then the playback (3.3c) and the executor.`
  - `docs/HANDOFF.md:85`: `U9: threads first, the GPU for learning's batches after a measured round trip (scales.md 4.7)`
  - `docs/HANDOFF.md:89`: `U7: cones on the timing wheel`
  - `docs/scales.md:40`: `U2, U3; 4.2`
  - `docs/scales.md:41`: `U1; 4.3, 4.4`
  - `docs/scales.md:42`: `U6; 4.3, 4.6`
  - `docs/learning.md:52`: `it is a one-way door, red-teamed before it is built (U2).`
  - `docs/learning.md:151`: `Frazzoli's paper is to be read in full before U1 freezes.`
  - `docs/representation.md:179`: `Inside a piece the exit is a monotone map of the entry: P2's map in free motion;`
  - `docs/PRINCIPLES.md:329`: `P1–P4 are the first four`
  - `docs/PRINCIPLES.md:331`: `the order of work U0–U9 in [HANDOFF](HANDOFF.md)`
- **His words:**
  - **S179** (`docs/reference/julien-2026-11-10.md:633`): `"what's up with those made u1 p3 and so on that means nothing say names i'm over those made up names just say what they are. you make up those things and anywhere i look in the doc it's not even explained i never know where the fuck the plan is have a plan.md in the doc ffs."`
- **Why it's an issue:** `docs/plan.md:3` states the rule: `Plain names only: a code (a branch, a job folder) appears in backticks after its plain name, never alone (S179).` All of the occurrences above still use bare `U1`–`U9` and `P1`–`P4` (and `HANDOFF.md:57`'s `"steps U1–U4 below"` is a dangling reference because the table was moved to `docs/plan.md` and renamed steps 1–4).
- **Smallest fix:**
  - `docs/HANDOFF.md:57`: replace `steps U1–U4 below` with `steps 1–4 in [plan.md](plan.md)`.
  - `docs/HANDOFF.md:85`: replace `U9:` with `step 9 (the whole machine, [plan.md](plan.md)):`.
  - `docs/HANDOFF.md:89`: replace `U7:` with `step 7 (other players, [plan.md](plan.md)):`.
  - `docs/scales.md:40–42`: replace `U2, U3`, `U1`, `U6` with `plan.md steps 2–3 (joints and sets, the loop on tracks)`, `plan.md step 1 (the core)`, `plan.md step 6 (the far map)`. Also in `docs/scales.md:3`, replace `The order of work lives in [HANDOFF](HANDOFF.md)` with `The order of work lives in [plan.md](plan.md)`.
  - `docs/learning.md:52`: replace `(U2)` with `(step 2, joints and sets, `jobs/u2`)`.
  - `docs/learning.md:151`: replace `U1` with `step 1 (the core, `jobs/u1`)`.
  - `docs/representation.md:179`: replace `P2's map` with `the exact integer map`.

---

### F11. `docs/HANDOFF.md` "Start here" and metadata are stale against `docs/plan.md` and S181–S211
- **Doc and line:**
  - `docs/HANDOFF.md:8`
  - `docs/HANDOFF.md:16–23`
  - `docs/reference/julien-2026-11-10.md:1`
- **What it says:**
  - `docs/HANDOFF.md:8`: `2. **His words:** [reference/julien-2026-11-10.md](reference/julien-2026-11-10.md) (S64–S105, X7–X16; S103 was never recorded verbatim).`
  - `docs/reference/julien-2026-11-10.md:1`: `# Julien's words, 10 to 13 November 2026`
  - `docs/HANDOFF.md:16–22`:
    - `## Start here (16 November, evening)`
    - `- **Running:** Astra builds the core (step 1, jobs/u1, spec jobs/u1/spec.md). Hydrogen: the plain-names audit (jobs/names-audit) and his tracks as the first flight's test set (jobs/flight-corpus). **Next:** Opus red-teams joints and sets (step 2) after the native window resets; the integrator then writes that step's spec.`
    - `- **His world:** eight maps imported as dimensions (/map), 51 tracks raced with recorded inputs.`
- **His words:** **S163–S164** (`docs/reference/julien-2026-11-10.md:563, 567`): `"did you forgot to update the handoff or the plan at some point?"` / `"Every new context started from HANDOFF's "start here" and a summary, not from the docs."`
- **Why it's an issue:**
  1. `docs/reference/julien-2026-11-10.md` runs through **S211 (17 November)**, not S105 (13 November). Telling a new session in `HANDOFF.md:8` that the file ends at S105 hides S106–S211.
  2. `HANDOFF.md:16–23` is frozen at 16 November evening (S180), contradicting `docs/plan.md:45–47, 61` where step 1 (`u1`) and step 2 (`u2`) are already built, Opus already red-teamed and re-derived twice (S188, S203), Astra is on step 2b (`jobs/u2b`), the four unused maps were removed and seven new maps were imported (S193, S194), and S210–S211 rules landed.
- **Smallest fix:**
  - In `docs/HANDOFF.md:8`, replace `(S64–S105, X7–X16; S103 was never recorded verbatim)` with `(S64–S211, X7–X16; S103 was never recorded verbatim)`.
  - In `docs/reference/julien-2026-11-10.md:1`, replace `# Julien's words, 10 to 13 November 2026` with `# Julien's words, 10 to 17 November 2026`.
  - Update `docs/HANDOFF.md:16–23` ("Start here") to match `docs/plan.md:45–47, 61` and `docs/PRINCIPLES.md:362–369`.

---

### F12. `docs/scales.md`, `docs/vocabulary.md`, `docs/engine.md`, and `docs/learning.md` still describe sets as "plateaus" / "sorted array of breakpoints" after S188 and S203
- **Doc and line:**
  - `docs/scales.md:15` and `docs/scales.md:127`
  - `docs/vocabulary.md:20` and `docs/vocabulary.md:102`
  - `docs/engine.md:60`
  - `docs/learning.md:52` and `docs/learning.md:93–94`
  - `docs/plan.md:46`
- **What it says:**
  - `docs/scales.md:127`: `A set is a sorted array of breakpoints, found by a 16-lane compare and a popcount with no branch.`
  - `docs/scales.md:15`: `| Joint | exact T per plateau, width, the slide signal | which plateau is wanted |`
  - `docs/vocabulary.md:20`: `Otherwise a set's plateau (representation.md section 10) says which certified piece the arriving state takes.`
  - `docs/vocabulary.md:102`: `Two states merge only inside one plateau, where the law's comparisons give them the same future.`
  - `docs/engine.md:60`: `"this crossing takes k ticks from these entry plateaus, within width w"`
  - `docs/learning.md:52`: `The set's record (plateaus between single-ulp breakpoints, premises, prices) is in representation.md section 10`
- **His words:**
  - **S185–S188** (`docs/reference/julien-2026-11-10.md:661–675`): `"Opus re-derives it"` ... `"Yes, adopt it"`
  - **S203** (`docs/reference/julien-2026-11-10.md:751`): `"sure looks good"` (adopting `jobs/pin-rederive/proposal.md`)
- **Why it's an issue:** As `docs/learning.md:127` and `docs/representation.md:179` record, "the old object, plateaus cut on every event, grew under refinement" and was killed on 17 November, then re-derived by Opus and adopted in S188 and S203 as **families** (cut into **pieces** at face crossings and contact shells), **points** (clamped exits), and **holds** (within 1e-7 of a face). `docs/scales.md:15, 127`, `docs/vocabulary.md:20, 102`, `docs/engine.md:60`, and `docs/learning.md:52` still describe the killed "plateaus / sorted array of breakpoints" representation.
- **Smallest fix:** Replace `plateaus` / `sorted array of breakpoints` in `docs/scales.md:15, 127`, `docs/vocabulary.md:20, 102`, `docs/engine.md:60`, and `docs/learning.md:52` with `families, pieces, points and holds (representation.md section 10)`.

---

### F13. `AGENTS.md` and `docs/clan.md` keep the S113-rebuked window formula and contradict each other on Astra's GPT slot limit
- **Doc and line:**
  - `AGENTS.md:33` and `AGENTS.md:37`
  - `docs/HANDOFF.md:14`
  - `docs/clan.md:118`
- **What it says:**
  - `AGENTS.md:33`: `Measured capacities and the usage protocol: [docs/clan.md](docs/clan.md) section 4 (14 November: Codex ~12 Terra runs + 1 Astra per 5 h; GLM ~10–12 lanes per 5 h; Antigravity Claude weekly).`
  - `AGENTS.md:37`: `plus up to four GPT Terra jobs, four at once only as fresh first prompts (S99; a resumed job runs with fewer), and only one GPT job while Astra runs`
  - `docs/HANDOFF.md:14`: `four GPT (two with Astra)`
  - `docs/clan.md:118`: `≤ 4 GPT, ≤ 2 with Astra`
- **His words:**
  - **S113** (`docs/reference/julien-2026-11-10.md:316`): `"also you are not thinking properly for usage you should not think that "~12 Terra runs plus 1 Astra design per window" but what each model uses roughly so that you know based on needs how you can use them properly so based of needs is important and it's rough estimations either don't think this is a hard science just to anchor yourself"`
  - **S15** (`docs/reference/julien-2026-11-04.md:65`): `"btw gpt sub-agents can be two max at the same time indenpedently of you and one if astra is being used"`
- **Why it's an issue:**
  1. `AGENTS.md:33` still contains the exact `"~12 Terra runs + 1 Astra per 5 h"` formula Julien rebuked in S113 (and superseded again in S209: `"the goal is to maximize astra more ... terra is more there to backfill those odd 20% left"`).
  2. `AGENTS.md:37` says `"only one GPT job while Astra runs"` (following S15: `"one if astra is being used"`), whereas `docs/HANDOFF.md:14` and `docs/clan.md:118` say `"two with Astra"` (`≤ 2 with Astra`).
- **Smallest fix:**
  - In `AGENTS.md:33`, replace `(14 November: Codex ~12 Terra runs + 1 Astra per 5 h; GLM ~10–12 lanes per 5 h; Antigravity Claude weekly). Rank GPT jobs by what they unblock.` with `(S113, S209: route by what the task needs and rough cost per run; Astra owns the Codex window and Terra backfills the leftover ~20%).`
  - Reconcile `HANDOFF.md:14` and `clan.md:118` with `AGENTS.md:37` and S15 (`1 GPT job while Astra runs`, or explicitly clarify whether "two with Astra" meant "Astra + 1 Terra" vs "Astra alone").

---

### F14. `docs/ladder.md` conflicts with S124 (end-to-end lag), S170 (clicked face), S189 (repeated retargets & neo-jump practice tracks), and S18 citation
- **Doc and line:**
  - `docs/ladder.md:7, 16, 21, 25, 87–108`
- **What it says:**
  - Line 7: `so for a trip T ends when the bot stands in the goal cell with everything restored.`
  - Line 16: `T runs from the tick the body leaves A to the tick it touches B, measured by the same detector for him and for the bot. The bot's latency from command to first move is reported separately.`
  - Line 21: `- **Latency:** ticks from the whisper's arrival at the bot's client to its first applied control. The p99 must be ≤ 1.`
  - Line 25: `- **Swarm:** the planner will serve hundreds of bots (S17)...`
  - Line 95: `| R-retarget | 2 | A new goal at a random tick mid-trip. T counts from the new command; momentum carries over | to freeze before Phase 2 runs |`
  - Lines 87–108: Rungs table uses the superseded 4 November 12-phase numbers (`Phase 2`, `3`, `4`, `5`, `6`, `7`, `8`, `10`, `11`) and marks `L0` as `to freeze before 1.5 runs`.
- **His words:**
  - **S124** (`docs/reference/julien-2026-11-10.md:376`): `"no lag is part of the execution let's no dismiss it we're doing real end to end, this is what matters in reality and we're not already strating to fake it with fake data"`
  - **S170** (`docs/reference/julien-2026-11-10.md:593`): `"modify the mod so that the face that is selected with the stick is the one that is targetted instead of all faces of the block and make it clear in the feedback otherwise it'll create issues of "cheating""`
  - **S189** (`docs/reference/julien-2026-11-10.md:687–689`): `"oh btw the practice tracks are neo jumps so you really need to match my T or faster since there's a floor underneath and you could get close without making a neo jump, not all my scores are perfect though as those are hard."` / `"we'll need not to forget to have a specific test where we keep changing the goal again and again."`
  - **S18** (`docs/reference/julien-2026-11-04.md:79`): `"will be used by hundreds of bots that's why i bother so much with performances"` (cited as S17 in `ladder.md:25`, whereas S17 is the brachistochrone turn).
- **Why it's an issue:**
  1. `ladder.md:16, 21` separates command-to-first-move latency from track T and starts the latency clock at `"the whisper's arrival at the bot's client"`, whereas S124 ruled that end-to-end lag from the server tick of the click (including transport and first answer) is part of execution and must never be dismissed (`PRINCIPLES.md:295`, `plan.md:7`, `engine.md:111`).
  2. `ladder.md:7, 16` still defines B as `"the goal cell"` / `"touches B"`, missing S170 (`"the face that is selected with the stick is the one that is targetted"`).
  3. `ladder.md:95` (`R-retarget`) tests only one new goal mid-trip, missing S189's `"where we keep changing the goal again and again"` (which is in `plan.md:49` but not `ladder.md`).
  4. `ladder.md:25` cites `(S17)` instead of `(S18)`.
  5. `ladder.md:87–108` still uses the retired 12-phase numbering (`Phase 5`, `Phase 6`, `Phase 7`, `Phase 8`, `Phase 10`, `Phase 11`).
- **Smallest fix:** Update `ladder.md:7, 16, 21` for S124 (end-to-end from the click's server tick, alongside the A-to-B track race clock) and S170 (the targeted face of B); update `ladder.md:25` `(S17)` → `(S18)`; update `ladder.md:95` (`R-retarget`) to include repeated goal changes (S189) and `R5` (line 102) to note the Diversity neo-jump practice tracks over a floor (S189); align phase numbers in `ladder.md:87–108` with `plan.md`.

---

### F15. `docs/engine.md` treats death unconditionally as a barrier ("ruin"), contradicting S189 and `docs/plan.md:76`
- **Doc and line:** `docs/engine.md:70`
- **What it says:**
  - `docs/engine.md:70`: `| ruin is excluded, not priced | an irreversible loss (death, an item lost for good, a world change) is a barrier from the first rung, never a cost in ticks. Tokens (Phase 7) price only what can be spent and replaced |`
- **His words:**
  - **S189** (`docs/reference/julien-2026-11-10.md:683`): `"also we'll need to reconsider what death truly is, there's no real death in this game it's not game over, death is a tool until you impose some constraints for game mechanics like a fight you don't want to die so T=∞ there for death. Death is a form of teleportation that's rather practical and with an ender chest + shulker boxes even a no brainer you could have stations or even another interesting thing, you could have ender pearl stasis chambers."`
- **Why it's an issue:** `docs/plan.md:76` adopted S189 (`"Death is not game over: it is a teleport to the respawn point, with a toll... It is T = ∞ only where a constraint forbids it, as in a fight (S189)"`), but `docs/engine.md:70` still lists `death` unconditionally as an irreversible loss / barrier.
- **Smallest fix:**
  - In `docs/engine.md:70`, replace `(death, an item lost for good, a world change)` with `(an item lost for good, an unrecoverable world change, or death where a constraint such as a fight or unrecovered drops forbids it; otherwise death is a respawn teleport with a toll, S189, plan.md)`.

---

### F16. `docs/learning.md` cites primary sources "from memory and unverified" and leaves Frazzoli unread past its own gate, against S42 and S89
- **Doc and line:** `docs/learning.md:55, 56, 84, 149, 151, 153`
- **What it says:**
  - Line 55: `Recalled, not read: goal regression in classical planning (Fikes and Nilsson 1971); incremental replanning (Koenig and Likhachev, D* Lite).`
  - Line 56: `Recalled, not read: Bellman's cost-to-go; Pontryagin's state and costate; liveness analysis in compilers; causal links in partial-order planning.`
  - Line 149: `For section 2 (16 November): his highlights of Taleb, Waitzkin's The Art of Learning, Isaacson, Ferriss and the Almanack of Naval Ravikant, about 250 through six Readwise searches; the books themselves not opened.`
  - Line 151: `Frazzoli's paper is to be read in full before U1 freezes.`
  - Line 153: `**Not reached, so from memory and unverified:** Sutton, Precup, Singh 1999 (keeping primitives beside options keeps optimality; re-deciding mid-option beats running to the end); Kahneman and Klein 2009; Frazzoli, Dahleh, Feron 2005 (chain when one exit set lies inside the next entry set); March 1991 (exploitation drives out exploration); Simon 1962; the contraction-hierarchy witness rule.`
- **His words:**
  - **S42** (`docs/reference/julien-2026-11-04.md:235`): `"what kind of reasearch is that? wikipedia of all things? research is always from elite primary sources."`
  - **S89** (`docs/reference/julien-2026-11-10.md:147`): `"also you deeply underestimated the library of books to help you think. ... when reference things always go to primary sources."`
  - **S90** (`docs/reference/julien-2026-11-10.md:151`): `"also i did not saw you reference many books or their principles especially something like the art of learning."`
- **Why it's an issue:** Step 1 (`u1`) has already been specced, built, and judged (`docs/plan.md:45`), yet `docs/learning.md:151` still says `Frazzoli's paper is to be read in full before U1 freezes` while listing Frazzoli (and five other cited papers) at line 153 under `Not reached, so from memory and unverified`.
- **Smallest fix:** Have Argon read the primary sources listed in `docs/learning.md:151, 153` (already Argon's named role in `docs/clan.md:51` and `docs/HANDOFF.md:81`) and replace the `"from memory and unverified"` / `"before U1 freezes"` lines with verified citations.

---

### F17. `docs/clan.md` and `docs/HANDOFF.md` still state that all lanes launch at Idle CPU priority, against S158
- **Doc and line:**
  - `docs/clan.md:96` and `docs/clan.md:139`
  - `docs/HANDOFF.md:28`
- **What it says:**
  - `docs/clan.md:96`: `6. **Lanes run at Idle CPU priority** (launch.ps1 does it), so a live gate shares the machine (S107).`
  - `docs/clan.md:139`: `1. Every launch goes through launch.ps1, detached, at Idle;`
  - `docs/HANDOFF.md:28`: `Every lane now launches at Idle.`
- **His words:**
  - **S158** (`docs/reference/julien-2026-11-10.md:543`): `"no need for low priority"` (recorded in `docs/PRINCIPLES.md:337` as `**Followed:** jobs at Normal priority; lower to Idle before a live gate (S106)`).
- **Why it's an issue:** `docs/clan.md:96, 139` and `docs/HANDOFF.md:28` contradict `docs/PRINCIPLES.md:337` and S158.
- **Smallest fix:** In `docs/clan.md:96, 139` and `docs/HANDOFF.md:28`, replace `at Idle` with `at Normal priority, lowered to Idle only while a live gate or his game runs (S158)`.

---

### F18. Requests Julien made twice (or more) with no row/owner in `docs/HANDOFF.md` Tracked requests
- **Doc and line:** `docs/HANDOFF.md:71–94` (`## Tracked requests (a request made twice gets an owner)`)
- **Rule:** `AGENTS.md:44`: `A request Julien makes twice becomes a tracked item in HANDOFF with an owner.`
- **Missing repeated requests:**
  1. **A smaller model never writes for a bigger one** — **S175–S176** (`docs/reference/julien-2026-11-10.md:617, 621`: `"hydrogen's specs? why would hydrogen write the specs for astra?"` / `"sorry i meant fable"`) and **S210** (`docs/reference/julien-2026-11-10.md:783`: `"no a smaller model don't write anything for a bigger one, never ever. this is already in the doc btw."`).
  2. **When asked to explain, explain only — do not act or build until he says so** — **S8** (`julien-2026-11-04.md:37`: `"i'm not asking you to do it but to think first, let's talk"`), **S132** (`julien-2026-11-10.md:423`: `"Your proposal sounds good just don't start yet since i've said other things"`), **S179 (before)** (`julien-2026-11-10.md:637`: `"ok explain me what's the next steps now when i ask to explain you explain you don't do"`), and **S181** (`julien-2026-11-10.md:645`: `"Explain it first"`).
  3. **Re-read the living docs after compaction and lint the docs against his words** — **S76–S77** (`julien-2026-11-10.md:70, 74`), **S161–S164** (`julien-2026-11-10.md:555–567`: `"ok so compactation should follow by a reread the docs"`), **S198** (`julien-2026-11-10.md:729`), and **S210–S211** (`julien-2026-11-10.md:787, 795`: `"for the linting i meant our own docs should be lintet based on what i said looking for issues from time to time"`).
  4. **Do not disrupt the PC when someone else is playing or using it** — **S126** (`julien-2026-11-10.md:386`), **S135** (`julien-2026-11-10.md:439`: `"btw someone is on the computer so don't start things in the foreground"`), **S140** (`julien-2026-11-10.md:463`), and **S144** (`julien-2026-11-10.md:481`: `"i said that someone else was on the computer i can't run on it"`).
  5. **Role and meter routing: Opus and Astra for the core / maximize Astra, Terra backfills, GLM for the port only, leverage Hydrogen and Argon** — **S51, S70, S81, S123, S125, S144, S173, S174, S179, S184, S208–S211** (`HANDOFF.md:82` only cites `S100, S104`).
- **Smallest fix:** Add rows (or expand lines 79 and 82) in `docs/HANDOFF.md:73–94` with `integrator` as owner and pointers to `AGENTS.md` and `docs/clan.md` 5c.

---

### F19. Contradictions between `docs/server.md` and `docs/representation.md` / `docs/plan.md` on hosting and phase numbers
- **Doc and line:**
  - `docs/server.md:1, 61–68, 114` vs `docs/representation.md:139` and `docs/HANDOFF.md:60`
- **What it says:**
  - `docs/representation.md:139`: `- **One engine** drives every bot from inside the server.`
  - `docs/server.md:61–63`: `## Hosting the engine: a separate process` / `**The engine runs as its own process,** over shared memory, pipelined one tick ahead (the engine is anytime).`
  - `docs/server.md:1`: `# One mind in the server (Phase 2 spec, draft)` and `docs/server.md:114`: `The chunk-ticking wall for lone bots (Chunks, above) is kept in mind for Phase 5, not solved now.`
- **Why it's an issue:**
  1. `representation.md:139` says `"from inside the server"` while `server.md:61–68` specifies a **separate process** over shared memory (and rejected in-process hosting on the server).
  2. `server.md:1` still calls itself `Phase 2 spec`, whereas in `HANDOFF.md:60` it is Roadmap item 5 (`Slice C, one mind in the server`).
- **Smallest fix:**
  - In `docs/representation.md:139`, replace `from inside the server` with `through the server mod ([server.md](server.md))`.
  - In `docs/server.md:1`, replace `(Phase 2 spec, draft)` with `(Slice C / Roadmap 5 spec, draft)`.

---

## 2. Findings we are unsure of

1. **`docs/PRINCIPLES.md` line count (381 lines) vs `AGENTS.md:58` budget (`each spec in docs/ ≤ 200 lines`):**
   - **Unsure because:** `AGENTS.md:58` says `Budgets: HANDOFF ≤ 120 lines, each spec in docs/ ≤ 200 lines, this file ≤ 60 lines.` All other 11 files in `docs/` are ≤ 200 lines (`footprints.md` is at 200/200, `representation.md` is at 199/200). `docs/PRINCIPLES.md` is **381 lines** because it holds the growing Register of S1–S211 (lines 93–370). It is probably exempt as the philosophy/register rather than a "spec", or its register could be split out, but `AGENTS.md:58` does not state the exemption.
2. **Paraphrased (unquoted) cells in the "Statement" column of `docs/PRINCIPLES.md` (`lines 100–212, 243, 273, 278, 293, 301, 355`):**
   - **Unsure because:** `docs/PRINCIPLES.md:3` says `"The labels are ours; the quotes are his"`, and `AGENTS.md:46` says `"His turns go verbatim into docs/reference/, and the register in PRINCIPLES.md cites them; models never reword them."` Many rows in the Register use unquoted topic summaries alongside quoted phrases (to fit long turns like S6 or S44 into a table cell). Unquoted summaries that point to the S-key are likely intentional labels, whereas altered text *inside* quotation marks (flagged in **F9** above) is a definite violation.
3. **`docs/PRINCIPLES.md:200–202` (S200–S202) — complex numbers / disc frame for free motion:**
   - **Unsure because:** In S200 (`julien-2026-11-10.md:739`), Julien asked `"why are we doing a x y for heaving when we could use complex numbers? the player acts as a ball of influence not as a cube and it does so at each step"`, and in S202 (`julien-2026-11-10.md:747`): `"ok now you can pitch opus"`. `docs/vocabulary.md:58` added one sentence on the sine-table norm in the disc frame (`"The table's norm varies with the yaw by about 3×10⁻⁸, so a lower bound in the disc frame uses its largest value"`), and `PRINCIPLES.md:359` marks `**Open:** a disc frame for free motion and bounds, the grid only at contacts`, while `representation.md` does not yet mention the disc frame. If that part is still intentionally open awaiting measurement, `representation.md` is clean on it; if S203 adopted it as part of the representation, `representation.md` section 10 should name it.

---

## 3. What was checked and found clean (one line per doc)

- **`AGENTS.md` (60/60 lines):** Budget exact (60 lines), no mojibake, S164 compaction re-read rule (`line 3`), S46 law definition (`line 12`), S210 smaller-never-writes-for-bigger and Astra cold-cache rules (`lines 38–39`), and ES 19 no-patches rule (`line 43`) are all in place.
- **`docs/PRINCIPLES.md` (381 lines):** Goal and 28 philosophy points (`lines 5–47`) match S1–S37 and X1–X6 verbatim, ES mapping (`lines 49–91`) and Contradictions table (`lines 371–381`) are accurate, and no UTF-8 mojibake exists.
- **`docs/plan.md` (83/200 lines):** Within budget, no mojibake, follows S179 plain-names rule throughout (`line 3`), holds the S161/S165 one-mechanism rule (`line 11`), S131/S166 two extremes (`lines 7–9`), and all S189–S197 additions (`lines 47–49, 61, 74–83`).
- **`docs/HANDOFF.md` (107/120 lines):** Within budget (107 lines), no mojibake, anomalies 12 and 13 from 17 November (`lines 46–47`) and the scoreboard (`lines 67–69`) are current.
- **`docs/learning.md` (153/200 lines):** Within budget, no mojibake, accurately folds S165 (`section 0`, one loop for every terrain), S93 (`section 1`, what stands), S169 (`section 2`, one queue, free compute inside certified words, play with skin in the game), and S188/S203 kill test 11 (`line 127`).
- **`docs/representation.md` (199/200 lines):** Within budget, no mojibake, accurately holds the S46 three grains (`section 1`), S59 programs plus scars (`section 2`), S32/S46 mission restoration and prices (`section 8`), S165 crystal with defects (`section 10`), and the S182/S188/S203 set record of families, pieces, points, and holds (`line 179`).
- **`docs/scales.md` (166/200 lines):** Within budget, no mojibake, accurately holds S64–S65 one rule at every scale (`section 2`), S65 presence as a tool (`section 4.2`), S84/S166 thin-bridge persistence rule (`section 4.3`), and S124/S166 micro latency and GPU-for-learning-batches rules (`section 4.7`).
- **`docs/vocabulary.md` (121/200 lines):** Within budget, no mojibake, accurately records Version 3 changes (`lines 6, 20, 43, 57–58, 121`: no planner decision at body-block change, `count` as an argument of every word, worldtube checked once, and S202 aim carries no memory).
- **`docs/engine.md` (186/200 lines):** Within budget, no mojibake, accurately holds S46 law definition (`line 15`), SR-71 sealing rule (`line 17`), S118/S148 event-to-event closed forms certified against the ticked law (`line 90`), S165 word-plan output contract (`lines 102–107`), and S166 GPU batch rule (`line 121`).
- **`docs/clan.md` (177/200 lines):** Within budget, no mojibake, section 5c (`lines 136–148`) accurately records the S128 post-mortem rules, S168 single-step usage check (`line 143`), S209 Astra window rule (`line 142`), and S210–S211 doc-linting rule (`line 148`).
- **`docs/ladder.md` (119/200 lines):** Within budget, no mojibake, accurately holds the ES Spartan testing method (`lines 40–79`), S62 server corrections as veto rate (`line 89`), S32/S122 Rediscovery techniques (`lines 98, 109–119`), S17 R-brach (`line 100`), and S65 R-lego (`line 99`).
- **`docs/server.md` (115/200 lines):** Within budget, no mojibake, accurately holds S46/S52 `"bots are players"` rulings (`lines 76–81`), S53 10–50 bots flight (`line 114`), and S65 worldtube tickets and presence (`lines 104–110`).
- **`docs/footprints.md` (200/200 lines):** Within budget (exact 200 lines), no mojibake, self-consistent technical specification of `RecordingWorld`, determinism inputs, and `PatternTable` digests with no drift against his words.
- **`lab/glm.md` (50/200 lines):** Within budget, no mojibake, accurately holds S20's machine-checkable rule (`lines 5–11`), S20's one-click encrypted key entry (`line 44`), and S59's `lab/wiki/` memory (`lines 46–50`).
