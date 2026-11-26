# The law on Minecraft 26.3

## Verified changes

| Mojang rule | Where it lands |
|---|---|
| A saved block state is a default-state ID or a compound with `id` and optional `properties`. A mixed NBT palette wraps a shorthand string in a compound under the empty key. Parkour Spiral has both homogeneous string lists and mixed compound lists. | `engine/world/src/region.rs` and `nbt.rs` decode these into the same `BlockState` as the 26.2 reader. The 26.3 level and chunk versions are 5023. [Mojang 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) |
| Mojang resets the player's momentum on ender-pearl teleport. | `engine/law/src/pearl.rs` and its existing callers need the jar's exact velocity and relative-movement rules before changing. [Mojang 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) |
| Spectators can activate Nether portals, End portals and End gateways. | Check `engine/law/src/portal.rs` and `block_hooks.rs` against the jar. The player's mode and the exact interaction path determine which law boundary changes. [Mojang 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) |
| Mojang fixes item drops appearing at the wrong location. | Compare loot spawning and position RNG with `engine/law/src/items.rs` before changing a spawn rule. [Mojang 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) |
| Poplar, shelf mushrooms, shrubs, cushions, straw beds, and wool and concrete stairs and slabs add states. | Regenerate the full tables through `ModelTable.java` and `UseEffectsTable.java`. Check cushion and straw-bed hooks before admitting their states. [Mojang 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) |
| Registries remove block codecs. Block transformations move to an item component and data. | Compare model generation and the placement and interaction law where it consumes those rules. We only need the outcomes queried by the kit. [Fabric 26.3 port notes](https://www.fabricmc.net/2026/09/15/263.html) |
| Window input moves from GLFW to SDL3. | Check launcher metadata, client input injection descriptors and the mod builds. This is outside Rust player movement because the law consumes already-resolved key states. [Fabric 26.3 port notes](https://www.fabricmc.net/2026/09/15/263.html) |

## Removed from the law diff

Realms invite codes, translation updates, debug overlay scale, the renderer's translucency changes and management log format have no inputs to the Rust law. The renderer change still calls for a build and a separate render check if the swarm client changes. [Mojang 26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3)

## Jar comparison pending

The table identifies observed changes or published comparison leads. Only the saved palette rule has been checked on actual 26.3 game data. The 26.3 jars are missing from the build environment. Release notes alone cannot close the comparison for movement, collision and shapes, fluids, block hooks, effects, attributes, food, damage, and all other ported classes. The diff is open until their Java source or bytecode is compared.
