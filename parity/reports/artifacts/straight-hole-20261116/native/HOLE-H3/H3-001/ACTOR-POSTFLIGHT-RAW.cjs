'use strict';
// Prospective post-summary filesystem guard only. Do not run before the actual summary returns.
// Reads opaque snapshots only for hashes; never parses project construction or invokes an evaluator.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = __dirname;
const repo = path.resolve(root, '../../..');
const packet = path.join(repo, '.local/svg-export-native-successor-plan-20261116');
const parsed = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const hash = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const bound = parsed(path.join(root, 'ROOT-BINDING.json'));
const summary = parsed(path.join(root, 'COMPLETION-SUMMARY-RETURN.json'));
if (bound.case !== 'HOLE-H3' || bound.attempt !== 'H3-001' || summary.summary_calls !== 1 || summary.attempt !== bound.attempt || summary.root_binding_sha256 !== hash(path.join(root, 'ROOT-BINDING.json'))) throw Error('Actual completed summary/binding required');
const closed = parsed(path.join(root, 'NATIVE-LEASE-CLOSED.json'));
if (closed.native_input_lease_closed !== true || closed.case !== bound.case || closed.attempt !== bound.attempt || closed.root_binding_sha256 !== summary.root_binding_sha256 || closed.immediate_completion_summary_calls !== 1) throw Error('Root input closure required');
const pins = [];
const pin = (label, file, descriptor) => {
  const expected = typeof descriptor === 'string' ? {sha256:descriptor} : descriptor;
  if (!expected || !/^[0-9a-f]{64}$/.test(expected.sha256)) throw Error('Invalid identity descriptor: ' + label);
  const stat = fs.statSync(file);
  const actual = {sha256:hash(file),bytes:stat.size,mode:'0o' + (stat.mode & 0o777).toString(8)};
  const matches = actual.sha256 === expected.sha256 &&
    (expected.bytes === undefined || actual.bytes === expected.bytes) &&
    (expected.mode === undefined || actual.mode === expected.mode);
  pins.push({label,path:file,expected,actual,matches});
};
for (const [relative, digest] of Object.entries(bound.source_inputs)) pin('source74/' + relative, path.join(bound.source_worktree, relative), digest);
pin('own-native', bound.native_binary_path, bound.native_binary_sha256);
pin('source-default-target-native', path.join(bound.own_target_directory, 'debug/weftforge'), bound.native_binary_sha256);
pin('bundle-plist', path.join(bound.bundle, 'Contents/Info.plist'), bound.bundle_plist_sha256);
for (const [relative, digest] of Object.entries(bound.inputs)) pin('accepted-input/' + relative, path.join(root, relative), {sha256:digest,mode:'0o444'});
const accepted = bound.accepted_h1_dependency;
for (const key of ['source73','root_binding','snapshot','saved_grade_report','root_native_ui_review']) pin('accepted-H1/' + key, accepted[key].path, accepted[key]);
for (const name of ['controller','recorder','observer']) pin(name, bound[name], bound[name + '_sha256']);
pin('bootstrap', bound.actor_filename, bound.actor_sha256);
for (const name of ['actor_persistence_check','actor_initial_capture']) pin(name, bound[name].path, bound[name].sha256);
pin('packet-freeze', path.join(packet, 'FREEZE.json'), bound.freeze_sha256);
const freeze = parsed(path.join(packet, 'FREEZE.json'));
for (const [relative, descriptor] of Object.entries(freeze.files)) pin('packet/' + relative, path.join(packet, relative), descriptor);
for (const [relative, descriptor] of Object.entries(freeze.external_dependencies)) pin('external/' + relative, path.resolve(repo, relative), descriptor);
const preparation = parsed(path.join(root, 'PREPARATION-FREEZE.json'));
for (const [relative, descriptor] of Object.entries(preparation.files)) pin('preparation/' + relative, path.join(root, relative), descriptor);
const names = fs.readdirSync(path.join(root, 'trace')).filter(name => name.endsWith('-before.json')).sort();
const errors = [];
for (let index = 0; index < names.length; index++) {
  const id = String(index + 1).padStart(4, '0');
  const before = parsed(path.join(root, 'trace', names[index]));
  const after = parsed(path.join(root, 'trace', id + '-after.json'));
  if (before.id !== id || after.id !== id || after.status !== 'returned' || after.error !== null) errors.push(id);
}
const gitDirectory = fs.readFileSync(path.join(bound.source_worktree, '.git'), 'utf8').trim().slice(8);
const head = fs.readFileSync(path.join(gitDirectory, 'HEAD'), 'utf8').trim();
const resolvedHead = head.startsWith('ref: ') ? fs.readFileSync(path.resolve(gitDirectory, fs.readFileSync(path.join(gitDirectory, 'commondir'), 'utf8').trim(), head.slice(5)), 'utf8').trim() : head;
const report = {schema:'weftforge.native.actor-postflight-raw.v1',evidence_kind:'postcase_filesystem_identity_and_raw_trace_only',case:bound.case,attempt:bound.attempt,root_binding_sha256:summary.root_binding_sha256,pinned_nodes:pins,pinned_node_count:pins.length,all_pinned_nodes_unchanged:pins.every(entry => entry.matches),source_input_count:Object.keys(bound.source_inputs).length,source_inputs_unchanged:pins.filter(entry => entry.label.startsWith('source74/')).every(entry => entry.matches),native_binary_unchanged:pins.filter(entry => ['own-native','source-default-target-native'].includes(entry.label)).every(entry => entry.matches),resolved_source_head:resolvedHead,source_head_unchanged:resolvedHead === bound.clean_metadata_commit,paired_native_external_operations:names.length,trace_error_records:errors,documentation_refresh_calls:summary.documentation_refresh_calls,summary_calls:summary.summary_calls,overall_actual_calls:summary.overall_actual_calls,controller_context_sha256:hash(path.join(root,'CONTEXT.json')),runtime_proof_sha256:hash(path.join(root,'ACTOR-RUNTIME-PROOF.json')),actor_readiness_sha256:hash(path.join(root,'ACTOR-PERSISTENCE-CHECK-RESULT.json')),completion_summary_return_sha256:hash(path.join(root,'COMPLETION-SUMMARY-RETURN.json')),native_exit_independently_verified:false,formal_grading_performed:false};
fs.writeFileSync(path.join(root, 'ACTOR-POSTFLIGHT-RAW.json'), JSON.stringify(report, null, 2) + '\n', {flag:'wx',mode:0o444});
if (!report.all_pinned_nodes_unchanged || !report.source_head_unchanged || errors.length) throw Error('Postcase raw identity/trace guard differs; preserved report requires root review');
console.log(JSON.stringify({file:'ACTOR-POSTFLIGHT-RAW.json',sha256:hash(path.join(root,'ACTOR-POSTFLIGHT-RAW.json')),pinned_node_count:pins.length,all_pinned_nodes_unchanged:report.all_pinned_nodes_unchanged,source_head_unchanged:report.source_head_unchanged,trace_error_records:errors}));
