// Prospective second actual-CUA cell. One counted FS check, before any external launch.
// Fresh local imports/recorder preserve a truthful result even if bootstrap exports are missing.
var sewCheckModule = await import('node:module');
var sewCheckRequire = sewCheckModule.createRequire('<WORKSPACE>/.local/native-cut-and-sew-20261116/S1-001/ACTOR-PERSISTENCE-CHECK.js');
var sewCheckFs = sewCheckRequire('node:fs');
var sewCheckPath = sewCheckRequire('node:path');
var sewCheckCrypto = sewCheckRequire('node:crypto');
var sewCheckProcess = sewCheckRequire('node:process');
var sewCheckRoot = '<WORKSPACE>/.local/native-cut-and-sew-20261116/S1-001';
var sewCheckHash = bytes => sewCheckCrypto.createHash('sha256').update(bytes).digest('hex');
var sewCheckWrite = (file, value) => sewCheckFs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', {flag:'wx',mode:0o600});
var sewCheckRecorderPath = '<WORKSPACE>/.local/native-length-readouts-v2-20261115/recorder.cjs';
if (sewCheckHash(sewCheckFs.readFileSync(sewCheckRecorderPath)) !== '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc') throw new Error('Independent recorder mismatch');
var sewCheckRecorder = sewCheckRequire(sewCheckRecorderPath).createRecorder(sewCheckRoot);
var sewCheckId = sewCheckRecorder.before('actual-cross-cell-runtime-bindings', 'local-runtime-bindings-check', {evidenceKind:'native',native_UI_calls:0,expected_source_record:'0003'});
var sewCheckBindingBytes = null;
var sewCheckBinding = null;
var sewCheckTypes = {sewModule:typeof sewModule,sewRequire:typeof sewRequire,sewFs:typeof sewFs,sewPath:typeof sewPath,sewCrypto:typeof sewCrypto,sewProcess:typeof sewProcess,sewRoot:typeof sewRoot,sewRepo:typeof sewRepo,sewPacket:typeof sewPacket,sewHash:typeof sewHash,sewWrite:typeof sewWrite,sewBindingBytes:typeof sewBindingBytes,sewBindingSha:typeof sewBindingSha,sewBinding:typeof sewBinding,sewRecorder:typeof sewRecorder,sewController:typeof sewController,sewObserve:typeof sewObserve,sewMark:typeof sewMark,sewProofId:typeof sewProofId,sewProofSha:typeof sewProofSha};
var sewCheckFlags = null;
var sewCheckContext = null;
var sewCheckContextExpected = null;
var sewCheckProofBytes = null;
var sewCheckProof = null;
var sewCheckProofExpected = null;
var sewCheckProofResult = null;
var sewCheckOfflineReceipt = null;
var sewCheckOfflineResult = null;
var sewCheckStatus = null;
var sewCheckResult = JSON.parse(sewCheckFs.readFileSync('<WORKSPACE>/.local/cut-and-sew-native-plan-20261116/ACTOR-PERSISTENCE-CHECK-RESULT.template.json', 'utf8'));
var sewCheckFailure = null;
sewCheckResult.helper_types = sewCheckTypes;
sewCheckResult.actual_cua_runtime = true;
sewCheckResult.native_UI_calls = 0;
sewCheckResult.source_record = sewCheckId;
try {
  sewCheckBindingBytes = sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'ROOT-BINDING.json'));
  sewCheckBinding = JSON.parse(sewCheckBindingBytes);
  Object.assign(sewCheckResult, {case:sewCheckBinding.case,attempt:sewCheckBinding.attempt,root_binding_sha256:sewCheckHash(sewCheckBindingBytes)});
  sewCheckFlags = {
    sewModule:typeof sewModule === 'object' && sewModule !== null && typeof sewModule.createRequire === 'function',
    sewRequire:typeof sewRequire === 'function',
    sewFs:typeof sewFs === 'object' && sewFs === sewCheckFs,
    sewPath:typeof sewPath === 'object' && sewPath === sewCheckRequire('node:path'),
    sewCrypto:typeof sewCrypto === 'object' && sewCrypto === sewCheckCrypto,
    sewProcess:typeof sewProcess === 'object' && sewProcess === sewCheckProcess,
    sewRoot:typeof sewRoot === 'string' && sewRoot === sewCheckRoot,
    sewRepo:typeof sewRepo === 'string' && sewRepo === '<WORKSPACE>',
    sewPacket:typeof sewPacket === 'string' && sewPacket === '<WORKSPACE>/.local/cut-and-sew-native-plan-20261116',
    sewHash:typeof sewHash === 'function' && sewHash(sewCheckBindingBytes) === sewCheckHash(sewCheckBindingBytes),
    sewWrite:typeof sewWrite === 'function',
    sewBindingBytes:typeof sewBindingBytes === 'object' && sewBindingBytes !== null && typeof sewBindingBytes.equals === 'function' && sewBindingBytes.equals(sewCheckBindingBytes),
    sewBindingSha:typeof sewBindingSha === 'string' && sewBindingSha === sewCheckHash(sewCheckBindingBytes),
    sewBinding:typeof sewBinding === 'object' && sewBinding !== null && JSON.stringify(sewBinding) === JSON.stringify(sewCheckBinding),
    sewRecorder:typeof sewRecorder === 'object' && sewRecorder !== null && typeof sewRecorder.before === 'function' && typeof sewRecorder.returned === 'function' && typeof sewRecorder.artifact === 'function',
    sewController:typeof sewController === 'object' && sewController !== null && typeof sewController.status === 'function' && typeof sewController.call === 'function' && typeof sewController.capture === 'function',
    sewObserve:typeof sewObserve === 'function' && sewObserve === sewCheckRequire(sewCheckBinding.observer).observe,
    sewMark:typeof sewMark === 'function',
    sewProofId:typeof sewProofId === 'string' && sewProofId === '0002',
    sewProofSha:typeof sewProofSha === 'string' && sewProofSha === sewCheckHash(sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'ACTOR-RUNTIME-PROOF.json')))
  };
  sewCheckContext = JSON.parse(sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'CONTEXT.json'), 'utf8'));
  sewCheckContextExpected = {evidenceKind:'native',callCap:160,elapsedCapS:1800,baselineCalls:0,recorderPath:sewCheckRecorderPath,recorderSha256:'45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc'};
  sewCheckProofBytes = sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'ACTOR-RUNTIME-PROOF.json'));
  sewCheckProof = JSON.parse(sewCheckProofBytes);
  sewCheckOfflineReceipt = JSON.parse(sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'offline-proof/captures/binding-raw.json.sha256.json'), 'utf8'));
  sewCheckProofExpected = {schema:'weftforge.straight-cut.actor-runtime-proof.v1',case:sewCheckBinding.case,attempt:sewCheckBinding.attempt,root_binding_sha256:sewCheckHash(sewCheckBindingBytes),evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:'0002',actor_filename:sewCheckBinding.actor_filename,actor_sha256:sewCheckBinding.actor_sha256,controller_configuration:sewCheckBinding.controller_configuration,observer_explicit_arguments:5,marker_initialized_synchronous_before_observer:true,controller_sha256:sewCheckBinding.controller_sha256,recorder_sha256:sewCheckBinding.recorder_sha256,observer_sha256:sewCheckBinding.observer_sha256,offline_receipt:sewCheckOfflineReceipt};
  sewCheckProofResult = JSON.parse(sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'trace/0002-after.json'), 'utf8'));
  sewCheckOfflineResult = JSON.parse(sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'offline-proof/trace/0001-after.json'), 'utf8'));
  sewCheckStatus = sewCheckFlags.sewController ? sewController.status() : null;
  Object.assign(sewCheckResult, {
    source_record:sewCheckId,actual_cua_runtime:true,native_UI_calls:0,helper_types:sewCheckTypes,helper_flags:sewCheckFlags,helpers_ready:Object.values(sewCheckFlags).every(Boolean),
    root:{expected:sewCheckRoot,actual:typeof sewRoot === 'string' ? sewRoot : null},
    controller_configuration:{expected:sewCheckBinding.controller_configuration,actual:sewCheckFlags.sewBinding ? sewBinding.controller_configuration : null},
    context_schema_ready:JSON.stringify(sewCheckContext) === JSON.stringify(sewCheckContextExpected),
    proof_schema_ready:Object.keys(sewCheckProof).sort().join(',') === Object.keys(sewCheckProofExpected).sort().join(',') && Object.keys(sewCheckProofExpected).every(key => JSON.stringify(sewCheckProof[key]) === JSON.stringify(sewCheckProofExpected[key])),
    proof_result_ready:sewCheckProofResult.status === 'returned' && JSON.stringify(sewCheckProofResult.result) === JSON.stringify({proof_sha256:sewCheckHash(sewCheckProofBytes),actual_cua_runtime:true,native_UI_calls:0}),
    offline_artifact_ready:sewCheckOfflineReceipt.file === 'captures/binding-raw.json' && sewCheckOfflineReceipt.source_record === '0001' && sewCheckOfflineReceipt.unedited === true && sewCheckOfflineReceipt.bytes === sewCheckBindingBytes.length && sewCheckOfflineReceipt.sha256 === sewCheckHash(sewCheckBindingBytes) && sewCheckFs.readFileSync(sewCheckPath.join(sewCheckRoot, 'offline-proof/captures/binding-raw.json')).equals(sewCheckBindingBytes) && sewCheckOfflineResult.status === 'returned' && JSON.stringify(sewCheckOfflineResult.result) === JSON.stringify({raw_binding_sha256:sewCheckHash(sewCheckBindingBytes)}),
    controller_status_ready:sewCheckStatus !== null && sewCheckStatus.evidenceKind === 'native' && sewCheckStatus.calls === 3 && sewCheckStatus.stopped === false && Number.isFinite(sewCheckStatus.elapsedS) && sewCheckStatus.elapsedS >= 0 && sewCheckStatus.elapsedS < 1800,
    proof_sha256:sewCheckHash(sewCheckProofBytes)
  });
  if (sewCheckId !== '0003' || sewCheckBinding.root_authorized_for_case !== true || sewCheckBinding.native_lease_granted !== true || sewCheckBinding.case !== 'SEW-S1' || sewCheckBinding.attempt !== 'S1-001' || !sewCheckResult.helpers_ready || !sewCheckResult.context_schema_ready || !sewCheckResult.proof_schema_ready || !sewCheckResult.proof_result_ready || !sewCheckResult.offline_artifact_ready || !sewCheckResult.controller_status_ready) sewCheckFailure = new Error('Cross-cell persistence/schema check failed; no launch permitted');
} catch (sewCheckError) {
  sewCheckFailure = sewCheckError;
  sewCheckResult.helper_types = sewCheckTypes;
  if (sewCheckFlags !== null) sewCheckResult.helper_flags = sewCheckFlags;
  sewCheckResult.helpers_ready = false;
}
sewCheckRecorder.returned(sewCheckId, sewCheckResult, sewCheckFailure);
sewCheckWrite(sewCheckPath.join(sewCheckRoot, 'ACTOR-PERSISTENCE-CHECK-RESULT.json'), sewCheckResult);
sewCheckWrite(sewCheckPath.join(sewCheckRoot, 'SUMMARY-LOWER-' + sewCheckId + '.json'), {kind:'conditional_marker_only_not_a_summary_attempt',source_record:sewCheckId,utc:new Date().toISOString(),monotonic_ns:sewCheckProcess.hrtime.bigint().toString()});
nodeRepl.write(sewCheckResult);
if (sewCheckFailure) throw sewCheckFailure;
