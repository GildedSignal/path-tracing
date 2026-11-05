// Offline structural checks. Pass a YAML module path if `yaml` is not installed.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const imported = require(process.argv[2] || 'yaml');
const yaml = imported.yaml || imported;
const source = fs.readFileSync(path.join(__dirname, '../../.github/workflows/glm-lab.yml'), 'utf8');
const doc = yaml.parseDocument(source);
assert.deepEqual(doc.errors, []);
assert.deepEqual(doc.warnings, []);
const workflow = doc.toJS();
function check(w) {
  assert.deepEqual(Object.keys(w.on), ['workflow_dispatch']);
  assert.deepEqual(Object.keys(w.on.workflow_dispatch.inputs).sort(), ['lenses', 'minutes']);
  assert.equal(w.on.workflow_dispatch.inputs.lenses.type, 'string');
  assert.equal(w.on.workflow_dispatch.inputs.minutes.type, 'number');
  assert.equal(w.on.workflow_dispatch.inputs.minutes.default, 60);
  assert.deepEqual(w.permissions, { contents: 'read' });
  assert.deepEqual(Object.keys(w.jobs), ['runs']);
  const job = w.jobs.runs;
  assert.equal(job['runs-on'], 'ubuntu-latest');
  assert.equal(job['timeout-minutes'], 300);
  assert.equal(job.steps.length, 4);
  const [checkout, install, run, upload] = job.steps;
  assert.equal(checkout.uses, 'actions/checkout@v4');
  assert.equal(checkout.with['persist-credentials'], false);
  assert.equal(install.run, 'bash lab/setup.sh');
  assert.equal(run.id, 'runs');
  assert.equal(run.env.ZAI_API_KEY, '${{ secrets.ZAI_API_KEY }}');
  assert.equal(run.env.LAB_LENSES, '${{ inputs.lenses }}');
  assert.equal(run.env.LAB_MINUTES, '${{ inputs.minutes }}');
  assert.match(run.run, /bash lab\/actions\.sh/);
  assert.equal(upload.uses, 'actions/upload-artifact@v4');
  assert.equal(upload.if, "${{ always() && steps.runs.outputs.sanitized == 'true' }}");
  assert.equal(upload.with.path, 'lab/results/');
  assert.equal(upload.with['include-hidden-files'], true);
  assert.equal(upload.with.name, 'glm-lab-results');
  assert.equal(upload.with['retention-days'], 14);
  const secretPaths = [];
  function visit(value, parts = []) {
    if (typeof value === 'string' && /secrets\./.test(value)) secretPaths.push(parts.join('.'));
    if (value && typeof value === 'object') for (const [k, v] of Object.entries(value)) visit(v, [...parts, k]);
  }
  visit(w);
  assert.deepEqual(secretPaths, ['jobs.runs.steps.2.env.ZAI_API_KEY']);
}
check(workflow);
// Controls: a public trigger, a job-wide secret or unsanitized upload must fail.
for (const mutate of [
  w => { w.on.pull_request = {}; },
  w => { w.jobs.runs.env = { ZAI_API_KEY: '${{ secrets.ZAI_API_KEY }}' }; },
  w => { w.jobs.runs.steps[3].if = '${{ always() }}'; },
]) {
  const mutant = structuredClone(workflow);
  mutate(mutant);
  assert.throws(() => check(mutant));
}
console.log('PASS: valid YAML; dispatch only, permissions, credential-free checkout, run-only secret, bounded job and scrub-gated artifact (negative controls rejected)');
