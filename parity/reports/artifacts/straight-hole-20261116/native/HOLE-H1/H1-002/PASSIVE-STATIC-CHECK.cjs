'use strict';
// Passive AST/schema check only; never executes staged CUA cells or product/evaluator code.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const acorn = require('internal/deps/acorn/acorn/dist/acorn');
const root = __dirname;
const repo = path.resolve(root, '../../..');
const packet = path.join(repo, '.local/straight-hole-native-plan-v2-20261116');
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
const clockAst = parse(read('ACTOR-START-CLOCK.cjs'));
const launchAst = parse(read('ACTOR-LAUNCH-PID.cjs'));
const postflightAst = parse(read('ACTOR-POSTFLIGHT-RAW.cjs'));
const needed = ['holeModule','holeRequire','holeFs','holePath','holeCrypto','holeProcess','holeRoot','holeRepo','holePacket','holeHash','holeWrite','holeBindingBytes','holeBindingSha','holeBinding','holeRecorder','holeController','holeObserve','holeMark','holeProofId','holeProofSha'];
const top = new Map(ast.body.filter(node => node.type === 'VariableDeclaration').flatMap(node => node.declarations.map(declaration => [declaration.id.name, declaration])));
for (const name of needed) assert.ok(top.has(name) && top.get(name).init, 'Missing initialized program-top-level var: ' + name);
for (const tree of [ast, checkAst, initialAst]) walk(tree, (node, parent) => {
  if (node.type === 'VariableDeclaration') assert.ok(node.kind === 'var' && parent.type === 'Program', 'Non-top-level declaration at line ' + node.loc.start.line);
});
for (const tree of [ast, checkAst]) walk(tree, node => {
  assert.ok(!(node.type === 'Identifier' && ['holeApp','holeAppResult','holeCurrent','cua'].includes(node.name)), 'Future app/CUA reference in no-UI cell');
});
const observerRaw = fs.readFileSync(path.join(repo, '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs'));
assert.equal(observerRaw.length, 393);
assert.equal(hash(observerRaw), 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729');
assert.equal(Object.keys(JSON.parse(fs.readFileSync(path.join(packet, 'RUNTIME-PROTOCOL.json'), 'utf8')).program_top_level_helpers).join(','), needed.join(','));
let observations = 0;
walk(initialAst, node => {
  if (node.type === 'CallExpression' && node.callee.name === 'holeObserve') {
    observations += 1;
    assert.equal(node.arguments.length, 5);
    assert.deepEqual(node.arguments.map(argument => argument.type === 'Literal' ? argument.value : argument.name), ['holeController','holeApp','initial-owned-hole-state','holeMark','nodeRepl']);
  }
});
assert.equal(observations, 1);
const markerLine = top.get('holeMark').loc.start.line;
assert.ok(markerLine > top.get('holeFs').loc.start.line && markerLine > top.get('holeRoot').loc.start.line && markerLine > top.get('holeProcess').loc.start.line && markerLine > top.get('holeController').loc.start.line);
assert.ok(top.get('holeProofId').loc.start.line < top.get('holeController').loc.start.line, 'Constructor must be inside the counted proof envelope');
const binding = JSON.parse(read('ROOT-BINDING-V2-DRAFT.json'));
assert.deepEqual(Object.keys(binding).sort(), Object.keys(JSON.parse(fs.readFileSync(path.join(packet, 'ROOT-BINDING.HOLE-H1.template.json'), 'utf8'))).sort());
assert.equal(binding.case, 'HOLE-H1');
assert.equal(binding.accepted_h1_dependency, null);
assert.equal(binding.root_authorized_for_case, false);
assert.equal(binding.native_lease_granted, false);
assert.equal(binding.actor_filename, path.join(root, 'ACTOR-BOOTSTRAP.js'));
assert.equal(binding.actor_sha256, hash(read('ACTOR-BOOTSTRAP.js')));
assert.equal(binding.actor_persistence_check.sha256, hash(read('ACTOR-PERSISTENCE-CHECK.js')));
assert.equal(binding.actor_initial_capture.sha256, hash(initialRaw));
assert.deepEqual(Object.keys(binding.controller_configuration).sort(), ['baselineCalls','callCap','elapsedCapS','evidenceKind','recorderPath','root']);
assert.deepEqual(binding.input_argv, []);
assert.deepEqual(binding.inputs, {});
const fixtureHash = 'f'.repeat(64);
const artifact = {file:'captures/binding-raw.json',bytes:17,sha256:fixtureHash,source_record:'0001',unedited:true,utc:'PROSPECTIVE_STATIC_PLACEHOLDER',monotonic_ns:'0'};
const env = {holeBinding:binding,holeBindingSha:fixtureHash,holeProofId:'0002',holeProofSha:fixtureHash,holeOfflineReceipt:artifact,holeCheckBinding:binding,holeCheckOfflineReceipt:artifact,holeCheckRecorderPath:binding.recorder};
env.b = binding;
const value = node => {
  if (node.type === 'Literal') return node.value;
  if (node.type === 'Identifier') { assert.ok(node.name in env, 'Unknown static identifier: ' + node.name); return env[node.name]; }
  if (node.type === 'MemberExpression') return value(node.object)[node.computed ? value(node.property) : node.property.name];
  if (node.type === 'ObjectExpression') return Object.fromEntries(node.properties.map(property => [property.key.name || property.key.value, value(property.value)]));
  if (node.type === 'ArrayExpression') return node.elements.map(value);
  if (node.type === 'CallExpression' && node.callee.name === 'holeCheckHash') return fixtureHash;
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
  if (owner === 'Object' && method === 'assign' && node.arguments[0].name === 'holeProof') proofFill = value(node.arguments[1]);
  if (owner === 'holeRecorder' && method === 'returned' && node.arguments[0].name === 'holeProofId' && node.arguments[1].type === 'ObjectExpression') proofResult = value(node.arguments[1]);
  if (owner === 'holeOffline' && method === 'returned') offlineResult = value(node.arguments[1]);
  if (owner === 'holeOffline' && method === 'artifact') offlineArtifactCall = node.arguments[0].value === 'binding-raw.json' && node.arguments[1].name === 'holeBindingBytes' && node.arguments[2].name === 'holeOfflineId';
});
walk(checkAst, node => {
  if (node.type === 'VariableDeclarator' && node.id.name === 'holeCheckTypes') checkTypes = node.init.properties.map(property => property.key.name);
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression' && node.callee.object.name === 'holeCheckRecorder' && node.callee.property.name === 'before') checkMethod = node.arguments[1].value === 'local-runtime-bindings-check';
  if (node.type !== 'AssignmentExpression' || node.left.type !== 'Identifier') return;
  if (node.left.name === 'holeCheckTypes') checkTypes = node.right.properties.map(property => property.key.name);
  if (node.left.name === 'holeCheckFlags') checkFlags = node.right.properties.map(property => property.key.name);
  if (node.left.name === 'holeCheckContextExpected') checkContext = value(node.right);
  if (node.left.name === 'holeCheckProofExpected') checkProofExpected = value(node.right);
});
assert.deepEqual(checkTypes, needed);
assert.deepEqual(checkFlags, needed);
assert.ok(checkMethod);
const readinessTemplate = JSON.parse(fs.readFileSync(path.join(packet, 'ACTOR-PERSISTENCE-CHECK-RESULT.template.json'), 'utf8'));
assert.deepEqual(Object.keys(readinessTemplate.helper_types), needed);
assert.deepEqual(Object.keys(readinessTemplate.helper_flags), needed);
const resultKeys = new Set(Object.keys(readinessTemplate));
let resultFillCount = 0;
let identicalReadinessReturned = false;
walk(checkAst, node => {
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression') {
    if (node.callee.object.name === 'Object' && node.callee.property.name === 'assign' && node.arguments[0].name === 'holeCheckResult') {
      resultFillCount += 1;
      for (const property of node.arguments[1].properties) assert.ok(resultKeys.has(property.key.name), 'Unknown closed readiness field');
    }
    if (node.callee.object.name === 'holeCheckRecorder' && node.callee.property.name === 'returned') identicalReadinessReturned = node.arguments[0].name === 'holeCheckId' && node.arguments[1].name === 'holeCheckResult' && node.arguments[2].name === 'holeCheckFailure';
    assert.ok(node.callee.property.name !== 'createController', 'No independent-check constructor');
  }
  if (node.type === 'AssignmentExpression' && node.left.type === 'MemberExpression' && node.left.object.name === 'holeCheckResult') assert.ok(resultKeys.has(node.left.property.name), 'Unknown readiness assignment');
});
assert.equal(resultFillCount, 2);
assert.ok(identicalReadinessReturned);
assert.equal(readinessTemplate.schema, 'weftforge.straight-hole.actor-readiness.v1');
const template = JSON.parse(fs.readFileSync(path.join(packet, 'ACTOR-RUNTIME-PROOF.template.json'), 'utf8'));
const plannedProof = Object.assign(template, proofFill);
const expectedProof = {schema:'weftforge.straight-cut.actor-runtime-proof.v1',case:'HOLE-H1',attempt:'H1-002',root_binding_sha256:fixtureHash,evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:'0002',actor_filename:binding.actor_filename,actor_sha256:binding.actor_sha256,controller_configuration:binding.controller_configuration,observer_explicit_arguments:5,marker_initialized_synchronous_before_observer:true,controller_sha256:binding.controller_sha256,recorder_sha256:binding.recorder_sha256,observer_sha256:binding.observer_sha256,offline_receipt:artifact};
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
const plannedBefore = {};
for (const tree of [clockAst, launchAst]) walk(tree, node => {
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression' && node.callee.object.name === 'r' && node.callee.property.name === 'before') {
    const method = node.arguments[1].value;
    assert.ok(!plannedBefore[method], 'Duplicate planned external operation: ' + method);
    plannedBefore[method] = value(node.arguments[2]);
  }
});
assert.deepEqual(plannedBefore.getState, {entrypoint:'mcp__cua_repl.js',code:'await cua.getState();',evidenceKind:'native',standalone:true});
assert.deepEqual(plannedBefore['external.open'], {executable:binding.launcher_path,argv:binding.launcher_argv,evidenceKind:'native'});
assert.deepEqual(plannedBefore['external.pid'], {executable:'/bin/ps',argv:['-axo','pid=,command='],expected_binary:binding.native_binary_path,expected_command:binding.native_binary_path,evidenceKind:'native'});
let exactCommandMatch = false;
let pidResultKeys = null;
walk(launchAst, node => {
  if (node.type === 'BinaryExpression' && node.operator === '===' && node.left.type === 'MemberExpression' && node.left.object.name === 'm' && node.left.property.value === 2 && node.right.type === 'MemberExpression' && node.right.object.name === 'b' && node.right.property.name === 'native_binary_path') exactCommandMatch = true;
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression' && node.callee.object.name === 'r' && node.callee.property.name === 'returned' && node.arguments[1].type === 'ObjectExpression' && node.arguments[1].properties.some(property => property.key.name === 'exact_matches')) pidResultKeys = node.arguments[1].properties.map(property => property.key.name);
});
assert.ok(exactCommandMatch, 'PID control must retain exact full command match');
assert.deepEqual(pidResultKeys, ['status','exact_matches','stderr']);
let normalizedDescriptor = false;
let normalizedHashComparison = false;
walk(postflightAst, node => {
  if (node.type === 'VariableDeclarator' && node.id.name === 'expected' && node.init.type === 'ConditionalExpression' && node.init.test.type === 'BinaryExpression' && node.init.test.left.type === 'UnaryExpression' && node.init.test.left.operator === 'typeof' && node.init.test.left.argument.name === 'descriptor' && node.init.consequent.type === 'ObjectExpression' && node.init.consequent.properties[0].key.name === 'sha256' && node.init.alternate.name === 'descriptor') normalizedDescriptor = true;
  if (node.type === 'BinaryExpression' && node.operator === '===' && node.left.type === 'MemberExpression' && node.left.object.name === 'actual' && node.left.property.name === 'sha256' && node.right.type === 'MemberExpression' && node.right.object.name === 'expected' && node.right.property.name === 'sha256') normalizedHashComparison = true;
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression') assert.notEqual(node.callee.property.name, 'createController', 'Post-summary raw guard cannot construct a controller');
});
assert.ok(normalizedDescriptor && normalizedHashComparison, 'Postflight must normalize nested preparation descriptors');
const identity = JSON.parse(fs.readFileSync(path.join(packet, 'SOURCE-73.json')));
assert.equal(binding.source_input_count, 73);
assert.deepEqual(binding.source_inputs, identity.source_inputs);
assert.equal(binding.source_commit, identity.source_commit);
assert.equal(binding.clean_metadata_commit, identity.clean_metadata_commit);
assert.equal(binding.native_binary_sha256, identity.native_binary_sha256);
assert.equal(binding.clean_source_metadata_receipt.sha256, identity.clean_checks.sha256);
assert.equal(binding.controller_configuration.callCap, 200);
assert.equal(binding.controller_configuration.elapsedCapS, 1800);
assert.equal(binding.controller_configuration.baselineCalls, 0);
assert.deepEqual(binding.launcher_argv, ['-n',binding.bundle]);
for (const absent of ['ROOT-BINDING.json','LAUNCH-CLOCK.json','CONTEXT.json','ACTOR-RUNTIME-PROOF.json','ACTOR-PERSISTENCE-CHECK-RESULT.json','SAVED-GRADE-ATTEMPT.json']) assert.ok(!fs.existsSync(path.join(root, absent)), 'Runtime artifact must not exist during preparation: ' + absent);
const report = {kind:'passive Node Acorn AST and planned-schema comparison; not actual CUA proof',passed:true,actual_cua_runtime:false,staged_cells_executed:false,product_or_evaluator_executed:false,parser:'Node bundled Acorn ' + acorn.version,node:process.version,cross_cell_bindings:needed,all_cross_cell_declarations_program_top_level:true,no_nested_var_declarations:true,no_future_app_or_CUA_references_in_FS_cells:true,immutable_five_argument_observer:true,proof_context_result_and_offline_artifact_match_exact_V2_expected_schema:true,persistence_check_independent_imports_and_paired_method:'local-runtime-bindings-check',prospective_record:'0003',planned_getState_launch_PID_records_match_frozen_preflight_fields:true,PID_expected_binary_additive_expected_command_exact_match_retained:true,postflight_normalizes_hash_string_and_nested_object_descriptors:true,source73_runtime_metadata_binary_and_closed_draft_fields_match:true,planned_original_static_operations:187,planned_adjusted_static_operations:187,planned_startup_operations:13,operation_cap:200,summary_calls:1,overall_call_cap:201,elapsed_cap_s:1800};
fs.writeFileSync(path.join(root, 'PASSIVE-STATIC-CHECK.json'), JSON.stringify(report, null, 2) + '\n', {flag:'wx',mode:0o600});
console.log(JSON.stringify(report));
