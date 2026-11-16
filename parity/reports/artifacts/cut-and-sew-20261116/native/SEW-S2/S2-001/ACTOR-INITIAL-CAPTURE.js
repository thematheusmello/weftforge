// Execute only after actual proof0002, passing independent readiness0003 and exact external own-PID accounting.
var sewAppResult = await sewController.call(cua, 'bind-already-running-owned-sew', 'getApp', [sewBinding.bundle]);
var sewApp = sewAppResult.value;
var sewCurrent = await sewObserve(sewController, sewApp, 'initial-owned-sew-state', sewMark, nodeRepl);
