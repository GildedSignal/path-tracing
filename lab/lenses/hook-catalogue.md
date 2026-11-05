# Lens: the hook catalogue (jar reader)

**The question.** In Minecraft 26.2 (Java Edition, the client jar), which classes override the hooks through which a block changes a player's movement, damage or exhaustion? The Rust port of the law (`engine/law` on branch `wt/law`, not on `main`) must either model each one or stop with `unimplemented!`, and today it knows only some of them.

**Getting the jar** (it is never committed to this public repository, nor any decompiled code):
- the version manifest `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json` gives 26.2's version file, which gives `downloads.client.url` and its `sha1`; download it and check the hash;
- the jar is unobfuscated, so read it with `javap -c -p -constants`. If the runner's JDK rejects the class-file version, install a Temurin JDK recent enough (Adoptium's API) and say which.

**What to produce.**
- `reproduce.sh`: downloads and verifies the jar, then lists, for every class in the jar, which of these methods it declares itself (not inherits): `Block.entityInside`, `Block.fallOn`, `Block.stepOn`, `Block.updateEntityMovementAfterFallOn`, `Block.getFriction`, `Block.getSpeedFactor`, `Block.getJumpFactor`, and their `BlockBehaviour` and `FluidState`/`Fluid` counterparts where they exist. Also every class passing a non-default `friction`, `speedFactor` or `jumpFactor` to `BlockBehaviour.Properties` in a static initialiser (`Blocks.<clinit>`), with the value.
- `catalogue.tsv`: one line per (class, method) with the bytecode evidence: the method signature, and for each constant it uses, its value and offset.
- `dossier.md`: the claim (one sentence: how many overriders exist per hook), and how the list was made complete (why a class can't be missed).

**Out of scope:** what each hook does in detail. That is a later lens. Don't paste method bodies; cite offsets.
