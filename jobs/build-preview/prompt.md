# A preview renderer, so a builder can see its build before it is placed

**Julien (S205, verbatim):** "ok that was too much for hydrogen to handle, based on my taste and the previous prompts write a new prompt that's more sized like the pagoda with mulitple passes in how it is builted that would be better. server is off so you can have terra too"

**Consumer:** Hydrogen builds for him in passes (`jobs\hearth-small\prompt.md`). Hydrogen writes commands but cannot see them: its three big builds came out blind. Between passes, the integrator renders the commands with your tool and gives the images back to Hydrogen, which looks at them and corrects. The images must show what a player would see.

## Build
One Java program, **`tools\preview\Preview.java`**, run as a single source file with the rig's JDK: `& "$Jdk\java.exe" tools\preview\Preview.java <args>` (`$Jdk` is in `scripts\common.ps1`). Use the JDK only: `java.util.zip`, `javax.imageio`, `java.awt.image`. No other libraries, no network. Add a wrapper, **`scripts\preview.ps1`**, that runs it at Normal priority.

**Input:**
- a command file (`build.txt` style: `fill`/`setblock` lines, `#` comments, leading `/` optional; block states in brackets);
- the box (x0 y0 z0 x1 y1 z1);
- the textures from the staged client jar `D:\path-tracing-deps\minecraft-26.2-client.jar` (`assets/minecraft/textures/block/*.png`; read it, never extract it into the repository; a game jar is never committed);
- a view list file (below);
- an output folder.

The world outside the box is superflat: grass block at y −61, dirt below, air above. Inside the box, the commands are replayed in order onto that ground.

**Shapes**, good enough to judge architecture:
- full cubes;
- slabs (top or bottom);
- stairs (`facing`, `half` and `shape`: a slab plus the raised quarter or quarters);
- fences, walls, iron bars and panes (a post with arms toward neighbours that connect);
- doors and trapdoors (open or closed, by `facing`, `half` and `open`);
- carpets and pressure plates (thin);
- lanterns and chains (small);
- flowers, saplings, grass and ferns (crossed planes);
- logs by `axis`;
- leaves (cutout);
- glass and panes (translucent);
- water (translucent blue surface).

Anything else renders as a cube, listed once in the run's notes.

**Textures:** `<id>.png`, `<id>_top.png`, `<id>_side.png` and `<id>_bottom.png` as each block uses them, with the logs' end and side. Animated strips use their first frame. Tint grass tops, leaves and grass plants with the plains biome colours, and water with its colour.

**Light** (this is what makes a refuge read at night):
- **Day:** a fixed sun direction, with face shading as Minecraft does (top 1.0, sides 0.8 and 0.6, bottom 0.5).
- **Night:** Minecraft's block light. Every emitter (lantern 15, torch 14, soul lantern 10, sea lantern 15, glowstone 15, shroomlight 15, campfire 15, and so on) spreads light by breadth-first decrease of 1 per block through non-opaque cells. The sky is dark and moonlight is faint. Take the emission values and which blocks are opaque from the jar's code or data, not from memory, and cite where.

## Views
The view list is a text file, one view per line:
- `iso <corner NE|NW|SE|SW> <day|night>`: isometric view of the whole box;
- `plan <y|top>`: top-down orthographic at 8 px per block, either the top surface or a horizontal section cut at y;
- `elev <N|E|S|W>`: orthographic elevation;
- `section <x|z> <value>`: vertical cut;
- `eye <x> <y> <z> <yaw> <pitch> <fov> <day|night> <name>`: a first-person perspective view, rendered by casting a ray through the voxel grid per pixel, with distance haze. Yaw and pitch use Minecraft's convention. Eye height is feet + 1.62.

The default size is 1280×720 for eye views and as large as needed for the others. Each image is a PNG named by its view, plus `views.md` listing the images with one line each.

## Gates
- **A fixture test:** a 5×5 stone-brick hut with a spruce door, a slab roof and a lantern inside, rendered from every view type. Assert the image sizes, and that the lantern's cell is lit at night with block light 15 and its neighbours 14. Assert that a planted wrong `facing` on the stairs changes the image (the renderer reads states).
- **Real builds,** run as sample outputs into `jobs\build-preview\samples\`, each with timing:
  - `jobs\pagoda\build-placed-x60.txt` (box x 60..92, y −60..−10, z −16..16), all view types, including an eye view from the courtyard;
  - `jobs\build-hearth\place.txt` (box x 2000..2179, y −61..120, z −60..59): 4 isos day, 1 iso night, a plan, and one eye view on the trail.
- **Time:** the pagoda's full set should take seconds. Report what it takes.

## Must not touch
`engine\`, `bot\`, `server\`, `clients\`, his tracks and runs. Launch no game, server or client. Never run `scripts\launch.ps1`, `gpt.ps1` or `follow-up.ps1`.

**`jobs\build-preview\last.md`:**
- the files;
- how to run it;
- what renders as a plain cube;
- where the emission and opacity values came from;
- the timings;
- the sample images.

End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
