#!/usr/bin/env bash
# SessionStart hook (S232): puts every living doc, in full, into the integrator's context at startup,
# on resume, after /clear and after every compaction. The model no longer chooses whether to read them.
# Its standard output becomes context. The last line is a sentinel: if it is missing, the load was cut
# short, and the integrator must read the files named below before anything else.
cd "$(dirname "$0")/.." || exit 0
docs="AGENTS.md map/goal.md map/principles.md map/architecture.md map/vocabulary.md map/work.md map/now.md"
echo "# The living docs, loaded in full by the SessionStart hook (scripts/load-docs.sh, S232)"
echo "These are the project's binding docs. They are in context below, whole; do not claim to have read anything else."
echo "If the line 'END OF LIVING DOCS' is not at the very end, the load was truncated: read the missing files with the Read tool, every page, before any spec, explanation or launch."
n=0; bytes=0
for f in $docs; do
  [ -f "$f" ] || { echo; echo "## MISSING: $f"; continue; }
  echo; echo "======== BEGIN $f ========"; cat "$f"; echo; echo "======== END $f ========"
  n=$((n+1)); bytes=$((bytes+$(wc -c <"$f")))
done
echo; echo "END OF LIVING DOCS: $n files, $bytes bytes"
