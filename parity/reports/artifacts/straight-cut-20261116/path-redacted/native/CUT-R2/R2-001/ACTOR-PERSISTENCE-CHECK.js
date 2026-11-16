// Prospective second actual-CUA cell. One counted FS check, before any external launch.
// Fresh local imports/recorder preserve a truthful result even if bootstrap exports are missing.
var cutCheckModule = await import('node:module');
var cutCheckRequire = cutCheckModule.createRequire('<WORKSPACE>/.local/native-straight-cut-20261116/R2-001/ACTOR-PERSISTENCE-CHECK.js');
var cutCheckFs = cutCheckRequire('node:fs');
var cutCheckPath = cutCheckRequire('node:path');
var cutCheckCrypto = cutCheckRequire('node:crypto');
var cutCheckProcess = cutCheckRequire('node:process');
var cutCheckRoot = '<WORKSPACE>/.local/native-straight-cut-20261116/R2-001';
var cutCheckHash = bytes => cutCheckCrypto.createHash('sha256').update(bytes).digest('hex');
var cutCheckWrite = (file, value) => cutCheckFs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', {flag:'wx',mode:0o600});
var cutCheckRecorderPath = '<WORKSPACE>/.local/native-length-readouts-v2-20261115/recorder.cjs';
if (cutCheckHash(cutCheckFs.readFileSync(cutCheckRecorderPath)) !== '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc') throw new Error('Independent recorder mismatch');
var cutCheckRecorder = cutCheckRequire(cutCheckRecorderPath).createRecorder(cutCheckRoot);
var cutCheckId = cutCheckRecorder.before('actual-cross-cell-runtime-bindings', 'local-runtime-bindings-check', {evidenceKind:'native',native_UI_calls:0,expected_source_record:'0003'});
var cutCheckBindingBytes = null;
var cutCheckBinding = null;
var cutCheckTypes = null;
var cutCheckFlags = null;
var cutCheckContext = null;
var cutCheckContextExpected = null;
var cutCheckProofBytes = null;
var cutCheckProof = null;
var cutCheckProofExpected = null;
var cutCheckProofResult = null;
var cutCheckOfflineReceipt = null;
var cutCheckOfflineResult = null;
var cutCheckStatus = null;
var cutCheckResult = null;
var cutCheckFailure = null;
try {
  cutCheckBindingBytes = cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'ROOT-BINDING.json'));
  cutCheckBinding = JSON.parse(cutCheckBindingBytes);
  cutCheckTypes = {cutModule:typeof cutModule,cutRequire:typeof cutRequire,cutFs:typeof cutFs,cutPath:typeof cutPath,cutCrypto:typeof cutCrypto,cutProcess:typeof cutProcess,cutRoot:typeof cutRoot,cutRepo:typeof cutRepo,cutPacket:typeof cutPacket,cutHash:typeof cutHash,cutWrite:typeof cutWrite,cutBindingBytes:typeof cutBindingBytes,cutBindingSha:typeof cutBindingSha,cutBinding:typeof cutBinding,cutRecorder:typeof cutRecorder,cutController:typeof cutController,cutObserve:typeof cutObserve,cutMark:typeof cutMark,cutProofId:typeof cutProofId,cutProofSha:typeof cutProofSha};
  cutCheckFlags = {
    cutModule:typeof cutModule === 'object' && cutModule !== null && typeof cutModule.createRequire === 'function',
    cutRequire:typeof cutRequire === 'function',
    cutFs:typeof cutFs === 'object' && cutFs === cutCheckFs,
    cutPath:typeof cutPath === 'object' && cutPath === cutCheckRequire('node:path'),
    cutCrypto:typeof cutCrypto === 'object' && cutCrypto === cutCheckCrypto,
    cutProcess:typeof cutProcess === 'object' && cutProcess === cutCheckProcess,
    cutRoot:typeof cutRoot === 'string' && cutRoot === cutCheckRoot,
    cutRepo:typeof cutRepo === 'string' && cutRepo === '<WORKSPACE>',
    cutPacket:typeof cutPacket === 'string' && cutPacket === '<WORKSPACE>/.local/straight-cut-native-plan-v2-20261116',
    cutHash:typeof cutHash === 'function' && cutHash(cutCheckBindingBytes) === cutCheckHash(cutCheckBindingBytes),
    cutWrite:typeof cutWrite === 'function',
    cutBindingBytes:typeof cutBindingBytes === 'object' && cutBindingBytes !== null && typeof cutBindingBytes.equals === 'function' && cutBindingBytes.equals(cutCheckBindingBytes),
    cutBindingSha:typeof cutBindingSha === 'string' && cutBindingSha === cutCheckHash(cutCheckBindingBytes),
    cutBinding:typeof cutBinding === 'object' && cutBinding !== null && JSON.stringify(cutBinding) === JSON.stringify(cutCheckBinding),
    cutRecorder:typeof cutRecorder === 'object' && cutRecorder !== null && typeof cutRecorder.before === 'function' && typeof cutRecorder.returned === 'function' && typeof cutRecorder.artifact === 'function',
    cutController:typeof cutController === 'object' && cutController !== null && typeof cutController.status === 'function' && typeof cutController.call === 'function' && typeof cutController.capture === 'function',
    cutObserve:typeof cutObserve === 'function' && cutObserve === cutCheckRequire(cutCheckBinding.observer).observe,
    cutMark:typeof cutMark === 'function',
    cutProofId:typeof cutProofId === 'string' && cutProofId === '0002',
    cutProofSha:typeof cutProofSha === 'string' && cutProofSha === cutCheckHash(cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'ACTOR-RUNTIME-PROOF.json')))
  };
  cutCheckContext = JSON.parse(cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'CONTEXT.json'), 'utf8'));
  cutCheckContextExpected = {evidenceKind:'native',callCap:65,elapsedCapS:600,baselineCalls:0,recorderPath:cutCheckRecorderPath,recorderSha256:'45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc'};
  cutCheckProofBytes = cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'ACTOR-RUNTIME-PROOF.json'));
  cutCheckProof = JSON.parse(cutCheckProofBytes);
  cutCheckOfflineReceipt = JSON.parse(cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'offline-proof/captures/binding-raw.json.sha256.json'), 'utf8'));
  cutCheckProofExpected = {schema:'weftforge.straight-cut.actor-runtime-proof.v1',case:'CUT-R2',attempt:'R2-001',root_binding_sha256:cutCheckHash(cutCheckBindingBytes),evidence_kind:'native_cua_runtime',actual_cua_runtime:true,native_UI_calls:0,createRequire:true,fs_writable:true,source_record:'0002',actor_filename:cutCheckBinding.actor_filename,actor_sha256:cutCheckBinding.actor_sha256,controller_configuration:cutCheckBinding.controller_configuration,observer_explicit_arguments:5,marker_initialized_synchronous_before_observer:true,controller_sha256:cutCheckBinding.controller_sha256,recorder_sha256:cutCheckBinding.recorder_sha256,observer_sha256:cutCheckBinding.observer_sha256,offline_receipt:cutCheckOfflineReceipt};
  cutCheckProofResult = JSON.parse(cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'trace/0002-after.json'), 'utf8'));
  cutCheckOfflineResult = JSON.parse(cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'offline-proof/trace/0001-after.json'), 'utf8'));
  cutCheckStatus = cutCheckFlags.cutController ? cutController.status() : null;
  cutCheckResult = {
    source_record:cutCheckId,actual_cua_runtime:true,native_UI_calls:0,helper_types:cutCheckTypes,helper_flags:cutCheckFlags,helpers_ready:Object.values(cutCheckFlags).every(Boolean),
    root:{expected:cutCheckRoot,actual:typeof cutRoot === 'string' ? cutRoot : null},
    controller_configuration:{expected:cutCheckBinding.controller_configuration,actual:cutCheckFlags.cutBinding ? cutBinding.controller_configuration : null},
    context_schema_ready:JSON.stringify(cutCheckContext) === JSON.stringify(cutCheckContextExpected),
    proof_schema_ready:Object.keys(cutCheckProof).sort().join(',') === Object.keys(cutCheckProofExpected).sort().join(',') && Object.keys(cutCheckProofExpected).every(key => JSON.stringify(cutCheckProof[key]) === JSON.stringify(cutCheckProofExpected[key])),
    proof_result_ready:cutCheckProofResult.status === 'returned' && JSON.stringify(cutCheckProofResult.result) === JSON.stringify({proof_sha256:cutCheckHash(cutCheckProofBytes),actual_cua_runtime:true,native_UI_calls:0}),
    offline_artifact_ready:cutCheckOfflineReceipt.file === 'captures/binding-raw.json' && cutCheckOfflineReceipt.source_record === '0001' && cutCheckOfflineReceipt.unedited === true && cutCheckOfflineReceipt.bytes === cutCheckBindingBytes.length && cutCheckOfflineReceipt.sha256 === cutCheckHash(cutCheckBindingBytes) && cutCheckFs.readFileSync(cutCheckPath.join(cutCheckRoot, 'offline-proof/captures/binding-raw.json')).equals(cutCheckBindingBytes) && cutCheckOfflineResult.status === 'returned' && JSON.stringify(cutCheckOfflineResult.result) === JSON.stringify({raw_binding_sha256:cutCheckHash(cutCheckBindingBytes)}),
    controller_status_ready:cutCheckStatus !== null && cutCheckStatus.evidenceKind === 'native' && cutCheckStatus.calls === 3 && cutCheckStatus.stopped === false && Number.isFinite(cutCheckStatus.elapsedS) && cutCheckStatus.elapsedS >= 0 && cutCheckStatus.elapsedS < 600,
    proof_sha256:cutCheckHash(cutCheckProofBytes)
  };
  if (cutCheckId !== '0003' || cutCheckBinding.root_authorized_for_case !== true || cutCheckBinding.native_lease_granted !== true || cutCheckBinding.case !== 'CUT-R2' || cutCheckBinding.attempt !== 'R2-001' || !cutCheckResult.helpers_ready || !cutCheckResult.context_schema_ready || !cutCheckResult.proof_schema_ready || !cutCheckResult.proof_result_ready || !cutCheckResult.offline_artifact_ready || !cutCheckResult.controller_status_ready) cutCheckFailure = new Error('Cross-cell persistence/schema check failed; no launch permitted');
} catch (cutCheckError) {
  cutCheckFailure = cutCheckError;
  cutCheckResult = {source_record:cutCheckId,actual_cua_runtime:true,native_UI_calls:0,helper_types:cutCheckTypes,helper_flags:cutCheckFlags,helpers_ready:false,error:cutCheckError.message};
}
cutCheckRecorder.returned(cutCheckId, cutCheckResult, cutCheckFailure);
cutCheckWrite(cutCheckPath.join(cutCheckRoot, 'ACTOR-PERSISTENCE-CHECK-RESULT.json'), cutCheckResult);
cutCheckWrite(cutCheckPath.join(cutCheckRoot, 'SUMMARY-LOWER-' + cutCheckId + '.json'), {kind:'conditional_marker_only_not_a_summary_attempt',source_record:cutCheckId,utc:new Date().toISOString(),monotonic_ns:cutCheckProcess.hrtime.bigint().toString()});
nodeRepl.write(cutCheckResult);
if (cutCheckFailure) throw cutCheckFailure;
