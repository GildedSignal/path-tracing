# The map against his words

**Goal:** a list of every line in `map/` that contradicts, drops or goes beyond what Julien said, each with his verbatim words and the rewrite.

You read; you change no file. The map (`map/*.md`, and `map/detail/*.md`) is meant to be his beliefs, rewritten in the present tense. His words are the source: `archive/turns/` (verbatim, every turn, `inbox.md` the newest) and `archive/register.md` (his turns indexed by S-number).

## What to look for

1. **Contradiction:** the map says X; a turn of his says not-X. The later turn wins; say which is later.
2. **Invention:** the map states a belief, a number or a rule that no turn of his supports, or names a turn (S-number) that does not say it.
3. **Drop:** a belief he stated more than once, or with force, that the map does not carry. Only beliefs that change what gets built; not every remark.
4. **His vocabulary:** a word in the map that is ours, not his (`map/vocabulary.md` is the list), where his word exists.

## Output

Your final message: a table, most consequential first, with columns: kind (1–4), `file:line`, the map's text (short), his words (verbatim, with the turn), the rewrite in the present tense. Then one line: how many lines of `map/` you checked and how many findings per kind. No finding without a quote of his.
