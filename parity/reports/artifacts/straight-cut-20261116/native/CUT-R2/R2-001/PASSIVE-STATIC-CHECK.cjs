'use strict';
// Passive AST/schema check only; never executes staged CUA cells or product/evaluator code.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const acorn = require('internal/deps/acorn/acorn/dist/acorn');
const root = __dirname;
const repo = path.resolve(root, '../../..');
const packet = path.join(repo, '.local/straight-cut-native-plan-v2-20261116');
const hash = raw => crypto.createHash('sha256').update(raw).digest('hex');
const read = name => fs.readFileSync(path.join(root, name));
const parse = raw => acorn.parse(raw.toString(), {ecmaVersion:'latest',sourceType:'module',locations:true});
const walk = (node, visit, parent = null) => {
  if (!node || typeof node !== 'object') return;
  if (node.type) visit(node, parent);
  for (const value of Object.values(node)) {
    if (Array.isArray(value)) for (const child of value) walk(child, visit, node);
    else if (value && typeof value === 'object') walk(value, visit, node);
  }
};
const ast = parse(read('ACTOR-BOOTSTRAP.js'));
const checkAst = parse(read('ACTOR-PERSISTENCE-CHECK.js'));
const initialRaw = read('ACTOR-INITIAL-CAPTURE.js');
const initialAst = parse(initialRaw);
const needed = ['cutModule','cutRequire','cutFs','cutPath','cutCrypto','cutProcess','cutRoot','cutRepo','cutPacket','cutHash','cutWrite','cutBindingBytes','cutBindingSha','cutBinding','cutRecorder','cutController','cutObserve','cutMark','cutProofId','cutProofSha'];
const top = new Map(ast.body.filter(node => node.type === 'VariableDeclaration').flatMap(node => node.declarations.map(declaration => [declaration.id.name, declaration])));
for (const name of needed) assert.ok(top.has(name) && top.get(name).init, 'Missing initialized program-top-level var: ' + name);
for (const tree of [ast, checkAst, initialAst]) walk(tree, (node, parent) => {
  if (node.type === 'VariableDeclaration') assert.ok(node.kind === 'var' && parent.type === 'Program', 'Non-top-level declaration at line ' + node.loc.start.line);
});
for (const tree of [ast, checkAst]) walk(tree, node => {
  assert.ok(!(node.type === 'Identifier' && ['cutApp','cutAppResult','cutCurrent','cua'].includes(node.name)), 'Future app/CUA reference in no-UI cell');
});
const observerRaw = fs.readFileSync(path.join(repo, '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs'));
assert.equal(observerRaw.length, 393);
assert.equal(hash(observerRaw), 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729');
assert.ok(initialRaw.equals(fs.readFileSync(path.join(packet, 'CUA-INITIAL-CAPTURE.template.js'))));
let observations = 0;
walk(initialAst, node => {
  if (node.type === 'CallExpression' && node.callee.name === 'cutObserve') {
    observations += 1;
    assert.equal(node.arguments.length, 5);
    assert.deepEqual(node.arguments.map(argument => argument.type === 'Literal' ? argument.value : argument.name), ['cutController','cutApp','initial-owned-cut-state','cutMark','nodeRepl']);
  }
});
assert.equal(observations, 1);
const markerLine = top.get('cutMark').loc.start.line;
assert.ok(markerLine > top.get('cutFs').loc.start.line && markerLine > top.get('cutRoot').loc.start.line && markerLine > top.get('cutProcess').loc.start.line && markerLine > top.get('cutController').loc.start.line);
const binding = JSON.parse(read('ROOT-BINDING-DRAFT.json'));
assert.equal(binding.root_authorized_for_case, false);
assert.equal(binding.native_lease_granted, false);
assert.equal(binding.actor_filename, path.join(root, 'ACTOR-BOOTSTRAP.js'));
assert.equal(binding.actor_sha256, hash(read('ACTOR-BOOTSTRAP.js')));
assert.equal(binding.actor_persistence_check.sha256, hash(read('ACTOR-PERSISTENCE-CHECK.js')));
assert.equal(binding.actor_initial_capture.sha256, hash(initialRaw));
assert.deepEqual(Object.keys(binding.controller_configuration).sort(), ['baselineCalls','callCap','elapsedCapS','evidenceKind','recorderPath','root']);
assert.deepEqual(binding.input_argv, [path.join(root, 'inputs/accepted-redo-r4.weftforge')]);
assert.deepEqual(binding.inputs, {'inputs/accepted-redo-r4.weftforge':'85ef9d89d4080a8994ca7c11f060e37d3c9ad91a613b090e07182fb5de6df366'});
assert.equal(binding.accepted_cut_r1_dependency.root_accepted, false);
assert.equal(binding.accepted_cut_r1_dependency.root_native_ui_review_sha256, null);
const fixtureHash = 'f'.repeat(64);
const artifact = {file:'captures/binding-raw.json',bytes:17,sha256:fixtureHash,source_record:'0001',unedited:true,utc:'PROSPECTIVE_STATIC_PLACEHOLDER',monotonic_ns:'0'};
const env = {cutBinding:binding,cutBindingSha:fixtureHash,cutProofId:'0002',cutProofSha:fixtureHash,cutOfflineReceipt:artifact,cutCheckBinding:binding,cutCheckOfflineReceipt:artifact,cutCheckRecorderPath:binding.recorder};
const value = node => {
  if (node.type === 'Literal') return node.value;
  if (node.type === 'Identifier') { assert.ok(node.name in env, 'Unknown static identifier: ' + node.name); return env[node.name]; }
  if (node.type === 'MemberExpression') return value(node.object)[node.computed ? value(node.property) : node.property.name];
  if (node.type === 'ObjectExpression') return Object.fromEntries(node.properties.map(property => [property.key.name || property.key.value, value(property.value)]));
  if (node.type === 'CallExpression' && node.callee.name === 'cutCheckHash') return fixtureHash;
  throw new Error('Unsupported passive data expression: ' + node.type);
};
let proofFill;
let proofResult;
let offlineResult;
let offlineArtifactCall = false;
let checkMethod = false;
let checkTypes;
let checkFlags;
let checkContext;
let checkProofExpected;
walk(ast, node => {
  if (node.type !== 'CallExpression' || node.callee.type !== 'MemberExpression') return;
  const owner = node.callee.object.name;
  const method = node.callee.property.name;
  if (owner === 'Object' && method === 'assign' && node.arguments[0].name === 'cutProof') proofFill = value(node.arguments[1]);
  if (owner === 'cutRecorder' && method === 'returned' && node.arguments[0].name === 'cutProofId' && node.arguments[1].type === 'ObjectExpression') proofResult = value(node.arguments[1]);
  if (owner === 'cutOffline' && method === 'returned') offlineResult = value(node.arguments[1]);
  if (owner === 'cutOffline' && method === 'artifact') offlineArtifactCall = node.arguments[0].value === 'binding-raw.json' && node.arguments[1].name === 'cutBindingBytes' && node.arguments[2].name === 'cutOfflineId';
});
walk(checkAst, node => {
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression' && node.callee.object.name === 'cutCheckRecorder' && node.callee.property.name === 'before') checkMethod = node.arguments[1].value === 'local-runtime-bindings-check';
  if (node.type !== 'AssignmentExpression' || node.left.type !== 'Identifier') return;
  if (node.left.name === 'cutCheckTypes') checkTypes = node.right.properties.map(property => property.key.name);
  if (node.left.name === 'cutCheckFlags') checkFlags = node.right.properties.map(property => property.key.name);
  if (node.left.name === 'cutCheckContextExpected') checkContext = value(node.right);
  if (node.left.name === 'cutCheckProofExpected') checkProofExpected = value(node.right);
});
assert.deepEqual(checkTypes, needed);
assert.deepEqual(checkFlags, needed);
assert.ok(checkMethod);
const template = JSON.parse(fs.readFileSync(path.join(packet, 'ACTOR-RUNTIME-PROOF.template.json'), 'utf8'));
const plannedProof = Object.assign(template, proofFill);
const expectedProof = {schema:'weftforge.straight-cut.actor-runtime-proof.v1',case:'CUT-R2',attempt:'R2-001',root_binding_sha256:fixtureHash,evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:'0002',actor_filename:binding.actor_filename,actor_sha256:binding.actor_sha256,controller_configuration:binding.controller_configuration,observer_explicit_arguments:5,marker_initialized_synchronous_before_observer:true,controller_sha256:binding.controller_sha256,recorder_sha256:binding.recorder_sha256,observer_sha256:binding.observer_sha256,offline_receipt:artifact};
assert.deepEqual(plannedProof, expectedProof);
assert.deepEqual(checkProofExpected, expectedProof);
assert.deepEqual(proofResult, {proof_sha256:fixtureHash,actual_cua_runtime:true,native_UI_calls:0});
assert.deepEqual(offlineResult, {raw_binding_sha256:fixtureHash});
assert.ok(offlineArtifactCall);
const predictedContext = Object.fromEntries(Object.entries(binding.controller_configuration).filter(([key]) => key !== 'root'));
predictedContext.recorderSha256 = binding.recorder_sha256;
assert.deepEqual(predictedContext, checkContext);
const recorderAst = parse(fs.readFileSync(binding.recorder));
let recorderArtifactKeys;
walk(recorderAst, node => {
  if (node.type === 'VariableDeclarator' && node.id.name === 'receipt' && node.init.type === 'ObjectExpression') recorderArtifactKeys = node.init.properties.flatMap(property => property.type === 'SpreadElement' ? ['utc','monotonic_ns'] : [property.key.name]);
});
assert.deepEqual(recorderArtifactKeys.sort(), Object.keys(artifact).sort());
for (const absent of ['ROOT-BINDING.json','LAUNCH-CLOCK.json','CONTEXT.json','ACTOR-RUNTIME-PROOF.json','ACTOR-PERSISTENCE-CHECK-RESULT.json','SAVED-GRADE-ATTEMPT.json']) assert.ok(!fs.existsSync(path.join(root, absent)), 'Runtime artifact must not exist during preparation: ' + absent);
const report = {kind:'passive Node Acorn AST and planned-schema comparison; not actual CUA proof',passed:true,actual_cua_runtime:false,staged_cells_executed:false,product_or_evaluator_executed:false,parser:'Node bundled Acorn ' + acorn.version,node:process.version,cross_cell_bindings:needed,all_cross_cell_declarations_program_top_level:true,no_nested_var_declarations:true,no_future_app_or_CUA_references_in_FS_cells:true,immutable_five_argument_observer:true,proof_context_result_and_offline_artifact_match_exact_V2_expected_schema:true,persistence_check_independent_imports_and_paired_method:'local-runtime-bindings-check',prospective_record:'0003',planned_original_static_operations:49,planned_adjusted_static_operations:50,planned_startup_operations:13,operation_cap:65,summary_calls:1,overall_call_cap:66,elapsed_cap_s:600};
fs.writeFileSync(path.join(root, 'PASSIVE-STATIC-CHECK.json'), JSON.stringify(report, null, 2) + '\n', {flag:'wx',mode:0o600});
console.log(JSON.stringify(report));
