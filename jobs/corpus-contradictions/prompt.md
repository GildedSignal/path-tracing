# Job corpus-contradictions: walk the corpus and say what disagrees with what

**Why.** This project's truth lives in documents that cite each other: `AGENTS.md` points at `docs/`, `docs/HANDOFF.md` points at jobs, jobs point at lab runs, lab runs point at `lab/wiki/`, and the register in `docs/PRINCIPLES.md` cites Julien's turns in `docs/reference/`. Truth is rewritten in place and history lives in git, so when a fact changes, every other place that stated it should have changed too — and sometimes does not. A stale number or a retired rule that is still written down somewhere gets obeyed by the next reader, model or human. Finding those is your job.

**Consumer.** The integrator, before the next wave of jobs. Each contradiction you find is either a doc edit or a rule to re-decide, and a false one costs a real read to dismiss — so precision matters more than volume here.

**Read.** You have the repository at `C:\Users\PASCAL\Desktop\path tracing`. Start with `AGENTS.md`, then `docs/PRINCIPLES.md` (the register of his statements at the end especially), `docs/HANDOFF.md`, `docs/clan.md`, `docs/representation.md`, `docs/scales.md`, `docs/learning.md`, `docs/vocabulary.md`, `docs/engine.md`, `docs/ladder.md`, `docs/footprints.md`, `docs/server.md`. Then the things they point at: `jobs/*/prompt.md` and `jobs/*/last.md`, `lab/glm.md`, `lab/wiki/index.md`, `lab/queue.txt`, and `scripts/*.ps1`. Follow the citations: a doc that cites a run folder or a turn key is making a claim you can check against the thing it cites.

**What counts as a contradiction.** Two places in the corpus that cannot both be true, or a rule that is still written as in force after a later decision retired it. Specifically worth checking:
- a **measured number** stated differently in two places, or a number stated in a doc that the run it cites does not support;
- a **rule in `AGENTS.md`** that a later turn in `docs/reference/` or a later doc has superseded, with the old wording still standing;
- a **script's actual behaviour** disagreeing with the doc that describes it — a default in a `.ps1` file that names something a decision retired, a path or a flag the docs do not match;
- a **status claim**: something described as done, frozen, merged or in flight that another file contradicts;
- a **word** used in two incompatible senses between `docs/vocabulary.md` and the docs that use it.

**What does not count.** A doc being less detailed than another; a plan superseded *and labelled as such* (the register's "Superseded" status is correct bookkeeping, not a contradiction); an open question written down as open; a number with a stated error bar overlapping another; anything in `docs/reference/` that is Julien's own words — his turns are verbatim and may contradict each other freely, because he is allowed to change his mind. A later turn overriding an earlier one is only a contradiction if a **doc or a rule** still states the earlier one as current.

**Return**, and nothing else, at most 60 lines:

1. **The contradictions, ranked by what obeying the wrong side would cost.** One per row: the two file paths (with line numbers where you can), what each says in a few words, and which one you believe is current and why. Where you cannot tell which side is current, say so — that is the honest answer and it is still useful.
2. **Near misses you checked and cleared**, one line each, at most eight. This section is what makes the report trustworthy: it shows what you looked at and did not flag.
3. **Where you could not reach.** Any file you could not read, any citation you could not follow, any claim that needs a run rather than a read to settle.

**The one rule.** Every row cites the files it comes from by path. A contradiction you cannot point at in two places is not a finding — do not report an impression, a style note, a suggestion, or anything you would like improved. There is no credit for length, and a short report with three real findings beats a long one with thirty guesses.
