// Prospective second actual-CUA cell. One counted FS check, before any external launch.
// Fresh local imports/recorder preserve a truthful result even if bootstrap exports are missing.
var holeCheckModule = await import('node:module');
var holeCheckRequire = holeCheckModule.createRequire('<WORKSPACE>/.local/native-svg-export-successor-20261116/H3-001/ACTOR-PERSISTENCE-CHECK.js');
var holeCheckFs = holeCheckRequire('node:fs');
var holeCheckPath = holeCheckRequire('node:path');
var holeCheckCrypto = holeCheckRequire('node:crypto');
var holeCheckProcess = holeCheckRequire('node:process');
var holeCheckRoot = '<WORKSPACE>/.local/native-svg-export-successor-20261116/H3-001';
var holeCheckHash = bytes => holeCheckCrypto.createHash('sha256').update(bytes).digest('hex');
var holeCheckWrite = (file, value) => holeCheckFs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', {flag:'wx',mode:0o600});
var holeCheckRecorderPath = '<WORKSPACE>/.local/native-length-readouts-v2-20261115/recorder.cjs';
if (holeCheckHash(holeCheckFs.readFileSync(holeCheckRecorderPath)) !== '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc') throw new Error('Independent recorder mismatch');
var holeCheckRecorder = holeCheckRequire(holeCheckRecorderPath).createRecorder(holeCheckRoot);
var holeCheckId = holeCheckRecorder.before('actual-cross-cell-runtime-bindings', 'local-runtime-bindings-check', {evidenceKind:'native',native_UI_calls:0,expected_source_record:'0003'});
var holeCheckBindingBytes = null;
var holeCheckBinding = null;
var holeCheckTypes = {holeModule:typeof holeModule,holeRequire:typeof holeRequire,holeFs:typeof holeFs,holePath:typeof holePath,holeCrypto:typeof holeCrypto,holeProcess:typeof holeProcess,holeRoot:typeof holeRoot,holeRepo:typeof holeRepo,holePacket:typeof holePacket,holeHash:typeof holeHash,holeWrite:typeof holeWrite,holeBindingBytes:typeof holeBindingBytes,holeBindingSha:typeof holeBindingSha,holeBinding:typeof holeBinding,holeRecorder:typeof holeRecorder,holeController:typeof holeController,holeObserve:typeof holeObserve,holeMark:typeof holeMark,holeProofId:typeof holeProofId,holeProofSha:typeof holeProofSha};
var holeCheckFlags = null;
var holeCheckContext = null;
var holeCheckContextExpected = null;
var holeCheckProofBytes = null;
var holeCheckProof = null;
var holeCheckProofExpected = null;
var holeCheckProofResult = null;
var holeCheckOfflineReceipt = null;
var holeCheckOfflineResult = null;
var holeCheckStatus = null;
var holeCheckResult = JSON.parse(holeCheckFs.readFileSync('<WORKSPACE>/.local/svg-export-native-successor-plan-20261116/ACTOR-PERSISTENCE-CHECK-RESULT.template.json', 'utf8'));
var holeCheckFailure = null;
holeCheckResult.helper_types = holeCheckTypes;
holeCheckResult.actual_cua_runtime = true;
holeCheckResult.native_UI_calls = 0;
holeCheckResult.source_record = holeCheckId;
try {
  holeCheckBindingBytes = holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'ROOT-BINDING.json'));
  holeCheckBinding = JSON.parse(holeCheckBindingBytes);
  Object.assign(holeCheckResult, {case:holeCheckBinding.case,attempt:holeCheckBinding.attempt,root_binding_sha256:holeCheckHash(holeCheckBindingBytes)});
  holeCheckFlags = {
    holeModule:typeof holeModule === 'object' && holeModule !== null && typeof holeModule.createRequire === 'function',
    holeRequire:typeof holeRequire === 'function',
    holeFs:typeof holeFs === 'object' && holeFs === holeCheckFs,
    holePath:typeof holePath === 'object' && holePath === holeCheckRequire('node:path'),
    holeCrypto:typeof holeCrypto === 'object' && holeCrypto === holeCheckCrypto,
    holeProcess:typeof holeProcess === 'object' && holeProcess === holeCheckProcess,
    holeRoot:typeof holeRoot === 'string' && holeRoot === holeCheckRoot,
    holeRepo:typeof holeRepo === 'string' && holeRepo === '<WORKSPACE>',
    holePacket:typeof holePacket === 'string' && holePacket === '<WORKSPACE>/.local/svg-export-native-successor-plan-20261116',
    holeHash:typeof holeHash === 'function' && holeHash(holeCheckBindingBytes) === holeCheckHash(holeCheckBindingBytes),
    holeWrite:typeof holeWrite === 'function',
    holeBindingBytes:typeof holeBindingBytes === 'object' && holeBindingBytes !== null && typeof holeBindingBytes.equals === 'function' && holeBindingBytes.equals(holeCheckBindingBytes),
    holeBindingSha:typeof holeBindingSha === 'string' && holeBindingSha === holeCheckHash(holeCheckBindingBytes),
    holeBinding:typeof holeBinding === 'object' && holeBinding !== null && JSON.stringify(holeBinding) === JSON.stringify(holeCheckBinding),
    holeRecorder:typeof holeRecorder === 'object' && holeRecorder !== null && typeof holeRecorder.before === 'function' && typeof holeRecorder.returned === 'function' && typeof holeRecorder.artifact === 'function',
    holeController:typeof holeController === 'object' && holeController !== null && typeof holeController.status === 'function' && typeof holeController.call === 'function' && typeof holeController.capture === 'function',
    holeObserve:typeof holeObserve === 'function' && holeObserve === holeCheckRequire(holeCheckBinding.observer).observe,
    holeMark:typeof holeMark === 'function',
    holeProofId:typeof holeProofId === 'string' && holeProofId === '0002',
    holeProofSha:typeof holeProofSha === 'string' && holeProofSha === holeCheckHash(holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'ACTOR-RUNTIME-PROOF.json')))
  };
  holeCheckContext = JSON.parse(holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'CONTEXT.json'), 'utf8'));
  holeCheckContextExpected = {evidenceKind:'native',callCap:100,elapsedCapS:1200,baselineCalls:0,recorderPath:holeCheckRecorderPath,recorderSha256:'45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc'};
  holeCheckProofBytes = holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'ACTOR-RUNTIME-PROOF.json'));
  holeCheckProof = JSON.parse(holeCheckProofBytes);
  holeCheckOfflineReceipt = JSON.parse(holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'offline-proof/captures/binding-raw.json.sha256.json'), 'utf8'));
  holeCheckProofExpected = {schema:'weftforge.straight-cut.actor-runtime-proof.v1',case:holeCheckBinding.case,attempt:holeCheckBinding.attempt,root_binding_sha256:holeCheckHash(holeCheckBindingBytes),evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:'0002',actor_filename:holeCheckBinding.actor_filename,actor_sha256:holeCheckBinding.actor_sha256,controller_configuration:holeCheckBinding.controller_configuration,observer_explicit_arguments:5,marker_initialized_synchronous_before_observer:true,controller_sha256:holeCheckBinding.controller_sha256,recorder_sha256:holeCheckBinding.recorder_sha256,observer_sha256:holeCheckBinding.observer_sha256,offline_receipt:holeCheckOfflineReceipt};
  holeCheckProofResult = JSON.parse(holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'trace/0002-after.json'), 'utf8'));
  holeCheckOfflineResult = JSON.parse(holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'offline-proof/trace/0001-after.json'), 'utf8'));
  holeCheckStatus = holeCheckFlags.holeController ? holeController.status() : null;
  Object.assign(holeCheckResult, {
    source_record:holeCheckId,actual_cua_runtime:true,native_UI_calls:0,helper_types:holeCheckTypes,helper_flags:holeCheckFlags,helpers_ready:Object.values(holeCheckFlags).every(Boolean),
    root:{expected:holeCheckRoot,actual:typeof holeRoot === 'string' ? holeRoot : null},
    controller_configuration:{expected:holeCheckBinding.controller_configuration,actual:holeCheckFlags.holeBinding ? holeBinding.controller_configuration : null},
    context_schema_ready:JSON.stringify(holeCheckContext) === JSON.stringify(holeCheckContextExpected),
    proof_schema_ready:Object.keys(holeCheckProof).sort().join(',') === Object.keys(holeCheckProofExpected).sort().join(',') && Object.keys(holeCheckProofExpected).every(key => JSON.stringify(holeCheckProof[key]) === JSON.stringify(holeCheckProofExpected[key])),
    proof_result_ready:holeCheckProofResult.status === 'returned' && JSON.stringify(holeCheckProofResult.result) === JSON.stringify({proof_sha256:holeCheckHash(holeCheckProofBytes),actual_cua_runtime:true,native_UI_calls:0}),
    offline_artifact_ready:holeCheckOfflineReceipt.file === 'captures/binding-raw.json' && holeCheckOfflineReceipt.source_record === '0001' && holeCheckOfflineReceipt.unedited === true && holeCheckOfflineReceipt.bytes === holeCheckBindingBytes.length && holeCheckOfflineReceipt.sha256 === holeCheckHash(holeCheckBindingBytes) && holeCheckFs.readFileSync(holeCheckPath.join(holeCheckRoot, 'offline-proof/captures/binding-raw.json')).equals(holeCheckBindingBytes) && holeCheckOfflineResult.status === 'returned' && JSON.stringify(holeCheckOfflineResult.result) === JSON.stringify({raw_binding_sha256:holeCheckHash(holeCheckBindingBytes)}),
    controller_status_ready:holeCheckStatus !== null && holeCheckStatus.evidenceKind === 'native' && holeCheckStatus.calls === 3 && holeCheckStatus.stopped === false && Number.isFinite(holeCheckStatus.elapsedS) && holeCheckStatus.elapsedS >= 0 && holeCheckStatus.elapsedS < 1200,
    proof_sha256:holeCheckHash(holeCheckProofBytes)
  });
  if (holeCheckId !== '0003' || holeCheckBinding.root_authorized_for_case !== true || holeCheckBinding.native_lease_granted !== true || holeCheckBinding.case !== 'HOLE-H3' || holeCheckBinding.attempt !== 'H3-001' || !holeCheckResult.helpers_ready || !holeCheckResult.context_schema_ready || !holeCheckResult.proof_schema_ready || !holeCheckResult.proof_result_ready || !holeCheckResult.offline_artifact_ready || !holeCheckResult.controller_status_ready) holeCheckFailure = new Error('Cross-cell persistence/schema check failed; no launch permitted');
} catch (holeCheckError) {
  holeCheckFailure = holeCheckError;
  holeCheckResult.helper_types = holeCheckTypes;
  if (holeCheckFlags !== null) holeCheckResult.helper_flags = holeCheckFlags;
  holeCheckResult.helpers_ready = false;
}
holeCheckRecorder.returned(holeCheckId, holeCheckResult, holeCheckFailure);
holeCheckWrite(holeCheckPath.join(holeCheckRoot, 'ACTOR-PERSISTENCE-CHECK-RESULT.json'), holeCheckResult);
holeCheckWrite(holeCheckPath.join(holeCheckRoot, 'SUMMARY-LOWER-' + holeCheckId + '.json'), {kind:'conditional_marker_only_not_a_summary_attempt',source_record:holeCheckId,utc:new Date().toISOString(),monotonic_ns:holeCheckProcess.hrtime.bigint().toString()});
nodeRepl.write(holeCheckResult);
if (holeCheckFailure) throw holeCheckFailure;
