# GLM

GLM 5.3 (Z.ai, through omp) writes port code against Mojang's jar: one prompt the integrator writes (`jobs/<id>/prompt.md`), one author session, results in `lab/results/<run>/` with its `dossier.md`, `reproduce.sh` and `work.patch`. There is no checker session, no lens library and no wiki (S234, S235): the work is judged in the game and by the integrator, and a delta is fixed when seen.

- **Launch:** `scripts\glm-local.ps1 -Lenses "<job-id>" -Minutes 60`. Commit the prompt first: lanes run from a checkout of HEAD. Results go to `lab\results\local-*`, which git ignores.
- **Its limit** is Zai's 5-hour token window (`scripts\meters.ps1`).
- **The key** is stored by `scripts\set-glm-key.ps1`, encrypted for Julien's account; it only enters an omp process's environment, and runners delete any output file that contains it.
- **It is weak** (S173) and reads untrusted bytecode: its patches are read by the integrator before they are merged.
