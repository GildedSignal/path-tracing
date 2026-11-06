# Lens: what observations make player resources replayable?

The dossier must start with a one-sentence `Claim: ` line. The independent checker must reproduce it with a machine check.

Minecraft 26.2 is the law. The Rust client movement replay now matches the live controls on full blocks. Its peaceful end saturation differs while the normal-difficulty repeat matches. A passive lab probe separately confirmed peaceful refills follow the server player's own tick phase. The project has a client recording with applied inputs, game time, local tick count and movement, plus RCON resource snapshots at the ends. It does not have an authoritative ordering of accepted player moves and server resource ticks.

**Question.** Which server observations are necessary and sufficient to replay the peaceful health, food, saturation and exhaustion updates for a living, unmounted survival player with no status effects? Trace the server player counter, the actual order of `tickRegeneration` and `FoodData.tick`, and the accepted movement/jump handlers that add exhaustion. Check whether the existing client clocks and RCON bracket determine that order. Identify a concrete ambiguity if they do not.

**Evidence.** Supply `reproduce.sh` that checks exact class, method and offsets against the 26.2 server jar. Pin its hash. Use the staged local reference jar read-only when present or verify the official server download through Mojang's version manifest and open its bundled server jar. For an ambiguity, supply the smallest executable witness with the same currently observed inputs and differing vanilla-consistent resource outcomes. Report the missing observation and the scope of the witness. A speculative implementation is out of scope.

Keep all work in the lens output. Read the isolated server jar only. Do not start the server, modify either rig, change production code or change the gate.
