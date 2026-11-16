// TEMPLATE ONLY. Execute after actual clocked CUA proof and exact external own-PID launch accounting.
// cutMark is initialized synchronously first; all five actual objects are explicit.
var cutAppResult = await cutController.call(cua, 'bind-already-running-owned-cut', 'getApp', [cutBinding.bundle]);
var cutApp = cutAppResult.value;
var cutCurrent = await cutObserve(cutController, cutApp, 'initial-owned-cut-state', cutMark, nodeRepl);
