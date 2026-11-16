// Execute only after actual proof0002, independent readiness0003, and exact external own-PID0005.
var holeOwnedPid = JSON.parse(holeFs.readFileSync(holePath.join(holeRoot, 'OWNED-PID.json'), 'utf8'));
var holePidResult = JSON.parse(holeFs.readFileSync(holePath.join(holeRoot, 'trace/0005-after.json'), 'utf8'));
if (holeOwnedPid.source_record !== '0005' || holeOwnedPid.command !== [holeBinding.native_binary_path,...holeBinding.input_argv].join(' ') || holePidResult.status !== 'returned' || holePidResult.result.exact_matches.length !== 1 || holePidResult.result.exact_matches[0].pid !== holeOwnedPid.pid || holePidResult.result.exact_matches[0].command !== holeOwnedPid.command || holeController.status().calls !== 5) throw new Error('Exact own-PID gate failed');
var holeAppResult = await holeController.call(cua, 'bind-already-running-owned-h3', 'getApp', [holeBinding.bundle]);
var holeApp = holeAppResult.value;
var holeCurrent = await holeObserve(holeController, holeApp, 'initial-owned-h3-state', holeMark, nodeRepl);
