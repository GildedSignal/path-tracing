# What a pinned word's exit really is, after Astra's 1-ULP witness

You are the Opus Designer in a fresh context (AGENTS.md; S174: "opus and astra for important things"). You think, read, and run small offline experiments. You do not build, edit code or docs, commit, or launch a game, server or client. **The harness may block your file writes: return your full proposal as your final message**, and the integrator saves it to `jobs\pin-rederive\proposal.md`.

## Read first (root `C:\Users\PASCAL\Desktop\path tracing\`)
1. `docs\PRINCIPLES.md` lines 1–47: the goal and Julien's philosophy, always on.
2. `docs\plan.md`, steps 2 and 2b. Julien (S186): "it is important to get even the simplest case right, if we can't make them right the rest will compound badly". S191: bottom-up, layer after layer.
3. `docs\learning.md` sections 0 and 6 (kill test 11), and `docs\representation.md` section 10. Line 179 is the ruling (S188): sets are families and points.
4. **The design that met the law:** `jobs\set-rederive\proposal.md` (yours, from a fresh context), all of it, and its crate `jobs\set-rederive\kt11x\`.
5. **Astra's build record:** `D:\path-tracing-wt\u2b-astra\jobs\u2b\last.md` (branch `wt/u2b-astra`), with its evidence in `D:\path-tracing-evidence\u2b\` and its tests in `D:\path-tracing-wt\u2b-astra\engine\plan\tests\u2b\`.

## What failed
The proposal said: where the word clamps the free coordinate, the law forgets it, so a piece's image is at most 2^f exact points, one per value of the declared non-monotone flag (f = 1: `minor_horizontal_collision`, `movement.rs:818`). Astra found, by direct `law::try_tick` on the gap fixture at yaw 275.625:

```text
U2B_PIN_WITNESS inputs=405818d3c15831d9,40581c7285449b7c tick=13 x=40592cccccc00000,40592cccccbfffff flag=false,false
```

Same pre-pin signature, same tick, same flag, and the pinned x differs by 1 ULP, in the reverse order of the inputs. The law adds a rounded moved distance to the old x; it does not assign the wall's coordinate. So the pin keeps a trace of the entry, and the clamp is not monotone at that boundary. Gap and step at yaw 275.625 refuse; yaw 270, `glm-ice` and `hardtrack-lane-wall` compile (5, 9, 2, 2 pieces). Everything else Astra built stands (kill test 10 at 158 against his 169, the core's regression, all mutants caught). The stored stop on B and zero-copy full solves wait on this answer.

## The question
**What is the exact image of a piece through a word that clamps its free coordinate, so that sets stay exact, stay few, and stay one mechanism for every terrain?**

Think from the law outward; never cut on rounding dust or add a tolerance (AGENTS.md, "No patches").
- Where exactly does the 1-ULP trace come from (which operations of `collide` and the position update, file:line), and is it bounded by a law fact: how many distinct pinned values can one piece produce, and why?
- Does the trace matter downstream? After the pin, does the next event (landing, the next tick's collision) erase it, so that points merge one tick later, or does it reach T or the next word's choice? Run it with `kt11x` or a small crate of your own (`CARGO_TARGET_DIR=D:\path-tracing-target\pin-rederive`, offline, Normal priority).
- Is the right object still "points", or a tiny exact family on the output side (an interval of the pinned lattice with its own monotone map), or should the cut move to where the law truly forgets?
- Why does yaw 270 close and 275.625 not? Does the answer cover every yaw on the bot's grid?
- What it costs at the two extremes: the million-block line and a hard track as Julien defines it (S166), with stated inputs.
- **The builder's list:** what changes in `representation.md:179` and kill test 11, what changes in Astra's code on `wt/u2b-astra`, and the frozen test that proves it, with the witness above as one of its cases.

Cite file:line for every claim about the law. Where you ran something, give the command and its output.
