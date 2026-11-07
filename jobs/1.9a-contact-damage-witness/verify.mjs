import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const TICKS = 32;
const ACCEPTED = new Set([0, 10, 20, 30]);
const CASES = ['magma-standing', 'magma-sneaking', 'netherrack-standing'];
const FRAME = [
  'x', 'y', 'z', 'ground', 'pose', 'careful', 'shift', 'move', 'supportPad', 'contactPad',
  'block', 'mode', 'difficulty', 'regen', 'fireDamage', 'effects', 'bare', 'maxHealth', 'absorption',
  'invulnerable', 'loaded', 'overworld', 'rate', 'frozen', 'health', 'cooldown',
];
const REQUEST = ['duringContact', 'source', 'amount', 'accepted', 'before', 'after', 'cdBefore', 'cdAfter'];

function equal(actual, expected, label) {
  if (actual !== expected) throw new Error(`${label}: expected ${expected}, got ${actual}`);
}

function clock(value, label) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 0 || String(parsed) !== value) {
    throw new Error(`${label}: invalid tick ${value}`);
  }
  return parsed;
}

function healthBits(health) {
  const bytes = Buffer.alloc(4);
  bytes.writeFloatBE(health);
  return bytes.toString('hex');
}

function conditions(trial, rate, block) {
  const sneak = trial === 'magma-sneaking';
  return {
    x: '4112.5', y: '-60.0', z: '4100.5', ground: 'true',
    pose: sneak ? 'CROUCHING' : 'STANDING', careful: String(sneak), shift: String(sneak),
    move: 'false', supportPad: 'true', contactPad: 'true', block,
    mode: 'SURVIVAL', difficulty: 'NORMAL', regen: 'false', fireDamage: 'true', effects: '0', bare: 'true',
    maxHealth: healthBits(20), absorption: healthBits(0), invulnerable: 'false',
    loaded: 'true', overworld: 'true', rate: `${rate}.0`, frozen: 'false',
  };
}

function verify(raw, trial, rate) {
  const active = trial === 'magma-standing';
  const block = trial === 'netherrack-standing' ? 'minecraft:netherrack' : 'minecraft:magma_block';
  const rows = raw.split(/\r?\n/);
  equal(rows.pop(), '', 'terminated trace');
  equal(rows.shift(), `contact-v1\t${trial}\tPathBot`, 'header');
  let cursor = 0;

  function take(kind, label, schema, expectedClocks) {
    const [seq, tag, ...rest] = (rows[cursor++] ?? '').split('\t');
    equal(seq, String(cursor), `${label} sequence`);
    equal(tag, kind, `${label} row kind (request order)`);
    equal(rest.length, schema.length + 2, `${label} width`);
    const clocks = rest.slice(0, 2).map((value, i) => clock(value, `${label} clock ${i}`));
    expectedClocks?.forEach((value, i) => equal(clocks[i], value, `${label} clock ${i}`));
    const fields = Object.fromEntries(schema.map((key, i) => [key, rest[i + 2]]));
    return { clocks, fields };
  }

  function snapshot(kind, label, expectedBlock, clocks, health, cooldown) {
    const frame = take(kind, label, FRAME, clocks);
    for (const [key, value] of Object.entries(conditions(trial, rate, expectedBlock))) {
      equal(frame.fields[key], value, `${label} ${key}`);
    }
    equal(frame.fields.health, healthBits(health), `${label} authoritative health`);
    equal(frame.fields.cooldown, String(cooldown), `${label} cooldown`);
    return frame.clocks;
  }

  let clocks = snapshot('P', 'predecessor', 'minecraft:stone', undefined, 20, 0);
  let health = 20;
  let cooldown = 0;
  const accepted = [];
  const healthAfter = [];
  let refused = 0;

  for (let tick = 0; tick < TICKS; tick++) {
    const label = `relative tick ${tick}`;
    clocks = clocks.map(value => value + 1);
    cooldown = Math.max(cooldown - 1, 0);
    snapshot('B', `${label} begin`, block, clocks, health, cooldown);
    snapshot('C', `${label} stepOn`, block, clocks, health, cooldown);

    if (active) {
      const { fields } = take('R', `${label} request`, REQUEST, clocks);
      equal(fields.duringContact, 'true', `${label} request provenance`);
      equal(fields.source, 'minecraft:hot_floor', `${label} damage source`);
      equal(fields.amount, healthBits(1), `${label} raw amount`);
      const accept = ACCEPTED.has(tick);
      equal(fields.accepted, String(accept), `${label} acceptance`);
      equal(fields.before, healthBits(health), `${label} request health before`);
      equal(fields.cdBefore, String(cooldown), `${label} request cooldown before`);
      if (accept) {
        accepted.push(tick);
        health -= 1;
        cooldown = 20;
      } else {
        refused++;
      }
      equal(fields.after, healthBits(health), `${label} request health after`);
      equal(fields.cdAfter, String(cooldown), `${label} request cooldown after`);
    }

    snapshot('E', `${label} end`, block, clocks, health, cooldown);
    healthAfter.push(health);
  }

  const [seq, tag, count, nanos, ...extra] = (rows[cursor++] ?? '').split('\t');
  equal(seq, String(cursor), 'seal sequence');
  equal(tag, 'Z', 'seal tag');
  equal(count, String(TICKS), 'seal count');
  equal(extra.length, 0, 'seal width');
  const captureNanos = clock(nanos, 'capture nanoseconds');
  if (captureNanos === 0) throw new Error('capture span is zero');
  equal(rows.length, cursor, 'rows after seal');
  return { accepted, refused, healthAfter, captureNanos };
}

// Synthetic rows exercise the verifier only. The expected cadence is explicit here
// and the fixture generates health and cooldown values via the >10 recurrence.
function synthetic(trial, rate) {
  const active = trial === 'magma-standing';
  const block = trial === 'netherrack-standing' ? 'minecraft:netherrack' : 'minecraft:magma_block';
  const rows = [];
  const add = (...fields) => rows.push(fields);
  let health = 20;
  let cooldown = 0;
  function frame(kind, world, player, floor) {
    add(kind, world, player, ...Object.values(conditions(trial, rate, floor)), healthBits(health), cooldown);
  }
  frame('P', 500, 200, 'minecraft:stone');
  for (let index = 1; index <= 32; index++) {
    const world = 500 + index;
    const player = 200 + index;
    cooldown = Math.max(cooldown - 1, 0);
    frame('B', world, player, block);
    frame('C', world, player, block);
    if (active) {
      const before = health;
      const cdBefore = cooldown;
      const accepted = cooldown <= 10;
      if (accepted) {
        health--;
        cooldown = 20;
      }
      add('R', world, player, true, 'minecraft:hot_floor', '3f800000', accepted,
        healthBits(before), healthBits(health), cdBefore, cooldown);
    }
    frame('E', world, player, block);
  }
  add('Z', 32, 1);
  return rows;
}

function serialize(trial, rows) {
  return `contact-v1\t${trial}\tPathBot\n` + rows.map((row, i) => [i + 1, ...row].join('\t')).join('\n') + '\n';
}

function selfTest() {
  for (const rate of [20, 2000]) {
    for (const trial of CASES) {
      const result = verify(serialize(trial, synthetic(trial, rate)), trial, rate);
      assert.deepEqual(result.accepted, trial === CASES[0] ? [0, 10, 20, 30] : []);
    }
  }
  const trial = CASES[0];
  function rejects(name, mutate, message) {
    const rows = synthetic(trial, 20);
    mutate(rows);
    assert.throws(() => verify(serialize(trial, rows), trial, 20), message);
    console.log(`PASS rejects ${name}`);
  }
  rejects('every request accepted (including rewritten health trace)', rows => {
    let health = 20;
    for (const row of rows) {
      if (['B', 'C', 'E'].includes(row[0])) {
        row[3 + FRAME.indexOf('health')] = healthBits(health);
        row[3 + FRAME.indexOf('cooldown')] = row[0] === 'E' ? 20 : health === 20 ? 0 : 19;
      } else if (row[0] === 'R') {
        row[3 + REQUEST.indexOf('accepted')] = true;
        row[3 + REQUEST.indexOf('before')] = healthBits(health);
        row[3 + REQUEST.indexOf('after')] = healthBits(--health);
        row[3 + REQUEST.indexOf('cdBefore')] = health === 19 ? 0 : 19;
        row[3 + REQUEST.indexOf('cdAfter')] = 20;
      }
    }
  }, /relative tick 1 acceptance/);
  rejects('dropped accepted health transition in the tick snapshots', rows => {
    const end = rows.find(row => row[0] === 'E' && row[1] === 511);
    end[3 + FRAME.indexOf('health')] = healthBits(19);
  }, /relative tick 10 end authoritative health/);
  rejects('swapped request and stepOn rows with fresh sequence numbers', rows => {
    [rows[2], rows[3]] = [rows[3], rows[2]];
  }, /stepOn row kind/);
  rejects('unrelated damage', rows => {
    rows.find(row => row[0] === 'R')[4] = 'minecraft:fall';
  }, /damage source/);
  rejects('unsettled support', rows => {
    rows.find(row => row[0] === 'C')[3 + FRAME.indexOf('supportPad')] = false;
  }, /stepOn supportPad/);
  rejects('partial capture', rows => rows.pop(), /seal/);
  console.log('PASS six synthetic case/rate traces');
}

try {
  const [mode, ...args] = process.argv.slice(2);
  if (mode === '--self-test' && args.length === 0) {
    selfTest();
  } else if (['20', '2000'].includes(mode) && args.length === 3) {
    for (const [i, trial] of CASES.entries()) {
      const result = verify(readFileSync(args[i], 'utf8'), trial, Number(mode));
      console.log(`PASS ${trial} at configured ${mode} TPS, requests ${result.accepted.length + result.refused}, `
        + `accepted relative ticks [${result.accepted}], refused ${result.refused}, health [${result.healthAfter}], `
        + `capture ${result.captureNanos} ns`);
      if (trial === CASES[0]) {
        console.log(`First difference from the GLM Rust witness: tick 0 end, server ${result.healthAfter[0]}, Rust 20`);
      }
    }
  } else {
    throw new Error('Usage: node verify.mjs --self-test | <20|2000> <magma-standing.tsv> <magma-sneaking.tsv> <netherrack-standing.tsv>');
  }
} catch (error) {
  console.error(`REFUSE ${error.message}`);
  process.exitCode = 1;
}
