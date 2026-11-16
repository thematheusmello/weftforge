'use strict';
// Passive AST/schema check only; never executes staged CUA cells or product/evaluator code.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const acorn = require('internal/deps/acorn/acorn/dist/acorn');
const root = __dirname;
const repo = path.resolve(root, '../../..');
const packet = path.join(repo, '.local/cut-and-sew-native-plan-20261116');
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
const needed = ['sewModule','sewRequire','sewFs','sewPath','sewCrypto','sewProcess','sewRoot','sewRepo','sewPacket','sewHash','sewWrite','sewBindingBytes','sewBindingSha','sewBinding','sewRecorder','sewController','sewObserve','sewMark','sewProofId','sewProofSha'];
const top = new Map(ast.body.filter(node => node.type === 'VariableDeclaration').flatMap(node => node.declarations.map(declaration => [declaration.id.name, declaration])));
for (const name of needed) assert.ok(top.has(name) && top.get(name).init, 'Missing initialized program-top-level var: ' + name);
for (const tree of [ast, checkAst, initialAst]) walk(tree, (node, parent) => {
  if (node.type === 'VariableDeclaration') assert.ok(node.kind === 'var' && parent.type === 'Program', 'Non-top-level declaration at line ' + node.loc.start.line);
});
for (const tree of [ast, checkAst]) walk(tree, node => {
  assert.ok(!(node.type === 'Identifier' && ['sewApp','sewAppResult','sewCurrent','cua'].includes(node.name)), 'Future app/CUA reference in no-UI cell');
});
const observerRaw = fs.readFileSync(path.join(repo, '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs'));
assert.equal(observerRaw.length, 393);
assert.equal(hash(observerRaw), 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729');
assert.equal(Object.keys(JSON.parse(fs.readFileSync(path.join(packet, 'RUNTIME-PROTOCOL.json'), 'utf8')).program_top_level_helpers).join(','), needed.join(','));
let observations = 0;
walk(initialAst, node => {
  if (node.type === 'CallExpression' && node.callee.name === 'sewObserve') {
    observations += 1;
    assert.equal(node.arguments.length, 5);
    assert.deepEqual(node.arguments.map(argument => argument.type === 'Literal' ? argument.value : argument.name), ['sewController','sewApp','initial-owned-sew-state','sewMark','nodeRepl']);
  }
});
assert.equal(observations, 1);
const markerLine = top.get('sewMark').loc.start.line;
assert.ok(markerLine > top.get('sewFs').loc.start.line && markerLine > top.get('sewRoot').loc.start.line && markerLine > top.get('sewProcess').loc.start.line && markerLine > top.get('sewController').loc.start.line);
assert.ok(top.get('sewProofId').loc.start.line < top.get('sewController').loc.start.line, 'Constructor must be inside the counted proof envelope');
const binding = JSON.parse(read('ROOT-BINDING-DRAFT.json'));
assert.deepEqual(Object.keys(binding).sort(), Object.keys(JSON.parse(fs.readFileSync(path.join(packet, 'ROOT-BINDING.SEW-S2.template.json'), 'utf8'))).sort());
assert.equal(binding.case, 'SEW-S2');
const depTemplate = JSON.parse(fs.readFileSync(path.join(packet,'ROOT-BINDING.SEW-S2.template.json'),'utf8')).accepted_input_dependency;
assert.deepEqual(Object.keys(binding.accepted_input_dependency).sort(),Object.keys(depTemplate).sort());
assert.equal(binding.accepted_input_dependency.attempt,'S1-001');
assert.equal(binding.accepted_input_dependency.root_binding_sha256,'80a45a09e7439792eddfc31be7000dfe9033760fad11a1baa8c00e4fe02ea807');
assert.equal(binding.accepted_input_dependency.snapshot_sha256,'6b3c61c9f42c3bcc2c8be75008b69eec44038d9de94ac21d3a5fb9ec07a4cfab');
assert.equal(binding.accepted_input_dependency.saved_grade_report_sha256,'174cb2e6540edf4934a2211ef775802d50857951cd1b3de5ac0a190527f8512c');
assert.equal(binding.accepted_input_dependency.root_native_ui_review_sha256,'37a7a2d5858aac346583c52952da14f2157cba92d0f6247839ae1ca03a6789e5');
const acceptedRun = path.dirname(binding.accepted_input_dependency.root_binding_path);
assert.equal(acceptedRun,path.join(repo,'.local/native-cut-and-sew-20261116/S1-001'));
for(const [file,key] of [['ROOT-BINDING.json','root_binding_sha256'],['SAVED-GRADE-001.json','saved_grade_report_sha256'],['ROOT-UI-ACCEPTANCE.json','root_native_ui_review_sha256']]) assert.equal(hash(fs.readFileSync(path.join(acceptedRun,file))),binding.accepted_input_dependency[key]);
const acceptedGrade = JSON.parse(fs.readFileSync(path.join(acceptedRun,'SAVED-GRADE-001.json'),'utf8'));
const acceptedUI = JSON.parse(fs.readFileSync(path.join(acceptedRun,'ROOT-UI-ACCEPTANCE.json'),'utf8'));
assert.equal(acceptedGrade.status,'passed_saved_files_only');
assert.equal(acceptedUI.accepted,true);
assert.equal(acceptedUI.reviewed_raw_images_and_traces,true);
assert.equal(acceptedGrade.root_binding_sha256,binding.accepted_input_dependency.root_binding_sha256);
assert.equal(acceptedUI.root_binding_sha256,binding.accepted_input_dependency.root_binding_sha256);
assert.equal(acceptedGrade.attempt,'S1-001');
assert.equal(acceptedUI.attempt,'S1-001');
assert.equal(binding.root_authorized_for_case, false);
assert.equal(binding.native_lease_granted, false);
assert.equal(binding.actor_filename, path.join(root, 'ACTOR-BOOTSTRAP.js'));
assert.equal(binding.actor_sha256, hash(read('ACTOR-BOOTSTRAP.js')));
assert.equal(binding.actor_persistence_check.sha256, hash(read('ACTOR-PERSISTENCE-CHECK.js')));
assert.equal(binding.actor_initial_capture.sha256, hash(initialRaw));
assert.deepEqual(Object.keys(binding.controller_configuration).sort(), ['baselineCalls','callCap','elapsedCapS','evidenceKind','recorderPath','root']);
assert.deepEqual(binding.input_argv,[path.join(root,'inputs/accepted-redo-r4.weftforge')]);
assert.deepEqual(binding.inputs,{'inputs/accepted-redo-r4.weftforge':binding.accepted_input_dependency.snapshot_sha256});
assert.equal(fs.statSync(binding.input_argv[0]).mode & 0o777,0o444);
assert.equal(hash(fs.readFileSync(binding.input_argv[0])),binding.accepted_input_dependency.snapshot_sha256);
assert.ok(fs.readFileSync(binding.input_argv[0]).equals(fs.readFileSync(path.join(acceptedRun,'snapshots/redo-r4.weftforge'))));
const fixtureHash = 'f'.repeat(64);
const artifact = {file:'captures/binding-raw.json',bytes:17,sha256:fixtureHash,source_record:'0001',unedited:true,utc:'PROSPECTIVE_STATIC_PLACEHOLDER',monotonic_ns:'0'};
const env = {sewBinding:binding,sewBindingSha:fixtureHash,sewProofId:'0002',sewProofSha:fixtureHash,sewOfflineReceipt:artifact,sewCheckBinding:binding,sewCheckOfflineReceipt:artifact,sewCheckRecorderPath:binding.recorder};
env.b = binding;
env.expectedCommand = [binding.native_binary_path,...binding.input_argv].join(' ');
const value = node => {
  if (node.type === 'Literal') return node.value;
  if (node.type === 'Identifier') { assert.ok(node.name in env, 'Unknown static identifier: ' + node.name); return env[node.name]; }
  if (node.type === 'MemberExpression') return value(node.object)[node.computed ? value(node.property) : node.property.name];
  if (node.type === 'ObjectExpression') return Object.fromEntries(node.properties.map(property => [property.key.name || property.key.value, value(property.value)]));
  if (node.type === 'ArrayExpression') return node.elements.map(value);
  if (node.type === 'CallExpression' && node.callee.name === 'sewCheckHash') return fixtureHash;
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
  if (owner === 'Object' && method === 'assign' && node.arguments[0].name === 'sewProof') proofFill = value(node.arguments[1]);
  if (owner === 'sewRecorder' && method === 'returned' && node.arguments[0].name === 'sewProofId' && node.arguments[1].type === 'ObjectExpression') proofResult = value(node.arguments[1]);
  if (owner === 'sewOffline' && method === 'returned') offlineResult = value(node.arguments[1]);
  if (owner === 'sewOffline' && method === 'artifact') offlineArtifactCall = node.arguments[0].value === 'binding-raw.json' && node.arguments[1].name === 'sewBindingBytes' && node.arguments[2].name === 'sewOfflineId';
});
walk(checkAst, node => {
  if (node.type === 'VariableDeclarator' && node.id.name === 'sewCheckTypes') checkTypes = node.init.properties.map(property => property.key.name);
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression' && node.callee.object.name === 'sewCheckRecorder' && node.callee.property.name === 'before') checkMethod = node.arguments[1].value === 'local-runtime-bindings-check';
  if (node.type !== 'AssignmentExpression' || node.left.type !== 'Identifier') return;
  if (node.left.name === 'sewCheckTypes') checkTypes = node.right.properties.map(property => property.key.name);
  if (node.left.name === 'sewCheckFlags') checkFlags = node.right.properties.map(property => property.key.name);
  if (node.left.name === 'sewCheckContextExpected') checkContext = value(node.right);
  if (node.left.name === 'sewCheckProofExpected') checkProofExpected = value(node.right);
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
    if (node.callee.object.name === 'Object' && node.callee.property.name === 'assign' && node.arguments[0].name === 'sewCheckResult') {
      resultFillCount += 1;
      for (const property of node.arguments[1].properties) assert.ok(resultKeys.has(property.key.name), 'Unknown closed readiness field');
    }
    if (node.callee.object.name === 'sewCheckRecorder' && node.callee.property.name === 'returned') identicalReadinessReturned = node.arguments[0].name === 'sewCheckId' && node.arguments[1].name === 'sewCheckResult' && node.arguments[2].name === 'sewCheckFailure';
    assert.ok(node.callee.property.name !== 'createController', 'No independent-check constructor');
  }
  if (node.type === 'AssignmentExpression' && node.left.type === 'MemberExpression' && node.left.object.name === 'sewCheckResult') assert.ok(resultKeys.has(node.left.property.name), 'Unknown readiness assignment');
});
assert.equal(resultFillCount, 2);
assert.ok(identicalReadinessReturned);
assert.equal(readinessTemplate.schema, 'weftforge.straight-cut-and-sew.actor-readiness.v1');
const template = JSON.parse(fs.readFileSync(path.join(packet, 'ACTOR-RUNTIME-PROOF.template.json'), 'utf8'));
const plannedProof = Object.assign(template, proofFill);
const expectedProof = {schema:'weftforge.straight-cut.actor-runtime-proof.v1',case:'SEW-S2',attempt:'S2-001',root_binding_sha256:fixtureHash,evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:'0002',actor_filename:binding.actor_filename,actor_sha256:binding.actor_sha256,controller_configuration:binding.controller_configuration,observer_explicit_arguments:5,marker_initialized_synchronous_before_observer:true,controller_sha256:binding.controller_sha256,recorder_sha256:binding.recorder_sha256,observer_sha256:binding.observer_sha256,offline_receipt:artifact};
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
assert.deepEqual(plannedBefore['external.open'], {executable:binding.launcher_path,argv:binding.launcher_argv});
assert.deepEqual(plannedBefore['external.pid'], {expected_binary:binding.native_binary_path,expected_command:env.expectedCommand});
let exactCommandMatch = false;
let pidResultKeys = null;
walk(launchAst, node => {
  if (node.type === 'BinaryExpression' && node.operator === '===' && node.left.type === 'MemberExpression' && node.left.object.name === 'm' && node.left.property.value === 2 && node.right.type === 'Identifier' && node.right.name === 'expectedCommand') exactCommandMatch = true;
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression' && node.callee.object.name === 'r' && node.callee.property.name === 'returned' && node.arguments[1].type === 'ObjectExpression' && node.arguments[1].properties.some(property => property.key.name === 'exact_matches')) pidResultKeys = node.arguments[1].properties.map(property => property.key.name);
});
assert.ok(exactCommandMatch, 'PID control must retain exact full command match');
let expectedCommandIncludesInput = false;
walk(launchAst,node=>{if(node.type==='VariableDeclarator' && node.id.name==='expectedCommand' && node.init.type==='CallExpression' && node.init.callee.type==='MemberExpression' && node.init.callee.property.name==='join' && node.init.arguments[0].value===' ' && node.init.callee.object.type==='ArrayExpression') { const values=node.init.callee.object.elements; expectedCommandIncludesInput=values.length===2 && values[0].object.name==='b' && values[0].property.name==='native_binary_path' && values[1].type==='SpreadElement' && values[1].argument.object.name==='b' && values[1].argument.property.name==='input_argv'; }});
assert.ok(expectedCommandIncludesInput,'S2 PID command must include exact accepted input argv');
assert.deepEqual(pidResultKeys, ['status','exact_matches','stderr','executable','argv']);
let normalizedDescriptor = false;
let normalizedHashComparison = false;
walk(postflightAst, node => {
  if (node.type === 'VariableDeclarator' && node.id.name === 'expected' && node.init.type === 'ConditionalExpression' && node.init.test.type === 'BinaryExpression' && node.init.test.left.type === 'UnaryExpression' && node.init.test.left.operator === 'typeof' && node.init.test.left.argument.name === 'descriptor' && node.init.consequent.type === 'ObjectExpression' && node.init.consequent.properties[0].key.name === 'sha256' && node.init.alternate.name === 'descriptor') normalizedDescriptor = true;
  if (node.type === 'BinaryExpression' && node.operator === '===' && node.left.type === 'MemberExpression' && node.left.object.name === 'actual' && node.left.property.name === 'sha256' && node.right.type === 'MemberExpression' && node.right.object.name === 'expected' && node.right.property.name === 'sha256') normalizedHashComparison = true;
  if (node.type === 'CallExpression' && node.callee.type === 'MemberExpression') assert.notEqual(node.callee.property.name, 'createController', 'Post-summary raw guard cannot construct a controller');
});
assert.ok(normalizedDescriptor && normalizedHashComparison, 'Postflight must normalize nested preparation descriptors');
const identity = JSON.parse(fs.readFileSync(path.join(packet, 'SOURCE-74.json')));
assert.equal(binding.source_input_count, 74);
assert.deepEqual(binding.source_inputs, identity.source_inputs);
assert.equal(binding.source_commit, identity.source_commit);
assert.equal(binding.clean_metadata_commit, identity.clean_metadata_commit);
assert.equal(binding.native_binary_sha256, identity.native_binary_sha256);
assert.equal(binding.clean_source_metadata_receipt.sha256, identity.clean_checks.sha256);
assert.equal(binding.controller_configuration.callCap, 75);
assert.equal(binding.controller_configuration.elapsedCapS, 900);
assert.equal(binding.controller_configuration.baselineCalls, 0);
assert.deepEqual(binding.launcher_argv, ['-n',binding.bundle,'--args',...binding.input_argv]);
for (const absent of ['ROOT-BINDING.json','LAUNCH-CLOCK.json','CONTEXT.json','ACTOR-RUNTIME-PROOF.json','ACTOR-PERSISTENCE-CHECK-RESULT.json','SAVED-GRADE-ATTEMPT.json']) assert.ok(!fs.existsSync(path.join(root, absent)), 'Runtime artifact must not exist during preparation: ' + absent);
const postflightBindingKeys = new Set();
const postflightDependencyKeys = new Set();
let postflightDependencyAlias = false;
walk(postflightAst, node => {
  if (node.type === 'VariableDeclarator' && node.id.name === 'accepted') {
    assert.equal(node.init.type, 'MemberExpression');
    assert.equal(node.init.object.name, 'bound');
    assert.equal(node.init.property.name, 'accepted_input_dependency');
    postflightDependencyAlias = true;
  }
  if (node.type !== 'MemberExpression' || node.object.type !== 'Identifier') return;
  const field = !node.computed ? node.property.name : node.property.type === 'Literal' ? node.property.value : null;
  if (field === null) return;
  if (node.object.name === 'bound') {
    assert.ok(Object.hasOwn(binding, field), 'Unknown postflight bound property: ' + field);
    postflightBindingKeys.add(field);
  }
  if (node.object.name === 'accepted') {
    assert.ok(Object.hasOwn(binding.accepted_input_dependency, field), 'Unknown accepted dependency property: ' + field);
    postflightDependencyKeys.add(field);
  }
});
assert.ok(postflightDependencyAlias);
assert.ok(!read('ACTOR-POSTFLIGHT-RAW.cjs').toString().includes('accepted_h1_dependency'));
for (const name of ['controller','recorder','observer']) {
  assert.equal(typeof binding[name], 'string');
  assert.equal(typeof binding[name + '_sha256'], 'string');
  assert.equal(hash(fs.readFileSync(binding[name])), binding[name + '_sha256']);
}
for (const name of ['actor_persistence_check','actor_initial_capture']) {
  assert.deepEqual(Object.keys(binding[name]).sort(), ['path','sha256']);
  assert.equal(hash(fs.readFileSync(binding[name].path)), binding[name].sha256);
}
const closedDraft = (actual, shape) => {
  if (shape && typeof shape === 'object' && !Array.isArray(shape)) {
    assert.ok(actual && typeof actual === 'object' && !Array.isArray(actual));
    assert.deepEqual(Object.keys(actual).sort(), Object.keys(shape).sort());
    for (const key of Object.keys(shape)) closedDraft(actual[key], shape[key]);
  } else assert.notEqual(actual, null, 'Unfilled draft field');
};
closedDraft(binding, JSON.parse(fs.readFileSync(path.join(packet, 'ROOT-BINDING.SEW-S2.template.json'), 'utf8')));
const expectedDependency = {"case":"SEW-S1","attempt":"S1-001","snapshot_label":"redo-r4","root_binding_path":"<WORKSPACE>/.local/native-cut-and-sew-20261116/S1-001/ROOT-BINDING.json","root_binding_sha256":"80a45a09e7439792eddfc31be7000dfe9033760fad11a1baa8c00e4fe02ea807","snapshot_sha256":"6b3c61c9f42c3bcc2c8be75008b69eec44038d9de94ac21d3a5fb9ec07a4cfab","saved_grade_report_sha256":"174cb2e6540edf4934a2211ef775802d50857951cd1b3de5ac0a190527f8512c","root_native_ui_review_sha256":"37a7a2d5858aac346583c52952da14f2157cba92d0f6247839ae1ca03a6789e5"};
assert.deepEqual(binding.accepted_input_dependency, expectedDependency);
const acceptedBinding = JSON.parse(fs.readFileSync(path.join(acceptedRun,'ROOT-BINDING.json'),'utf8'));
assert.equal(acceptedBinding.case,'SEW-S1');
assert.equal(acceptedBinding.attempt,'S1-001');
for(const field of ['source_commit','clean_metadata_commit','source_inputs','source_input_count','native_binary_sha256']) assert.deepEqual(acceptedBinding[field],binding[field]);
assert.equal(acceptedBinding.clean_metadata_commit,'3858848afb3764014a5d305a0e5dfb5661ea6d86');
assert.equal(acceptedBinding.source_input_count,74);
assert.equal(acceptedBinding.freeze_sha256,binding.freeze_sha256);
assert.equal(acceptedGrade.freeze_sha256,binding.freeze_sha256);
assert.equal(acceptedGrade.case,'SEW-S1');
assert.equal(acceptedUI.case,'SEW-S1');
assert.equal(acceptedUI.single_saved_grade_report_sha256,expectedDependency.saved_grade_report_sha256);
assert.equal(acceptedUI.accepted_final_snapshot_label,'redo-r4');
assert.equal(acceptedUI.accepted_final_snapshot_sha256,expectedDependency.snapshot_sha256);
for(const [file,digest] of [['NATIVE-LEASE-CLOSED.json','eca65faa200f595f0ed17b4f8356ed5d86e53b0894b9fbe2c4805489fb6d3b45'],['POSTFLIGHT.json','e7873d6b795d971863ab7ce1893a242c23029804d6892985a30a1b35fda79c19'],['ACTOR-RUNTIME-TOOL-HISTORY.json','fd5fa2b3b04c5b72cefea53ef87103812c4d3f800be6be3d8bbc32cbd8f728e5']]) assert.equal(hash(fs.readFileSync(path.join(acceptedRun,file))),digest);
const samePacketGuards = ['clean_metadata_commit','source_input_count','source_inputs','freeze_sha256'];
const bootstrapRaw = read('ACTOR-BOOTSTRAP.js').toString();
for(const field of samePacketGuards) assert.ok(bootstrapRaw.includes('sewEntry.binding.'+field),'S2 bootstrap missing same-packet prior guard:'+field);
assert.ok(bootstrapRaw.includes('sewEntry.grade.freeze_sha256 !== sewBinding.freeze_sha256'));
const gateChecks = [];
for (const [key, names, descriptor] of [['runtime',['fmt','test','clippy','debug-build'],identity.runtime_checks],['clean_build',['xtask','debug-build'],identity.clean_checks]]) {
  assert.deepEqual(binding.integrated_local_gate_receipts[key], descriptor);
  assert.equal(hash(fs.readFileSync(descriptor.path)), descriptor.sha256);
  const gate = JSON.parse(fs.readFileSync(descriptor.path, 'utf8'));
  assert.deepEqual(gate.source_hashes,binding.source_inputs);
  assert.equal(gate.source_count,74);
  assert.equal(gate.all_passed,true);
  assert.equal(gate.checkout_status_after,'');
  for(const flag of ['checkout_clean_before','checkout_clean_after','source74_unchanged']) assert.equal(gate[flag],true);
  assert.deepEqual(gate.steps.map(step => step.name),names);
  assert.equal(gate.runtime_commit,binding.source_commit);
  for(const step of gate.steps) {
    assert.equal(step.exit_code,0);
    for(const stream of ['stdout','stderr']) assert.equal(hash(fs.readFileSync(path.join(path.dirname(descriptor.path), step.name + '.' + stream))), step[stream + '_sha256']);
  }
  if(key === 'runtime') {assert.equal(gate.target_directory,binding.own_target_directory);assert.equal(gate.cross_worktree_target_copy,false);}
  else {assert.equal(gate.checked_clean_commit,binding.clean_metadata_commit);assert.equal(gate.fresh_owned_default_target_no_cache_copy,true);}
  assert.equal(gate.binaries.weftforge.path,path.join(binding.own_target_directory,'debug/weftforge'));
  assert.equal(gate.binaries.weftforge.sha256,binding.native_binary_sha256);
  assert.equal(gate.binaries.weftforge.bytes,62245032);
  gateChecks.push(key);
}
const report = {kind:'passive Node Acorn AST and planned-schema comparison; not actual CUA proof',passed:true,actual_cua_runtime:false,staged_cells_executed:false,product_or_evaluator_executed:false,parser:'Node bundled Acorn ' + acorn.version,node:process.version,cross_cell_bindings:needed,all_cross_cell_declarations_program_top_level:true,no_nested_var_declarations:true,no_future_app_or_CUA_references_in_FS_cells:true,immutable_five_argument_observer:true,proof_context_result_and_offline_artifact_match_exact_V2_expected_schema:true,persistence_check_independent_imports_and_paired_method:'local-runtime-bindings-check',prospective_record:'0003',planned_getState_launch_PID_records_match_frozen_preflight_fields:true,PID_expected_binary_and_full_input_command_exact_match:true,accepted_S1_001_binding_once_grade_UI_and_readonly_R4_input_hashes_match:true,postflight_normalizes_hash_string_and_nested_object_descriptors:true,postflight_all_literal_bound_properties_exist:Array.from(postflightBindingKeys).sort(),postflight_all_accepted_dependency_properties_exist:Array.from(postflightDependencyKeys).sort(),postflight_dynamic_module_and_actor_descriptors_checked:true,integrated_gate_metadata_and_raw_log_hashes_match:gateChecks,source74_runtime_metadata_binary_and_closed_draft_fields_match:true,accepted_S1_same_clean385_full_named_source74_and_count_and_freeze_matches_current:true,prior_sole_grade_freeze_matches_current:true,prior_closed_lifecycle_hashes_verified_without_grading:true,planned_frozen_static_operations_including_mandatory_refresh:72,planned_paired_operations:71,mandatory_documentation_refresh_calls:1,planned_startup_operations:13,operation_cap:75,summary_calls:1,overall_call_cap:76,elapsed_cap_s:900};
fs.writeFileSync(path.join(root, 'PASSIVE-STATIC-CHECK.json'), JSON.stringify(report, null, 2) + '\n', {flag:'wx',mode:0o600});
console.log(JSON.stringify(report));
