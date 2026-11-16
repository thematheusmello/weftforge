// Execute only after actual proof0002, passing independent readiness0003 and exact external own-PID accounting.
var holeAppResult = await holeController.call(cua, 'bind-already-running-owned-hole', 'getApp', [holeBinding.bundle]);
var holeApp = holeAppResult.value;
var holeCurrent = await holeObserve(holeController, holeApp, 'initial-owned-hole-state', holeMark, nodeRepl);
