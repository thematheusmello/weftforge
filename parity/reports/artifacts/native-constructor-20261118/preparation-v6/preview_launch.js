// Definitions only; root must bind a distinct exact case and grant GO before invocation.
async function runPreparedConstructorPreview() {
  const sealed = load("nativeUIPreparedPlan");
  if (!sealed || !sealed.case || !sealed.case_path || !sealed.python ||
      !/^[a-f0-9]{64}$/.test(sealed.plan_sha256)) throw Error("Missing exact prospective sealed plan; unarmed");
  const plan = {...sealed, preparing: true, expected_pid: undefined, binding: undefined, bound_app: undefined,
    watchdog_exec: undefined, resource_exec: undefined, launch_exec: undefined,
    watchdog_live: undefined, resource_live: undefined, launch_live: undefined, launch_started: false};
  let armed = false, delegated = false, primaryError = null;
  try {
    // Bootstrap grants no GUI dispatch; actual STARTED replaces this bound after exact app comparison.
    plan.deadline_epoch = Date.now() / 1000 + 15;
    const arm = await previewBinding(plan, "arm");
    if (arm.exit_code !== 0) throw Error("Fresh clock arm failed");
    const actual = JSON.parse(arm.output);
    armed = true;
    if (actual.case !== plan.case || actual.plan_sha256 !== plan.plan_sha256 ||
        typeof actual.app !== "string" || !actual.app || actual.app !== sealed.app ||
        !Number.isFinite(actual.started.start_epoch) ||
        actual.started.action_cutoff_epoch !== actual.started.start_epoch + 240 ||
        actual.started.deadline_epoch !== actual.started.start_epoch + 300 ||
        actual.started.start_epoch * 1000 > Date.now()) throw Error("Arm app/actual clock does not match the sealed plan");
    plan.app = actual.app;
    plan.action_cutoff_epoch = actual.started.action_cutoff_epoch;
    plan.deadline_epoch = actual.started.deadline_epoch;
    store("nativeUIRuntimePlan", plan);
    for (const [name, file, args] of [["WATCHDOG", "cleanup.py", " --watchdog"], ["RESOURCE", "resource_observer.py", ""]]) {
      const receipt = await tools.exec_command({cmd: previewQuote(plan.python) + " -B " + previewQuote(plan.case_path + "/" + file) + args,
        workdir: plan.case_path, yield_time_ms: 1000, max_output_tokens: 2000});
      plan[name.toLowerCase() + "_exec"] = receipt;
      store(previewKey(plan, name + "-ORIGINAL-HANDLE"), receipt);
      store(previewKey(plan, name + "-RECOVERY"), {status: "UNKNOWN_PENDING_NO_RESTART", session_id: receipt.session_id, exact_receipt: receipt});
      await previewSave(plan, name + "-EXEC-INITIAL.json", receipt);
      if (!Number.isInteger(receipt.session_id) || receipt.session_id <= 0 || receipt.exit_code != null)
        throw Error(name + " did not return its original pending monitor handle");
    }
    plan.launch_started = true;
    const original = await tools.exec_command({cmd: previewQuote(plan.python) + " -B " +
      previewQuote(plan.case_path + "/preview_launch.py") + " launch", workdir: plan.case_path,
      yield_time_ms: 1000, max_output_tokens: 4000});
    plan.launch_exec = original;
    store(previewKey(plan, "LAUNCH-ORIGINAL-HANDLE"), original);
    store(previewKey(plan, "LAUNCH-RECOVERY"), {status: "UNKNOWN_PENDING_NO_RESTART", session_id: original.session_id, exact_receipt: original});
    await previewSave(plan, "LAUNCH-EXEC-INITIAL.json", original);
    if (!Number.isInteger(original.session_id) || original.session_id <= 0 || original.exit_code != null)
      throw Error("Direct producer did not return its original pending handle");
    for (const name of ["WATCHDOG", "RESOURCE", "LAUNCH"]) {
      const initial = plan[name.toLowerCase() + "_exec"];
      const receipt = await tools.write_stdin({session_id: initial.session_id, chars: "", yield_time_ms: 1000, max_output_tokens: 2000});
      plan[name.toLowerCase() + "_live"] = receipt;
      await previewSave(plan, name + "-LIVE.json", {session_id_awaited: initial.session_id, receipt});
      if (receipt.session_id !== initial.session_id || receipt.exit_code != null)
        throw Error(name + " original handle is not still live; no GUI dispatch");
    }
    const bound = await previewBinding(plan, "bind");
    if (bound.exit_code !== 0) throw Error("Actual app/clock/lease/READY binding failed");
    plan.binding = JSON.parse(bound.output);
    if (typeof plan.binding.app !== "string" || !plan.binding.app || plan.binding.app !== actual.app)
      throw Error("Bound disk app does not match authoritative arm app; no GUI dispatch");
    const window = previewMainWindow(plan.binding.inventory, plan.binding.lease.pid);
    if (window.kCGWindowNumber !== plan.binding.lease.window_id) throw Error("Observed original content window changed");
    plan.bound_app = plan.binding.app;
    plan.app = plan.bound_app;
    plan.expected_pid = plan.binding.lease.pid;
    plan.preparing = false;
    previewValidateBinding(plan);
    store("nativeUIRuntimePlan", plan);
    delegated = true;
    return await continuousConstructorPreview();
  } catch (error) {
    primaryError = error;
    if (!delegated) store(previewKey(plan, "STARTUP-ERROR"), {status: "NEGATIVE", error: String(error.message || error),
      launch_original: plan.launch_exec, launch_live: plan.launch_live});
    throw error;
  } finally {
    if (armed && !delegated) {
      try { await previewClose(plan); }
      catch (error) {
        store(previewKey(plan, "STARTUP-CLEANUP-ERROR"), {status: "NEGATIVE", error: String(error.message || error)});
        if (!primaryError) throw error;
      }
    }
  }
}
