# 1.9d offline gate

The 20 TPS standing traces are the direct server oracle. Use the unchanged
[`contact-v1` format and pass line](../1.9a-contact-damage-witness/gate.md).
Run the integrator's standing-only verifier independently of the Rust check.
The frozen inputs under `D:/path-tracing-evidence/contact-stock-20261107T154929`
are the two `<case>/server/<case>.tsv` files, with SHA-256 in case order:

- `magma-standing` `C18034D2D42EF56BFF6C43AE1C30BF88B8BF74B8F32AA1F9E2CD82EDFDB21DAE`
- `netherrack-standing` `B529713DAF57B1FB2C8362770D71D9ADCCA2464AB79E7803721EFC4D0136A104`

In each case seed position, health and cooldown from the stone predecessor.
Keep the settled zero-input movement and the authoritative resource timeline
separate. Supply the typed effect on the hot floor in fixture data. For each
of 32 consecutive ticks:

- Advance the cooldown before the HEAD sample. Compare raw health bits and
  integer cooldown at HEAD, grounded dispatch, request entry/return and RETURN,
  wherever those rows exist. Read expectations only from the raw trace.
- Run the movement hook on the settled body. Check the selected support,
  position, ground flag, pose and careful flag against the dispatch row.
  Compare request presence, source and raw amount independently of acceptance.
- Resolve the predicted request once at the ordered dispatch boundary. Compare
  the loss with the reported acceptance boolean and preserve the difference
  between raw requests and accepted hits. Observed damage events remain direct
  resource debits and are separately tested.

Offline branch controls use the server 26.2 bytecode. Grounded dispatch uses
the legacy contact block, preserves the selected support's horizontal column
on an overhang, and honors `onGround` and the shift flag rather than pose.
Equal raw hits are refused strictly above cooldown 10, with health, last raw
hit and cooldown unchanged. At 10 the full amount is applied and the cooldown
resets to 20. A larger eligible raw hit inside the window contributes only
its excess and leaves the cooldown in place. A refused request adds no
exhaustion. One accepted unmodified hot-floor hit adds the source's vanilla
0.1 exhaustion. The raw TSVs do not sample exhaustion or last raw hit.

Run the existing movement and resource tests. In an isolated copy of the
engine, change the cooldown threshold from `> 10` to `> 9` and require the
same standing differential to reject it on magma relative tick 10 at
request return. Preserve the original test and inputs for this run.

The separate live sneaking case and accelerated-clock recheck stay with 1.9a.
