# His tracks as the first flight's test set

**Consumer:** the first flight of the new algorithm, where one `solve` runs his tracks and is compared with his recorded times (`docs\plan.md`, step 3, "the loop on real tracks"). Also the choice of which tracks gate which step.

You read only, and write only your report. Root `C:\Users\PASCAL\Desktop\path tracing\`.

## Read
- `lab\tracks\*.json`: every track, with its A, B, dimension and, for new tracks, `bFace`. Skip the `*.kit.json` kit files.
- `lab\tracks\runs\*.jsonl`: his runs, with T, the recording path, settings including `sprintWindow`, and `recordingNote`.
- `lab\tracks\tracks.txt` and `engine\world\tests\fixtures\tracks.txt`: the old track list.
- `jobs\map-scout\sonnet.md`: what each imported map contains.
- `docs\learning.md` section 6: the kill tests, so you can say which track feeds which.

## Write `jobs\flight-corpus\report.md`
1. **One row per track:**
   - its name and map (dimension);
   - the straight distance from A to B;
   - B's face (or "any face, before S170");
   - his best T and his number of runs;
   - whether his best run has a recording, and whether the engine can replay it today. It cannot when `sprintWindow` is not 0 (`engine\law\tests\replay.rs:366`) or when there is a `recordingNote`.
   - the mechanics the track needs, judged from the map description and the track's span. Say "unknown" where you cannot tell.
2. **Groups:** sort the tracks into
   - straight or open movement;
   - technical jumps;
   - climbs (Infinity Tower);
   - long courses (parkour_race, frosted_ice);
   - the old 21.
3. **A first-flight set:** the smallest group of tracks that covers every mechanic his runs show, plus the two extremes (the longest simple line and the most technical span). Give the reason for each.
4. **Gaps:**
   - mechanics he raced that our engine does not port (`jobs\port-backlog\report.md`);
   - recordings that cannot be replayed, and why.
