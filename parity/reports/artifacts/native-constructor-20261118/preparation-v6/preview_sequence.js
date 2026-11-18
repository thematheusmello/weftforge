// Definitions only. No live invocation is authorized by loading these sources.
function previewMainWindow(inventory, pid) {
  const windows = inventory.windows.filter(w => w.kCGWindowOwnerPID === pid &&
    w.kCGWindowLayer === 0 && w.kCGWindowName === "WeftForge" && w.kCGWindowIsOnscreen === true &&
    Number.isFinite(w.kCGWindowBounds.Width) && Number.isFinite(w.kCGWindowBounds.Height) &&
    w.kCGWindowBounds.Width > 100 && w.kCGWindowBounds.Height > 100 &&
    Number.isInteger(w.kCGWindowNumber) && w.kCGWindowNumber > 0);
  if (windows.length !== 1) throw Error("Missing or ambiguous owned content window");
  return windows[0];
}

const previewQuote = value => "'" + String(value).replaceAll("'", "'\\''") + "'";
const previewKey = (plan, name) => "preview:" + plan.case_path + ":" + name;

async function previewSave(plan, name, value, persist = true) {
  // Known handles remain recoverable even when their first filesystem write fails.
  store(previewKey(plan, name), value);
  if (persist) await tools.apply_patch("*** Begin Patch\n*** Add File: " + plan.case_path + "/" + name + "\n" +
    JSON.stringify(value, null, 2).split("\n").map(line => "+" + line).join("\n") + "\n*** End Patch");
}

async function previewAwaitOriginal(plan, stage, initial, {persist = true, maxOutputTokens = 2000, originalHandle = initial && initial.session_id} = {}) {
  let receipt = initial, waits = 0;
  const handle = originalHandle;
  const rememberPending = reason => store(previewKey(plan, stage + "-RECOVERY"), {
    status: "UNKNOWN_PENDING_NO_RESTART", session_id: handle, exact_receipt: receipt, reason
  });
  const rememberTerminal = () => store(previewKey(plan, stage + "-RECOVERY"), {
    status: "TERMINAL_OBSERVED_BEFORE_RETENTION", session_id: handle, exact_receipt: receipt
  });
  if (receipt && receipt.session_id == null && Number.isInteger(receipt.exit_code)) rememberTerminal();
  else if (handle != null) rememberPending("Known initial handle; terminal not yet established");
  try {
    await previewSave(plan, stage + "-INITIAL.json", receipt, persist);
    while (receipt.session_id != null) {
      if (!Number.isInteger(handle) || handle <= 0 || receipt.session_id !== handle || receipt.exit_code != null)
        throw Error("Ambiguous cleanup/controller handle; no restart");
      const remaining = plan.deadline_epoch * 1000 - Date.now();
      if (remaining <= 5000) {
        await previewSave(plan, stage + "-PENDING.json", {session_id: handle, status: "UNKNOWN_PENDING_NO_RESTART"}, persist);
        throw Error("Pending original handle at whole-case deadline");
      }
      receipt = await tools.write_stdin({session_id: handle, chars: "", max_output_tokens: maxOutputTokens,
        yield_time_ms: Math.min(10000, Math.floor(remaining))});
      store(previewKey(plan, stage + "-LAST-RECEIPT"), receipt);
      if (receipt.session_id != null) rememberPending("Original handle remains pending");
      else if (Number.isInteger(receipt.exit_code)) rememberTerminal();
      await previewSave(plan, stage + "-WAIT-" + String(++waits).padStart(3, "0") + ".json",
        {session_id_awaited: handle, receipt}, persist);
    }
    if (!Number.isInteger(receipt.exit_code)) throw Error("Cleanup/controller terminal unknown");
    await previewSave(plan, stage + "-COMPLETION.json", receipt, persist);
    store(previewKey(plan, stage + "-RECOVERY"), {status: "TERMINAL_RETAINED", session_id: handle, exact_receipt: receipt});
    if (Date.now() >= plan.deadline_epoch * 1000) {
      await previewSave(plan, stage + "-LATE.json", {status: "NEGATIVE_LATE_COMPLETION",
        exit_code: receipt.exit_code, deadline_epoch: plan.deadline_epoch}, persist);
      throw Error("Late controller completion is negative even with exit zero");
    }
    return receipt;
  } catch (error) {
    store(previewKey(plan, stage + "-ERROR"), {status: "NEGATIVE", error: String(error.message || error),
      session_id: handle, last_receipt: receipt, terminal_known: Number.isInteger(receipt && receipt.exit_code)});
    throw error;
  }
}

async function previewOwner(plan, phase, options) {
  if (phase !== "close" && Date.now() >= plan.deadline_epoch * 1000) throw Error("No new admission work after whole-case deadline");
  const initial = await tools.exec_command({cmd: previewQuote(plan.python) + " -B " +
    previewQuote(plan.case_path + "/preview_owner.py") + " " + phase, workdir: plan.case_path,
    yield_time_ms: 10000, max_output_tokens: 2000});
  return previewAwaitOriginal(plan, "OWNER-" + phase, initial, options);
}

async function previewBinding(plan, phase) {
  if (Date.now() >= plan.deadline_epoch * 1000) throw Error("No binding work after whole-case deadline");
  const initial = await tools.exec_command({cmd: previewQuote(plan.python) + " -B " +
    previewQuote(plan.case_path + "/preview_launch.py") + " " + phase, workdir: plan.case_path,
    yield_time_ms: 10000, max_output_tokens: 4000});
  return previewAwaitOriginal(plan, "BIND-" + phase, initial, {maxOutputTokens: 4000});
}

async function previewClose(plan) {
  const errors = [];
  try {
    if ((await previewOwner(plan, "close")).exit_code !== 0) throw Error("Owned cleanup failed");
  } catch (error) { errors.push(String(error.message || error)); }
  // Owner close must establish a producer terminal/barrier before it writes DONE/RESOURCE-STOP.
  for (const name of ["LAUNCH", "WATCHDOG", "RESOURCE"]) {
    const original = plan[name.toLowerCase() + "_exec"];
    if (!original && !plan.launch_started && name === "LAUNCH") continue;
    if (!original && plan.preparing && name !== "LAUNCH") continue;
    try {
      if (!original) throw Error("Missing original " + name + " handle");
      const live = plan[name.toLowerCase() + "_live"];
      const terminal = live && live.session_id == null && Number.isInteger(live.exit_code) ? live : original;
      if ((await previewAwaitOriginal(plan, name, terminal, {originalHandle: original.session_id})).exit_code !== 0)
        throw Error(name + " terminal failed");
    } catch (error) { errors.push(String(error.message || error)); }
  }
  if (errors.length) throw Error("Cleanup/controller admission failed: " + errors.join("; "));
}

function previewValidateBinding(plan) {
  const binding = plan && plan.binding;
  const started = binding && binding.started;
  if (!started || !Number.isFinite(started.start_epoch) ||
      started.action_cutoff_epoch !== started.start_epoch + 240 || started.deadline_epoch !== started.start_epoch + 300 ||
      plan.action_cutoff_epoch !== started.action_cutoff_epoch || plan.deadline_epoch !== started.deadline_epoch ||
      binding.case_path !== plan.case_path || binding.case !== plan.case || binding.plan_sha256 !== plan.plan_sha256 ||
      typeof binding.fixture_basename !== "string" || !binding.fixture_basename.trim() || /[\\/]/.test(binding.fixture_basename) ||
      typeof binding.app !== "string" || !binding.app || binding.app !== plan.app || plan.bound_app !== binding.app ||
      binding.lease.app_path !== binding.app ||
      binding.lease.pid !== plan.expected_pid || !Number.isInteger(plan.expected_pid) || plan.expected_pid <= 0 ||
      binding.lease.bundle_id !== plan.bundle_id || binding.lease.source_binary_sha256 !== plan.native.sha256 ||
      !(binding.lease.launch_time === null || Number.isFinite(binding.lease.launch_time)) || binding.lease.executable !== plan.native.path ||
      typeof binding.lease.native_start_token !== "string" || !binding.lease.native_start_token.trim() ||
      !Number.isInteger(binding.lease.parent_pid) || binding.lease.parent_pid <= 0 ||
      !Number.isFinite(binding.lease.direct_child_created_epoch) || binding.lease.direct_child_created_epoch < started.start_epoch ||
      !Number.isFinite(binding.lease.native_identity_observed_epoch) ||
      binding.lease.native_identity_observed_epoch < binding.lease.direct_child_created_epoch ||
      !Number.isInteger(binding.lease.window_id) || binding.lease.window_id <= 0 ||
      !Number.isFinite(binding.checked_epoch) || binding.checked_epoch < binding.lease.native_identity_observed_epoch || binding.checked_epoch * 1000 > Date.now() ||
      Date.now() - binding.checked_epoch * 1000 > 5000 || binding.sources_exact !== true ||
      binding.same_case_ready_and_processes_live !== true) throw Error("Missing fresh actual same-case epoch/lease binding");
  const producer = binding.producer;
  if (!producer || producer.case !== plan.case || producer.plan_sha256 !== plan.plan_sha256 ||
      producer.start_epoch !== started.start_epoch || !Number.isInteger(producer.pid) || producer.pid <= 0 ||
      producer.pgid !== producer.pid || binding.lease.process_group !== producer.pgid || binding.lease.parent_pid !== producer.pid ||
      !plan.launch_exec || !Number.isInteger(plan.launch_exec.session_id) || plan.launch_exec.session_id <= 0 ||
      plan.launch_exec.exit_code != null || !plan.launch_live || plan.launch_live.session_id !== plan.launch_exec.session_id ||
      plan.launch_live.exit_code != null) throw Error("Missing original live direct producer/child group binding");
  for (const name of ["WATCHDOG", "RESOURCE"]) {
    const original = plan[name.toLowerCase() + "_exec"], live = plan[name.toLowerCase() + "_live"];
    const ready = binding.monitors[name];
    if (!original || !live || !Number.isInteger(original.session_id) || original.session_id <= 0 ||
        original.exit_code != null || live.session_id !== original.session_id || live.exit_code != null ||
        !ready || ready.case !== plan.case || ready.plan_sha256 !== plan.plan_sha256 ||
        ready.start_epoch !== started.start_epoch || ready.action_cutoff_epoch !== started.action_cutoff_epoch ||
        ready.deadline_epoch !== started.deadline_epoch || !Number.isFinite(ready.ready_epoch) || ready.ready_epoch < started.start_epoch ||
        ready.ready_epoch > binding.checked_epoch || !Number.isInteger(ready.pid) || ready.pid <= 0)
      throw Error("Missing actual LIVE READY original " + name + " monitor");
  }
  if (plan.watchdog_exec.session_id === plan.resource_exec.session_id ||
      binding.monitors.WATCHDOG.pid === binding.monitors.RESOURCE.pid) throw Error("Monitor identities are not distinct");
}

async function continuousConstructorPreview() {
  const plan = load("nativeUIRuntimePlan");
  const textOf = result => (result.content || []).filter(c => c.type === "text").map(c => c.text).join("\n");
  const ownedState = state => {
    const headers = [...textOf(state).matchAll(/^App=.*\bpid\s+(\d+)\)/gm)];
    if (headers.length !== 1 || Number(headers[0][1]) !== plan.expected_pid) throw Error("Returned state ownership mismatch");
  };
  const button = (state, label) => {
    ownedState(state);
    const matches = [...textOf(state).matchAll(/^\s*(\d+) button(?: \(([^)]*)\))? ([^\n]+)$/gm)]
      .filter(match => match[3].trim() === label);
    if (matches.length !== 1 || /disabled/i.test(matches[0][2] || "")) throw Error("Missing, ambiguous or disabled " + label);
    return matches[0][1];
  };
  const result = {preview_error: null, quit_error: null, constructor_response: null,
    normal_quit_helper_completed: false, quit_response_qualification: null, visual_alignment_accepted: false, calls: []};
  let owned = false;
  const navigation = async (label, method, args = {}) => {
    if (Date.now() + 120000 >= plan.action_cutoff_epoch * 1000) throw Error("Preserve full navigation and Quit allowances");
    const previousOwned = owned; owned = false;
    try {
      const state = await nativeCall({label, method, args, actions: method === "get_app_state" ? 0 : 1,
        observations: 2, showImage: false});
      ownedState(state); owned = true; result.calls.push(label); return state;
    } catch (error) {
      const gate = load("native-ui:" + plan.case_path + ":" + label + "-GATE-COMPLETION.json");
      if (gate && Number.isInteger(gate.exit_code) && gate.exit_code !== 0) owned = previousOwned;
      throw error;
    }
  };
  try {
    previewValidateBinding(plan);
    let state = await navigation("VISUAL-01-state", "get_app_state");
    owned = false;
    const post = await previewBinding(plan, "post-getter");
    const checked = JSON.parse(post.output);
    if (post.exit_code !== 0 || checked.app !== plan.bound_app || checked.pid !== plan.expected_pid ||
        checked.window_id !== plan.binding.lease.window_id || checked.same_observed_lease !== true ||
        !Number.isFinite(checked.checked_epoch) || checked.checked_epoch * 1000 > Date.now() ||
        Date.now() - checked.checked_epoch * 1000 > 5000) throw Error("Post-getter ownership changed; no navigation/replay");
    owned = true;
    if (!textOf(state).includes(plan.binding.fixture_basename) || !textOf(state).includes("Rest mesh rebuilt from editable construction."))
      throw Error("Loaded settled public sample not established");
    await previewSave(plan, "SAMPLE-SETTLED.json", {pid: plan.expected_pid, settled_epoch: Date.now() / 1000,
      basis: "VISUAL-01-state loaded public sample; no Empty-project or workflow credit"});
    if ((await previewOwner(plan, "baseline")).exit_code !== 0) throw Error("Settled sample baseline failed");
    state = await navigation("VISUAL-02-frame", "click", {element_index: button(state, "Frame all")});
    state = await navigation("VISUAL-03-create", "click", {element_index: button(state, "Create pattern")});
    for (const label of ["Width m", "Height m", "Position X m", "Position Y m"]) {
      const labels = [...textOf(state).matchAll(/^\s*\d+ text(?: \([^)]*\))? ([^\n]+)$/gm)]
        .filter(match => match[1].trim() === label);
      if (labels.length !== 1) throw Error("Missing or ambiguous constructor label " + label);
    }
    if (!(state.content || []).some(c => c.type === "image" && ["image/png", "image/jpeg"].includes(c.mimeType) &&
        typeof c.data === "string" && c.data.length > 0 && c.data.length % 4 === 0 &&
        /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(c.data)))
      throw Error("Constructor response lacks nonempty strict image base64; no refresh/retry");
    result.constructor_response = "VISUAL-03-create";
  } catch (error) {
    result.preview_error = String(error.message || error);
  } finally {
    try {
      const quitStarted = Date.now();
      const quitDeadline = Math.min(quitStarted + 60000, plan.action_cutoff_epoch * 1000, plan.deadline_epoch * 1000);
      if (!owned || quitStarted + 60000 >= Math.min(plan.action_cutoff_epoch * 1000, plan.deadline_epoch * 1000))
        throw Error("No confirmed owned state/full Quit allowance");
      const quit = {label: "VISUAL-04-quit", method: "press_key", args: {key: "super+q"},
        actions: 1, observations: 2, showImage: false};
      const key = suffix => "native-ui:" + plan.case_path + ":VISUAL-04-quit" + suffix;
      if ([".json", "-GATE-COMPLETION.json", "-CLOSE-COMPLETION.json"].some(suffix => load(key(suffix)) != null))
        throw Error("Quit label already has retained evidence; no replay");
      await previewSave(plan, "QUIT-EXPECTED-" + plan.expected_pid + ".json", {pid: plan.expected_pid,
        normal_native_Quit_expected: true, epoch: Date.now() / 1000});
      let qualification = {kind: "NONERROR_NATIVE_CALL_RETURN"};
      try { await nativeCall(quit); }
      catch (error) {
        const raw = load(key(".json")), gate = load(key("-GATE-COMPLETION.json")), close = load(key("-CLOSE-COMPLETION.json"));
        // The unique exact value is compared structurally: load may deserialize each receipt.
        const appQuit = value => value && Object.keys(value).sort().join(",") === "content,isError" && value.isError === true &&
          Array.isArray(value.content) && value.content.length === 1 && value.content[0] &&
          Object.keys(value.content[0]).sort().join(",") === "text,type" && value.content[0].type === "text" &&
          value.content[0].text === "Computer Use server error -10005: App quit";
        let lifecycle;
        try { lifecycle = JSON.parse(close.output.split("\n")[0]); } catch (_) {}
        const qualified = Date.now();
        if (!owned || quit.label !== "VISUAL-04-quit" || quit.method !== "press_key" || quit.args.key !== "super+q" ||
            error.message !== "Stop native authoring; retained failed/late call without replay or MCP cancellation claim" ||
            !appQuit(raw) || !appQuit(load("native-ui-last-result")) ||
            !gate || gate.exit_code !== 0 || gate.session_id != null || !close || close.exit_code !== 0 || close.session_id != null ||
            !lifecycle || lifecycle.label !== quit.label || lifecycle.bounded_lifecycle !== true ||
            !Number.isFinite(lifecycle.closed_epoch) || lifecycle.closed_epoch < quitStarted / 1000 || lifecycle.closed_epoch > qualified / 1000 ||
            !Number.isFinite(lifecycle.elapsed_through_persisted_receipt_seconds) || lifecycle.elapsed_through_persisted_receipt_seconds < 0 ||
            lifecycle.elapsed_through_persisted_receipt_seconds >= 60 || qualified >= quitDeadline ||
            ["-GATE-LATE.json", "-GATE-PENDING.json", "-CLOSE-LATE.json", "-CLOSE-PENDING.json"].some(suffix => load(key(suffix)) != null))
          throw error;
        qualification = {kind: "EXACT_OBSERVED_APP_QUIT_ERROR_WITH_BOUNDED_CLOSE", raw_isError: true,
          response_key: key(".json"), gate_completion_key: key("-GATE-COMPLETION.json"), close_completion_key: key("-CLOSE-COMPLETION.json")};
      }
      const qualified = Date.now();
      if (quit.label !== "VISUAL-04-quit" || quit.method !== "press_key" || quit.args.key !== "super+q" || qualified >= quitDeadline)
        throw Error("Quit dispatch or caller lifecycle is not the exact timely owned request");
      result.quit_response_qualification = {...qualification, label: quit.label, method: quit.method, key: quit.args.key,
        caller_start_epoch: quitStarted / 1000, caller_deadline_epoch: quitDeadline / 1000, qualified_epoch: qualified / 1000};
      result.normal_quit_helper_completed = true;
    } catch (error) { result.quit_error = String(error.message || error); }
    finally {
      try { await previewSave(plan, "SEQUENCE-METADATA.json", result); }
      finally {
        await previewClose(plan);
        if ((await previewOwner(plan, "seal")).exit_code !== 0) throw Error("Preview seal negative; no regrade");
        // The audit writes nothing: every filesystem receipt/manifest exists before its final disk/byte/time probe.
        const audit = await previewOwner(plan, "audit", {persist: false});
        const final = JSON.parse(audit.output);
        if (audit.exit_code !== 0 || final.status !== "PROVISIONAL_FINAL_AUDIT" || final.provisional_candidate !== true ||
            final.case !== plan.case || final.plan_sha256 !== plan.plan_sha256 ||
            final.deadline_epoch !== plan.deadline_epoch || final.checked_epoch > Date.now() / 1000 ||
            Date.now() >= plan.deadline_epoch * 1000) throw Error("Final post-retention admission failed");
        const admission = {...result, status: "BOUNDED_CAPTURE_AND_NORMAL_CLOSE_PENDING_VISUAL_REVIEW",
          normal_close_accepted: true, alignment_accepted: false, final_audit: final,
          final_audit_receipt_key: previewKey(plan, "OWNER-audit-COMPLETION.json"),
          authoritative_admission_epoch: Date.now() / 1000, persisted_files_remain_provisional: true};
        store(previewKey(plan, "FINAL-ADMISSION"), admission);
        if (Date.now() >= plan.deadline_epoch * 1000) {
          store(previewKey(plan, "FINAL-ADMISSION"), {status: "NEGATIVE_LATE_FINAL_MEMORY_RETENTION",
            normal_close_accepted: false, alignment_accepted: false, deadline_epoch: plan.deadline_epoch});
          throw Error("Final admission retention crossed the whole-case deadline");
        }
        return admission;
      }
    }
  }
}
