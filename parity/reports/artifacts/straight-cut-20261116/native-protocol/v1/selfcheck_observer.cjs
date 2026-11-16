'use strict';
// Standard Node vm mocks only; not an actual-CUA runtime proof.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const observerPath = path.resolve(__dirname, '../native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs');
const observer = fs.readFileSync(observerPath);
const initial = fs.readFileSync(path.join(__dirname, 'CUA-INITIAL-CAPTURE.template.js'), 'utf8');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

async function main() {
  assert.equal(observer.length, 393);
  assert.equal(hash(observer), 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729');
  for (const name of ['CUT-R1', 'CUT-R2']) {
    const binding = JSON.parse(fs.readFileSync(path.join(__dirname, 'ROOT-BINDING.' + name + '.template.json')));
    assert.equal(binding.root_authorized_for_case, false);
    assert.equal(binding.native_lease_granted, false);
    assert.deepEqual(Object.keys(binding.controller_configuration).sort(),
      ['root', 'recorderPath', 'evidenceKind', 'callCap', 'elapsedCapS', 'baselineCalls'].sort());
  }
  const definition = vm.createContext({module: {exports: {}}});
  let reads = 0;
  for (const name of ['cutApp', 'cutController']) {
    Object.defineProperty(definition, name, {get() { reads++; throw Error('future binding read'); }});
  }
  vm.runInContext(observer.toString(), definition);
  const app = {synthetic: true}; // Created after observer definition, in a separate context.
  const events = [];
  const capture = {sourceRecord: 'SYNTHETIC-ONLY', status: {}, state: 'mock',
    images: [{bytes: new Uint8Array([1, 2]), mimeType: 'image/jpeg'}]};
  const context = vm.createContext({
    cutController: {
      async call(target, label, method, args) {
        assert.equal(method, 'getApp'); assert.equal(args[0], 'SYNTHETIC-NOT-A-BUNDLE');
        events.push('getApp'); return {value: app};
      },
      async capture(actualApp, label) {
        assert.equal(actualApp, app); assert.equal(label, 'initial-owned-cut-state');
        events.push('capture'); return capture;
      },
    },
    cutBinding: {bundle: 'SYNTHETIC-NOT-A-BUNDLE'}, cua: {}, cutObserve: definition.module.exports.observe,
    cutMark(source) { assert.equal(source, capture.sourceRecord); events.push('marker'); },
    nodeRepl: {
      write(value) { assert.equal(value.sourceRecord, capture.sourceRecord); events.push('write'); },
      async emitImage(value) { assert.equal(value.bytes, capture.images[0].bytes); events.push('image'); },
    },
  });
  await vm.runInContext('(async function () {\n' + initial + '\n})()', context);
  assert.equal(reads, 0);
  assert.deepEqual(events, ['getApp', 'capture', 'marker', 'write', 'image']);
  const result = {status: 'passed_standard_Node_mock_observer_only', actual_CUA_runtime_proof: false,
    real_native_or_UI_calls: 0, real_controller_constructor_invoked: false, observer_bytes: observer.length,
    explicit_arguments: 5, future_app_or_controller_free_binding_reads: reads,
    actual_callsite_mocked_in_separate_vm_context: true, observer_sha256: hash(observer)};
  fs.writeFileSync(path.join(__dirname, 'SELF-CHECK-OBSERVER-001.json'), JSON.stringify(result, null, 2) + '\n', {flag: 'wx'});
  process.stdout.write(JSON.stringify(result) + '\n');
}
main().catch(error => {process.stderr.write(error.stack + '\n'); process.exitCode = 1;});
