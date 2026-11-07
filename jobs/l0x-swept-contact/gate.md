# L0-x ? one-ULP swept contact

Freeze before running. Check Mojang 26.2 AABB.collidedAlongVector against the
Rust swept contact law on the same raw body, displacement and shape. Sweep
distances from the origin through 10,000, 1,000,000 and 29,999,984 blocks, with
both signs. At each distance examine exact tangency and the representable
coordinates within four ULPs of the vanilla inset. Include both contact and
no-contact controls. Use a fixed seeded set of offsets and a second family
with diagonal motion through a two-box shape. Run all cells and report the
first mismatch per family and distance, with raw inputs retained.

Every result must match the staged jar. The negative control removes the
body inflation inset from the rarely exercised swept contact branch in a
disposable Rust build. A grazing case must distinguish the mutant from the
jar. This is a narrow law falsifier. The full L0 gate and the separate mixed
terrain run retain their own pass lines.
