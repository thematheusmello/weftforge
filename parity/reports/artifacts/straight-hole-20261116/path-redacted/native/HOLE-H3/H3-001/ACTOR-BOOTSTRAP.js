// Prospective actual-CUA cell. Execute only after root GO and the clocked standalone getState.
// Cross-cell bindings must be initialized at program top level for cua_repl persistence.
var holeModule = await import('node:module');
var holeRequire = holeModule.createRequire('<WORKSPACE>/.local/native-svg-export-successor-20261116/H3-001/ACTOR-BOOTSTRAP.js');
var holeFs = holeRequire('node:fs');
var holePath = holeRequire('node:path');
var holeCrypto = holeRequire('node:crypto');
var holeProcess = holeRequire('node:process');
var holeRoot = '<WORKSPACE>/.local/native-svg-export-successor-20261116/H3-001';
var holeRepo = '<WORKSPACE>';
var holePacket = holePath.join(holeRepo, '.local/svg-export-native-successor-plan-20261116');
var holeHash = bytes => holeCrypto.createHash('sha256').update(bytes).digest('hex');
var holeWrite = (file, value) => holeFs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', {flag:'wx',mode:0o600});
var holeBindingBytes = holeFs.readFileSync(holePath.join(holeRoot, 'ROOT-BINDING.json'));
var holeBindingSha = holeHash(holeBindingBytes);
var holeBinding = JSON.parse(holeBindingBytes);
if (holeBinding.root_authorized_for_case !== true || holeBinding.native_lease_granted !== true || holeBinding.case !== 'HOLE-H3' || holeBinding.attempt !== 'H3-001') throw new Error('No exact H3-001 root grant');
if (holeBinding.actor_filename !== holePath.join(holeRoot, 'ACTOR-BOOTSTRAP.js') || holeHash(holeFs.readFileSync(holeBinding.actor_filename)) !== holeBinding.actor_sha256) throw new Error('Complete bootstrap binding mismatch');
if (holeBinding.controller !== holePath.join(holeRepo, '.local/native-transform-controller-20261115/controller.cjs') || holeBinding.controller_sha256 !== '9debdc6b1d42b49dce16c8b1227df0621b579532ca7c5fd76c3a8a5db537790b' || holeHash(holeFs.readFileSync(holeBinding.controller)) !== holeBinding.controller_sha256) throw new Error('Controller mismatch');
if (holeBinding.recorder !== holePath.join(holeRepo, '.local/native-length-readouts-v2-20261115/recorder.cjs') || holeBinding.recorder_sha256 !== '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc' || holeHash(holeFs.readFileSync(holeBinding.recorder)) !== holeBinding.recorder_sha256) throw new Error('Recorder mismatch');
if (holeBinding.observer !== holePath.join(holeRepo, '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs') || holeBinding.observer_sha256 !== 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729' || holeHash(holeFs.readFileSync(holeBinding.observer)) !== holeBinding.observer_sha256) throw new Error('Observer mismatch');
var holeRecorder = holeRequire(holeBinding.recorder).createRecorder(holeRoot);
holeRecorder.returned('0001', {tool_returned:true,entrypoint:'mcp__cua_repl.js',code:'await cua.getState();',native_UI_calls:0,raw_result_retained_in_actual_tool_history:true});
var holeProofId = holeRecorder.before('actual-cua-runtime-filesystem-proof', 'local-filesystem-proof', {evidenceKind:'native',native_UI_calls:0});
var holeController = holeRequire(holeBinding.controller).createController(holeBinding.controller_configuration);
var holeObserve = holeRequire(holeBinding.observer).observe;
var holeMark = sourceRecord => {
  if (!holeFs.existsSync(holePath.join(holeRoot, 'trace', sourceRecord + '-after.json'))) throw new Error('Marker source is not returned');
  holeWrite(holePath.join(holeRoot, 'SUMMARY-LOWER-' + sourceRecord + '.json'), {kind:'conditional_marker_only_not_a_summary_attempt',source_record:sourceRecord,utc:new Date().toISOString(),monotonic_ns:holeProcess.hrtime.bigint().toString()});
};
var holeEntry = null;
var holeHistoricalKey = null;
var holeSentinelName = null;
var holeGateName = null;
var holeGate = null;
var holeGateStep = null;
var holeGateStream = null;
var holeListFiles = directory => holeFs.readdirSync(directory, {withFileTypes:true}).flatMap(entry => entry.isDirectory() ? holeListFiles(holePath.join(directory, entry.name)) : [holePath.relative(holePacket, holePath.join(directory, entry.name))]);
var holeClosedShape = (actual, shape) => shape && typeof shape === 'object' ? Array.isArray(shape) ? Array.isArray(actual) && actual.length === shape.length && shape.every((entry,index) => holeClosedShape(actual[index], entry)) : actual && typeof actual === 'object' && !Array.isArray(actual) && Object.keys(actual).sort().join(',') === Object.keys(shape).sort().join(',') && Object.keys(shape).every(key => holeClosedShape(actual[key], shape[key])) : actual !== null && actual !== undefined;
var holeFreezeBytes = null;
var holeFreeze = null;
var holeOffline = null;
var holeOfflineId = null;
var holeOfflineReceipt = null;
var holeProof = null;
var holeProofSha = null;
try {
  if (!holeClosedShape(holeBinding, JSON.parse(holeFs.readFileSync(holePath.join(holePacket, 'ROOT-BINDING.HOLE-H3.template.json'), 'utf8')))) throw new Error('Incomplete or extra final binding fields');
  if (holeProofId !== '0002') throw new Error('Filesystem proof must be record0002');
  holeFreezeBytes = holeFs.readFileSync(holePath.join(holePacket, 'FREEZE.json'));
  if (holeHash(holeFreezeBytes) !== '64077323f7f1ac3998f4e6763099c0bbf04e8dd1be710a3900ed8a4c59c07c55' || holeBinding.freeze_sha256 !== holeHash(holeFreezeBytes)) throw new Error('Frozen Hole packet mismatch');
  holeFreeze = JSON.parse(holeFreezeBytes);
  if (Object.keys(holeFreeze.files).length !== 48 || Object.keys(holeFreeze.external_dependencies).length !== 110 || holeListFiles(holePacket).filter(name => name !== 'FREEZE.json').sort().join(',') !== Object.keys(holeFreeze.files).sort().join(',')) throw new Error('Complete frozen packet coverage differs');
  if (holeHash(holeFs.readFileSync(holePath.join(holeRepo, '.local/svg-export-native-successor-root-review-20261116/FREEZE-001.json'))) !== 'a1f93ba3556cf7f6b964b392a82cbe5176b712dd3f13a3af3f06aab531cf7167') throw new Error('Root packet-freeze receipt differs');
  for (holeEntry of Object.entries(holeFreeze.files)) if (holeHash(holeFs.readFileSync(holePath.join(holePacket, holeEntry[0]))) !== holeEntry[1]) throw new Error('Frozen packet artifact mismatch: ' + holeEntry[0]);
  for (holeEntry of Object.entries(holeFreeze.external_dependencies)) if (holeHash(holeFs.readFileSync(holePath.join(holeRepo, holeEntry[0]))) !== holeEntry[1]) throw new Error('Frozen external artifact mismatch: ' + holeEntry[0]);
  holeEntry = JSON.parse(holeFs.readFileSync(holePath.join(holePacket, 'SOURCE-74.json'), 'utf8'));
  if (holeBinding.source_commit !== holeEntry.source_commit || holeBinding.clean_metadata_commit !== holeEntry.clean_metadata_commit || Object.keys(holeBinding.source_inputs).sort().join(',') !== Object.keys(holeEntry.source_inputs).sort().join(',') || !Object.keys(holeEntry.source_inputs).every(key => holeBinding.source_inputs[key] === holeEntry.source_inputs[key]) || holeBinding.source_worktree !== holePath.join(holeRepo, '.local/svg-export-fix-20261116') || holeBinding.own_target_directory !== holePath.join(holeBinding.source_worktree, 'target') || JSON.stringify(holeBinding.integrated_local_gate_receipts.runtime) !== JSON.stringify(holeEntry.runtime_checks) || JSON.stringify(holeBinding.integrated_local_gate_receipts.clean_build) !== JSON.stringify(holeEntry.clean_checks) || JSON.stringify(holeBinding.clean_source_metadata_receipt) !== JSON.stringify(holeEntry.clean_checks)) throw new Error('Sealed source74/runtime/clean gate binding differs');
  if (holeBinding.source74_sha256 !== holeHash(holeFs.readFileSync(holePath.join(holePacket, 'SOURCE-74.json'))) || holeBinding.native_binary_sha256 !== holeEntry.native_binary_sha256 || holeBinding.native_binary_bytes !== holeEntry.native_binary_bytes) throw new Error('Sealed source74 artifact/native identity differs');
  if (holeBinding.source_input_count !== 74 || Object.keys(holeBinding.source_inputs).length !== 74) throw new Error('Source74 count mismatch');
  for (holeEntry of Object.entries(holeBinding.source_inputs)) if (holeHash(holeFs.readFileSync(holePath.join(holeBinding.source_worktree, holeEntry[0]))) !== holeEntry[1]) throw new Error('Source74 mismatch: ' + holeEntry[0]);
  holeEntry = holeFs.readFileSync(holePath.join(holeBinding.source_worktree, '.git'), 'utf8').trim();
  if (!holeEntry.startsWith('gitdir: ')) throw new Error('Worktree metadata pointer differs');
  holeEntry = {directory:holeEntry.slice(8)};
  holeEntry.head = holeFs.readFileSync(holePath.join(holeEntry.directory, 'HEAD'), 'utf8').trim();
  if (holeEntry.head.startsWith('ref: ')) holeEntry.head = holeFs.readFileSync(holePath.resolve(holeEntry.directory, holeFs.readFileSync(holePath.join(holeEntry.directory, 'commondir'), 'utf8').trim(), holeEntry.head.slice(5)), 'utf8').trim();
  if (holeEntry.head !== holeBinding.clean_metadata_commit) throw new Error('Clean metadata HEAD differs');
  for (holeGateName of ['runtime','clean_build']) {
    holeEntry = holeBinding.integrated_local_gate_receipts[holeGateName];
    if (holeHash(holeFs.readFileSync(holeEntry.path)) !== holeEntry.sha256) throw new Error('Integrated gate receipt mismatch');
    holeGate = JSON.parse(holeFs.readFileSync(holeEntry.path, 'utf8'));
    if (JSON.stringify(holeGate.source_hashes) !== JSON.stringify(holeBinding.source_inputs) || holeGate.source_count !== 74 || holeGate.runtime_commit !== holeBinding.source_commit || holeGate.all_passed !== true || holeGate.checkout_status_after !== '' || holeGate.checkout_clean_before !== true || holeGate.checkout_clean_after !== true || holeGate.source74_unchanged !== true || holeGate.steps.map(step => step.name).join(',') !== (holeGateName === 'runtime' ? 'fmt,test,clippy,debug-build' : 'xtask,debug-build')) throw new Error('Actual source74 gate fields differ');
    if (holeGateName === 'runtime' ? holeGate.target_directory !== holeBinding.own_target_directory || holeGate.cross_worktree_target_copy !== false : holeGate.checked_clean_commit !== holeBinding.clean_metadata_commit || holeGate.fresh_owned_default_target_no_cache_copy !== true) throw new Error('Actual own-worktree gate authority differs');
    if (holeGate.binaries.weftforge.path !== holePath.join(holeBinding.own_target_directory, 'debug/weftforge') || holeGate.binaries.weftforge.sha256 !== holeBinding.native_binary_sha256 || holeGate.binaries.weftforge.bytes !== holeBinding.native_binary_bytes) throw new Error('Actual build receipt binary descriptor differs');
    for (holeGateStep of holeGate.steps) {
      if (holeGateStep.exit_code !== 0) throw new Error('Integrated gate failed');
      for (holeGateStream of ['stdout','stderr']) if (holeHash(holeFs.readFileSync(holePath.join(holePath.dirname(holeEntry.path), holeGateStep.name + '.' + holeGateStream))) !== holeGateStep[holeGateStream + '_sha256']) throw new Error('Actual gate raw log changed');
    }
  }
  if (holeHash(holeFs.readFileSync(holeBinding.native_binary_path)) !== holeBinding.native_binary_sha256 || holeHash(holeFs.readFileSync(holePath.join(holeBinding.own_target_directory, 'debug/weftforge'))) !== holeBinding.native_binary_sha256) throw new Error('Owned native/build original mismatch');
  if (holeHash(holeFs.readFileSync(holePath.join(holeBinding.bundle, 'Contents/Info.plist'))) !== holeBinding.bundle_plist_sha256) throw new Error('Owned plist mismatch');
  holeEntry = JSON.parse(holeFs.readFileSync(holePath.join(holePacket, 'HISTORICAL-DEPENDENCY.json'), 'utf8'));
  if (JSON.stringify(holeBinding.accepted_h1_dependency) !== JSON.stringify(holeEntry)) throw new Error('Exact accepted historical H1-002 dependency differs');
  for (holeHistoricalKey of ['source73','root_binding','snapshot','saved_grade_report','root_native_ui_review']) if (holeHash(holeFs.readFileSync(holeEntry[holeHistoricalKey].path)) !== holeEntry[holeHistoricalKey].sha256) throw new Error('Historical dependency changed: ' + holeHistoricalKey);
  if (JSON.stringify(holeBinding.input_argv) !== JSON.stringify([holePath.join(holeRoot, 'inputs/accepted-h1-002-redo-r5.weftforge')]) || Object.keys(holeBinding.inputs).join(',') !== 'inputs/accepted-h1-002-redo-r5.weftforge' || holeBinding.inputs['inputs/accepted-h1-002-redo-r5.weftforge'] !== holeEntry.snapshot.sha256) throw new Error('H3-001 requires only the accepted actual historical R5 input');
  if ((holeFs.statSync(holeEntry.snapshot.path).mode & 0o222) !== 0 || (holeFs.statSync(holeBinding.input_argv[0]).mode & 0o222) !== 0 || holeHash(holeFs.readFileSync(holeBinding.input_argv[0])) !== holeEntry.snapshot.sha256 || !holeFs.readFileSync(holeBinding.input_argv[0]).equals(holeFs.readFileSync(holeEntry.snapshot.path))) throw new Error('Actual readonly accepted R5 bytes differ');
  if (holeFs.readdirSync(holePath.join(holeRoot, 'inputs')).sort().join(',') !== 'accepted-h1-002-redo-r5.weftforge' || holeFs.readdirSync(holePath.join(holeRoot, 'work')).length !== 0 || holeFs.readdirSync(holePath.join(holeRoot, 'exports')).sort().join(',') !== 'hole-r6.dxf,hole-r6.dxf.recovery') throw new Error('Fresh H3 input/work/export inventory differs');
  for (holeSentinelName of ['hole-r6.dxf','hole-r6.dxf.recovery']) if (!holeFs.readFileSync(holePath.join(holeRoot, 'exports', holeSentinelName)).equals(holeFs.readFileSync(holePath.join(holePacket, 'fixtures', holeSentinelName)))) throw new Error('Literal DXF sentinel differs');
  if (holeBinding.bundle !== holePath.join(holeRoot, 'WeftForgeSvgHoleH3001.app') || holeBinding.bundle_identifier !== 'org.weftforge.preview.svghole20261116.h3001' || holeBinding.native_binary_path !== holePath.join(holeBinding.bundle, 'Contents/MacOS/weftforge') || holeBinding.native_binary_bytes !== 62231992 || holeFs.statSync(holeBinding.native_binary_path).size !== holeBinding.native_binary_bytes || holeFs.statSync(holePath.join(holeBinding.own_target_directory, 'debug/weftforge')).size !== holeBinding.native_binary_bytes || holeBinding.launcher_path !== '/usr/bin/open' || JSON.stringify(holeBinding.launcher_argv) !== JSON.stringify(['-n',holeBinding.bundle,'--args',...holeBinding.input_argv])) throw new Error('Exact own bundle/binary/argv identity differs');
  if (Object.keys(holeBinding.controller_configuration).sort().join(',') !== 'baselineCalls,callCap,elapsedCapS,evidenceKind,recorderPath,root' || holeBinding.controller_configuration.root !== holeRoot || holeBinding.controller_configuration.recorderPath !== holeBinding.recorder || holeBinding.controller_configuration.evidenceKind !== 'native' || holeBinding.controller_configuration.callCap !== 100 || holeBinding.controller_configuration.elapsedCapS !== 1200 || holeBinding.controller_configuration.baselineCalls !== 0) throw new Error('Six-key controller configuration mismatch');
  for (holeEntry of [holeBinding.actor_persistence_check, holeBinding.actor_initial_capture]) if (!holeEntry || holeHash(holeFs.readFileSync(holeEntry.path)) !== holeEntry.sha256) throw new Error('Bound H3-001 orchestration artifact mismatch');
  holeOffline = holeRequire(holeBinding.recorder).createRecorder(holePath.join(holeRoot, 'offline-proof'));
  holeOfflineId = holeOffline.before('exact-root-binding-opaque-copy', 'opaque-binding-proof', {});
  holeOffline.returned(holeOfflineId, {raw_binding_sha256:holeBindingSha});
  holeOfflineReceipt = holeOffline.artifact('binding-raw.json', holeBindingBytes, holeOfflineId);
  if (holeOfflineId !== '0001' || holeOfflineReceipt.sha256 !== holeBindingSha || !holeFs.readFileSync(holePath.join(holeRoot, 'offline-proof', holeOfflineReceipt.file)).equals(holeBindingBytes)) throw new Error('Opaque binding proof failed');
  holeProof = JSON.parse(holeFs.readFileSync(holePath.join(holePacket, 'ACTOR-RUNTIME-PROOF.template.json'), 'utf8'));
  Object.assign(holeProof, {case:holeBinding.case,attempt:holeBinding.attempt,root_binding_sha256:holeBindingSha,evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:holeProofId,actor_filename:holeBinding.actor_filename,actor_sha256:holeBinding.actor_sha256,controller_configuration:holeBinding.controller_configuration,marker_initialized_synchronous_before_observer:true,offline_receipt:holeOfflineReceipt});
  holeWrite(holePath.join(holeRoot, 'ACTOR-RUNTIME-PROOF.json'), holeProof);
  holeProofSha = holeHash(holeFs.readFileSync(holePath.join(holeRoot, 'ACTOR-RUNTIME-PROOF.json')));
  holeRecorder.returned(holeProofId, {proof_sha256:holeProofSha,actual_cua_runtime:true,native_UI_calls:0});
  holeMark(holeProofId);
  nodeRepl.write({source_record:holeProofId,proof_sha256:holeProofSha,actual_cua_runtime:true,native_UI_calls:0,status:holeController.status()});
} catch (holeError) {
  if (!holeFs.existsSync(holePath.join(holeRoot, 'trace', holeProofId + '-after.json'))) holeRecorder.returned(holeProofId, null, holeError);
  holeWrite(holePath.join(holeRoot, 'ACTOR-FAILURE.json'), {phase:'actual-runtime-filesystem-proof',message:holeError.message,utc:new Date().toISOString(),monotonic_ns:holeProcess.hrtime.bigint().toString()});
  throw holeError;
}
