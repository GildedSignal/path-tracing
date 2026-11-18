# Strip history out of map/detail/: present tense only, nothing new

**Julien (S236, verbatim):** "there's a lot of "this was the case before" just remove that, too much details where it doesn't matter … We should not have stale things in the doc, history is stored elsewhere."

**Where:** your worktree `D:\path-tracing-wt\detail-strip` (branch `wt/detail-strip`, from `main`). Edit only the files in `map\detail\` there: `clan.md`, `engine.md`, `footprints.md`, `ladder.md`, `learning.md`, `plan.md`, `representation.md`, `scales.md`, `server.md`, `vocabulary.md`. The top level `map\*.md` is the truth they must agree with; read it first, do not edit it.

**What to delete** (this is deletion, never rewriting the design):
- history: dates ("on 14 November", "since 13 November"), "was", "used to", "superseded", "previously", "no longer", "version 1 did …", "the old …", "before job X", records of who built what and when, measurements labelled with a date when a newer one exists;
- bookkeeping: job ids and run folders cited only as history (keep a job path only where it is the place to find a present fact);
- whatever `map\*.md` already says: replace it with a link to the `map\` file;
- in `clan.md`: the vendor benchmarks and the post-mortem narratives (keep the routing in one table);
- in `plan.md`: everything `map\now.md` already holds (the order of work and its state).

**What to keep, untouched in meaning:** every present design claim, every number that is current, his quoted words with their turn (S-number), every kill test and gate definition, every link that resolves.

**Gate:** in each file no line matches `[0-9]{1,2} (October|November|December)|superseded|previously|no longer|used to be`; every file is shorter; `git diff` shows only deletions, and rewordings where a sentence had to be put in the present tense. Run `bash scripts/git-hooks/pre-commit` from the worktree after `git add map` to check the links.

**Report** (`report.md`, written as you go): per file, bytes before and after, and a list of what kind of thing you removed. Anything you were unsure about: say it, with the line, and leave it in.
