# GLM

GLM 5.4 (Z.ai, through omp; it spends less of the 5-hour window than 5.3) extracts **parameter table rows, Mojang scars, and `server-26.2.jar` bit-witnesses** against our generator basis (`jobs/<id>/prompt.md` -> `lab/results/<run>/` `dossier.md`, `reproduce.sh`, `work.patch`), plus short compiled `lab/wiki/` notes. Broad class-by-class Rust file porting is frozen, and there is no second checker session (S234, S235): results are judged by jar witnesses, the integrator, and live flights.

- **Launch:** GLM never idles while it has queued work (S241): `scripts\glm-chain.ps1` drains `jobs\glm-chain.txt` using a rolling `-Parallel` worker pool, honoring Z.ai 5-hour cooldowns (`retry-after-ms`) and logging to `jobs\glm-chain.log`. One job by hand: `scripts\glm-local.ps1 -Lenses "<job-id>" -Minutes 60`. Commit the prompt first: lanes run from a checkout of HEAD. Results go to `lab\results\local-*`, which git ignores.
- **Its limit** is Zai's 5-hour token window (`scripts\meters.ps1`).
- **The key** is stored by `scripts\set-glm-key.ps1`, encrypted for Julien's account; it only enters an omp process's environment, and runners delete any output file that contains it.
- **It is weak** (S173) and reads untrusted bytecode: its patches are read by the integrator before they are merged.
