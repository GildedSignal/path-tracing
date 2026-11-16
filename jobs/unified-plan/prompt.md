# The unified plan: one mechanism from a million-block line to the hardest parkour

You are a Fable 5.2 sub-agent, the Designer for this job, in a fresh context. You think, research and write. You do not build, edit code or docs, commit, push, or launch anything. Your output is a proposal Julien reads and rules on (PRINCIPLES 24: "think together first, then build fast").

## Why this job exists

Julien (S164, verbatim, the brief):

> Have a fable sub-agent update the plan thinking about the hard and easy case, make it unified and coherent thinking also about the other things in the plan that are already meant like far field and such, it should read all the doc and all of what i said and what we learned, fermi estimates. It should research the math, representations and so on at a high level from the legends in the field, see if we missed things or removed good things or badly incorporated ideas / badly wired them, it should think in all the time/spatial scales and how they communicate and allow to make less work, it should have all my verbatim, it should be deeply aware of computer architecture and how to leverage it like a quant trader would, proper way of thinking of the conditional T map also with the wormholes and such and even being able to mine blocks or add them or fly in elytra, the goal is to reduce T thinking time while solving the most complex puzzle possible by folding the space such that it becomes trivial, we might not even need A* like algos. maybe the resulting space will be weird and need weird tools like octonions or whatever there's so much powerful math that just does not pop in the head but would be extremely good i don't know but we start with ideas and constraints to come up with the lowest T thinking time possible we're digging papers of math and physics of kings or even the prince himself.

**What went wrong (read it first):** `C:\Users\PASCAL\Desktop\path tracing\docs\reference\postmortem-2026-11-16.md`. The planner forked into a "plain path" for easy ground and the old A* for "exceptional spans" (S161: "we can't have two paths like that "easy" or "hard" it should be the same mechanism"). The living docs stopped absorbing his turns on 14 November.

## Read everything, in this order, by absolute path (root `C:\Users\PASCAL\Desktop\path tracing\`)

1. **`docs\PRINCIPLES.md` in full.** Its first two sections, the goal and the philosophy, are his words and are always on. The register follows, and so does the contradictions table.
2. **All his verbatim:** `docs\reference\julien-2026-11-04.md`, `julien-2026-11-09.md`, `julien-2026-11-10.md` (S1–S164, X1–X16), and `docs\reference\leads.md`. Every turn, not a sample. Note which ideas of his never reached a doc.
3. **The living docs:** `AGENTS.md`, `docs\learning.md`, `representation.md`, `scales.md`, `engine.md`, `vocabulary.md`, `footprints.md`, `ladder.md`, `server.md`, `HANDOFF.md` (its roadmap), and the approved plan `C:\Users\PASCAL\.claude\plans\wiggly-questing-lemur.md`.
4. **His ES engine,** the source of the philosophy: https://github.com/GildedSignal/emergent-space-engine. At least `docs/PRINCIPLES.md` and the docs on certificates, worldtubes, light cones, multipoles and fermi.
5. **What we learned and measured:**
   - the designs: `jobs\zones-design\design.md`, `redteam-opus.md`; `jobs\3.2d-3\spec.md` (the forked spec);
   - the audits: `jobs\inventory-map\report.md`, `jobs\port-backlog\report.md`, `jobs\spec-premises\report.md`, `jobs\spec-fixes\report.md`, `jobs\event-forms\report.md`, `jobs\glm-digest\report.md`, `jobs\docs-truth\report.md`;
   - the evidence: `jobs\3.2d-3-P1-judge\last.md` and `D:\path-tracing-wt\3.2d-3-P1\jobs\3.2d-3-P1\last.md` (the stored opening: 263-tick transient, 12-tick cycle, µs lookup); `jobs\hard-track\report.md` and `shortcut.md` (his 428-tick shortcut that the "proofs" missed); `lab\wiki\index.md` (the lab's settled facts).
6. **The code, to ground claims, at the level you need:** `engine\law\src\` (the tick law), `engine\words\src\` (words, programs), `engine\plan\src\` (today's planner, A*, fields).

## Think, then write

The objective is his: **minimise T, thinking time included** (S6, S90, S124: end to end from the click's server tick). The test is his two extremes, held at once (S131):
- a one-million-block straight line on plain ground with bare legs;
- the most convoluted tool-heavy 3D parkour, with tight timings and non-trivial tool combinations.

Both must be **one mechanism**, whose compute mirrors the terrain's complexity (S131). Work it out across:

1. **The space.** Use the conditional T map: T belongs to the arriving state, not to a block (S91). Fold in:
   - momentum;
   - kits as classes (S134);
   - wormholes: pearls, and Nether portals as built manifolds (S132);
   - edits as tolls with restoration (S32, S13): mining, placing, water;
   - elytra (one long arc);
   - boats;
   - other players' cones (S121).

   What is the right space, and what folding makes the search trivial (PRINCIPLES 3, 7)? Is there a representation where no A*-like search is needed in acting, and where search lives only in learning (learning.md section 1)? Be concrete about what a node, an edge or a set is, if any remain.
2. **Every scale and how they talk** (scales.md): from the tick to the vocabulary's lifetime. What each scale hands up and down so that the others do less work: the far field, multipoles, landmarks, zones from need (S132), cutting points and parallel segments (S131), the wiggle (S94), the cheap hardness signal and changing your mind fast (S121).
3. **The machine, like a quant would think of it:** cache lines, memory traffic, SIMD, branch prediction, the GPU's integer throughput, threads, latency tails (p99, ES 12), and precomputing in idle time. Give Fermi floors with every input stated, never a made-up number (S132), for:
   - the 1M-block line;
   - his 21 tracks;
   - the hard track (his 428 ticks);
   - a 10⁴-block varied trip;
   - 10⁴ bots.
4. **The math and physics, from primary sources.** Search for ideas that fold the space:
   - tropical / min-plus geometry and Hamilton–Jacobi (Hopf–Lax);
   - viscosity solutions and fast marching (Sethian);
   - Finsler and sub-Riemannian geometry, optimal control and reachability (Pontryagin, Bellman, Hamilton–Jacobi reachability);
   - kinetic data structures;
   - hierarchical decompositions (multigrid, fast multipole, Hashlife, contraction hierarchies, hub labels);
   - Morse theory and saddles ("water can branch on saddle points", S132);
   - symplectic and Lie-group views of the recurrence, and number theory for the binade and modulus structure;
   - anything stranger that genuinely fits.

   He asks for the masters ("the kings or even the prince himself": Gauss, and the like). Read the primary paper where you cite one, and say how far you read each.
5. **An audit of the current plan against his words:** what we missed, what good thing we removed, what we incorporated badly or wired wrongly. Cite his turn and the doc line for each.

## Write (`C:\Users\PASCAL\Desktop\path tracing\jobs\unified-plan\`)

- **`plan.md`:** the unified mechanism, written plainly in his vocabulary (words, sentences, certificates, worldtubes, light cones, T, zones, portals). It covers:
  - the space;
  - the loop at every scale;
  - how the two extremes and the middle run through the same loop, walked through as the bot;
  - where computer architecture enters;
  - the Fermi floors;
  - kill tests;
  - the order of work that replaces the 3.2d-3 spec's slices C–K. Slices A and B (channel, codec) and premise tests P1–P4 exist; say what stands.

  At most 400 lines.
- **`audit.md`:** the audit (5 above), one row per finding with its citations.
- **`sources.md`:** each source, what it gives us, how far you read it, and whether it is primary.
- **`questions.md`:** the decisions that are his, each with context, the options, and your recommendation.

## Rules

- **No made-up numbers.** Every constant comes from the law, a measurement, or a stated Fermi input.
- **No patches, tolerances, special cases or terrain-kind code paths** (AGENTS.md, ES 19). Run the one-mechanism test on your own design.
- **Never reword his words;** quote them. Cite every claim by file and line, or by URL.
- **Write as you go,** appending, so a stopped run still leaves the work.

Your final message: the five most important findings, and where the files are.
