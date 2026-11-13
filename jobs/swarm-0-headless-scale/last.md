The measurement is blocked before the server can be run. The required rig lock is `D:\path-tracing-lab\rig.lock`, but the sandbox does not allow writes to `D:\path-tracing-lab`. Without that lock, I cannot safely use the shared rig or perform the real-client check.

No mod was built and no measurements were taken.

| N | Dense, ms/tick | Dispersed, ms/tick |
|---:|---|---|
| 0 | Not measured | Not measured |
| 1 | Not measured | Not measured |
| 10 | Not measured | Not measured |
| 100 | Not measured | Not measured |
| 1000 | Not measured | Not measured |

Sustained rates, memory, JFR profile shares and the one-bot correction count are also unavailable. The run needs write access to the reference rig, including its lock, before it can proceed.