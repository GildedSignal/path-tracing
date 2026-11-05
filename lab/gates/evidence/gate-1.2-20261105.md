# Gate 1.2

UTC: 20261105T0838027621258Z
Rig: C:\Users\PASCAL\Desktop\path tracing
Start: 0.5,-60,0.5,0,0 (x,y,z,yaw,pitch); survival on lab superflat.
Runs: 8; ticks per replay: 400; seeds: Seed + i, i=1..Runs; Seed=1.
Replay timeout: 100 seconds (60 + ceiling(Ticks / 10), capped at 2147400); exit: 0.

Assumptions: prepared flat course at y=-60, no nearby players; common.ps1 supplies ports and names.
Usage ping is the unknown recorder subcommand rec gate-ping; unknown top-level words are silent.
Uses the frozen comparison helper (ignores absolute tickCount). No health/food/inventory reset.

Gate 1: PASS - 8 of 8 replay pairs identical. Evidence: [gate-1.txt](gate-1.txt).
Gate 2: PASS - 16 recordings passed with no anomalies or structural errors. Evidence: [gate-2.txt](gate-2.txt).
Gate 3: PASS - Exit 0: Dump OK: state lines 32366, written 32366, registry 32366; movement constants match. Evidence: [0097-check-dump.log](0097-check-dump.log).
Gate 4: PASS - Checker rejected only named anomalies and caught the injected teleport after clean baseline ticks. Evidence: [0106-check-recording.log](0106-check-recording.log).

Driver and cleanup:

- Rig: C:\Users\PASCAL\Desktop\path tracing; server 127.0.0.1:25565; RCON 25575; bot PathBot; owner Brain_Forge.
- Start: 0.5,-60,0.5,0,0 (x,y,z,yaw,pitch), lab superflat; survival; Runs=8; Ticks=400; Seed=1.
- Bot jar SHA256: FDA502CBFDE73D4C495C90E4729D2AA6E1C7AD0D9FD1363D2E86A80A39C8D4C1.
- Stopped owned bot process 1464.
- Server stop: see 0107-rcon.log.

Command outputs (each log includes the invocation, exit, stdout and stderr):

- [0001-start-server.log](0001-start-server.log)
- [0002-rcon.log](0002-rcon.log)
- [0003-rcon.log](0003-rcon.log)
- [0004-launch-client.log](0004-launch-client.log)
- [0005-rcon.log](0005-rcon.log)
- [0006-rcon.log](0006-rcon.log)
- [0007-rcon.log](0007-rcon.log)
- [0008-rcon.log](0008-rcon.log)
- [0009-rcon.log](0009-rcon.log)
- [0010-rcon.log](0010-rcon.log)
- [0011-rcon.log](0011-rcon.log)
- [0012-rcon.log](0012-rcon.log)
- [0013-rcon.log](0013-rcon.log)
- [0014-rcon.log](0014-rcon.log)
- [0015-rcon.log](0015-rcon.log)
- [0016-rcon.log](0016-rcon.log)
- [0017-rcon.log](0017-rcon.log)
- [0018-rcon.log](0018-rcon.log)
- [0019-rcon.log](0019-rcon.log)
- [0020-rcon.log](0020-rcon.log)
- [0021-rcon.log](0021-rcon.log)
- [0022-rcon.log](0022-rcon.log)
- [0023-rcon.log](0023-rcon.log)
- [0024-rcon.log](0024-rcon.log)
- [0025-bot-command.log](0025-bot-command.log)
- [0026-rcon.log](0026-rcon.log)
- [0027-bot-command.log](0027-bot-command.log)
- [0028-rcon.log](0028-rcon.log)
- [0029-rcon.log](0029-rcon.log)
- [0030-rcon.log](0030-rcon.log)
- [0031-new-inputs.log](0031-new-inputs.log)
- [0032-rcon.log](0032-rcon.log)
- [0033-bot-command.log](0033-bot-command.log)
- [0034-check-recording.log](0034-check-recording.log)
- [0035-rcon.log](0035-rcon.log)
- [0036-bot-command.log](0036-bot-command.log)
- [0037-check-recording.log](0037-check-recording.log)
- [0038-compare-recordings.log](0038-compare-recordings.log)
- [0039-new-inputs.log](0039-new-inputs.log)
- [0040-rcon.log](0040-rcon.log)
- [0041-bot-command.log](0041-bot-command.log)
- [0042-check-recording.log](0042-check-recording.log)
- [0043-rcon.log](0043-rcon.log)
- [0044-bot-command.log](0044-bot-command.log)
- [0045-check-recording.log](0045-check-recording.log)
- [0046-compare-recordings.log](0046-compare-recordings.log)
- [0047-new-inputs.log](0047-new-inputs.log)
- [0048-rcon.log](0048-rcon.log)
- [0049-bot-command.log](0049-bot-command.log)
- [0050-check-recording.log](0050-check-recording.log)
- [0051-rcon.log](0051-rcon.log)
- [0052-bot-command.log](0052-bot-command.log)
- [0053-check-recording.log](0053-check-recording.log)
- [0054-compare-recordings.log](0054-compare-recordings.log)
- [0055-new-inputs.log](0055-new-inputs.log)
- [0056-rcon.log](0056-rcon.log)
- [0057-bot-command.log](0057-bot-command.log)
- [0058-check-recording.log](0058-check-recording.log)
- [0059-rcon.log](0059-rcon.log)
- [0060-bot-command.log](0060-bot-command.log)
- [0061-check-recording.log](0061-check-recording.log)
- [0062-compare-recordings.log](0062-compare-recordings.log)
- [0063-new-inputs.log](0063-new-inputs.log)
- [0064-rcon.log](0064-rcon.log)
- [0065-bot-command.log](0065-bot-command.log)
- [0066-check-recording.log](0066-check-recording.log)
- [0067-rcon.log](0067-rcon.log)
- [0068-bot-command.log](0068-bot-command.log)
- [0069-check-recording.log](0069-check-recording.log)
- [0070-compare-recordings.log](0070-compare-recordings.log)
- [0071-new-inputs.log](0071-new-inputs.log)
- [0072-rcon.log](0072-rcon.log)
- [0073-bot-command.log](0073-bot-command.log)
- [0074-check-recording.log](0074-check-recording.log)
- [0075-rcon.log](0075-rcon.log)
- [0076-bot-command.log](0076-bot-command.log)
- [0077-check-recording.log](0077-check-recording.log)
- [0078-compare-recordings.log](0078-compare-recordings.log)
- [0079-new-inputs.log](0079-new-inputs.log)
- [0080-rcon.log](0080-rcon.log)
- [0081-bot-command.log](0081-bot-command.log)
- [0082-check-recording.log](0082-check-recording.log)
- [0083-rcon.log](0083-rcon.log)
- [0084-bot-command.log](0084-bot-command.log)
- [0085-check-recording.log](0085-check-recording.log)
- [0086-compare-recordings.log](0086-compare-recordings.log)
- [0087-new-inputs.log](0087-new-inputs.log)
- [0088-rcon.log](0088-rcon.log)
- [0089-bot-command.log](0089-bot-command.log)
- [0090-check-recording.log](0090-check-recording.log)
- [0091-rcon.log](0091-rcon.log)
- [0092-bot-command.log](0092-bot-command.log)
- [0093-check-recording.log](0093-check-recording.log)
- [0094-compare-recordings.log](0094-compare-recordings.log)
- [0095-rcon.log](0095-rcon.log)
- [0096-bot-command.log](0096-bot-command.log)
- [0097-check-dump.log](0097-check-dump.log)
- [0098-rcon.log](0098-rcon.log)
- [0099-rcon.log](0099-rcon.log)
- [0100-rcon.log](0100-rcon.log)
- [0101-bot-command.log](0101-bot-command.log)
- [0102-rcon.log](0102-rcon.log)
- [0103-rcon.log](0103-rcon.log)
- [0104-rcon.log](0104-rcon.log)
- [0105-bot-command.log](0105-bot-command.log)
- [0106-check-recording.log](0106-check-recording.log)
- [0107-rcon.log](0107-rcon.log)

Overall: PASS

The full evidence (recordings, dump, every command log; 39 MB) is kept at `D:\path-tracing-evidence‚61105T0838027621258Z-gate-1.2`.
