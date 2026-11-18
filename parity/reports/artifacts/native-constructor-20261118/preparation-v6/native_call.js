async function nativeCall({label, method, args = {}, actions = 1, observations = 1, showImage = false}) {
  const started = Date.now();
  const plan = load("nativeUIRuntimePlan");
  if (!plan || !/^[A-Za-z0-9_-]+$/.test(label)) throw Error("Missing frozen plan or invalid own label");
  const cutoff = plan.action_cutoff_epoch * 1000;
  const whole = plan.deadline_epoch * 1000;
  if (!Number.isFinite(cutoff) || !Number.isFinite(whole) || started + 60000 >= Math.min(cutoff, whole)) {
    throw Error("No native dispatch: missing actual clock binding or complete call allowance");
  }
  const deadline = Math.min(started + 60000, cutoff, whole);
  const allowed = ["get_app_state", "click", "press_key", "type_text", "scroll"];
  if (!allowed.includes(method)) throw Error("Unbound native control");
  const q = value => "'" + String(value).replaceAll("'", "'\\''") + "'";
  const py = q(plan.python) + " -B ";
  const save = async (name, value) => {
    store("native-ui:" + plan.case_path + ":" + name, value);
    const raw = JSON.stringify(value, null, 2) + "\n";
    await tools.apply_patch("*** Begin Patch\n*** Add File: " + plan.case_path + "/" + name + "\n" +
      raw.trimEnd().split("\n").map(line => "+" + line).join("\n") + "\n*** End Patch");
  };
  const completeExec = async (stage, cmd, max_output_tokens) => {
    if (Date.now() >= deadline) {
      await save(label + "-" + stage + "-LATE.json", {status: "NEGATIVE_NO_DISPATCH", deadline_epoch: deadline / 1000});
      throw Error("Call lifecycle expired; no new exec dispatch or remote MCP cancellation claimed");
    }
    let receipt = await tools.exec_command({cmd, workdir: plan.case_path, max_output_tokens, yield_time_ms: 10000});
    await save(label + "-" + stage + ".json", receipt);
    const handle = receipt.session_id;
    let waits = 0;
    while (receipt.session_id != null) {
      if (!Number.isInteger(handle) || handle <= 0 || receipt.session_id !== handle || receipt.exit_code != null) {
        throw Error("Ambiguous exec state; retained receipt, no dispatch or replay");
      }
      const remaining = deadline - Date.now();
      if (remaining <= 5000) {
        await save(label + "-" + stage + "-PENDING.json", {
          status: "PENDING_KNOWN_HANDLE_NO_DISPATCH", session_id: handle, deadline_epoch: deadline / 1000,
          reason: "Insufficient bounded poll allowance; process completion/termination is unknown"
        });
        throw Error("Pending exec retained at known handle; stop dispatch without replay or cancellation claim");
      }
      receipt = await tools.write_stdin({session_id: handle, chars: "", max_output_tokens,
        yield_time_ms: Math.min(10000, Math.floor(remaining))});
      waits += 1;
      await save(label + "-" + stage + "-WAIT-" + String(waits).padStart(3, "0") + ".json",
        {session_id_awaited: handle, receipt});
    }
    if (!Number.isInteger(receipt.exit_code)) throw Error("Exec terminal state unknown; no native dispatch or replay");
    await save(label + "-" + stage + "-COMPLETION.json", receipt);
    if (receipt.exit_code !== 0) throw Error("Exec terminal nonzero; stop native authoring without replay");
    if (Date.now() >= deadline) {
      await save(label + "-" + stage + "-LATE.json", {status: "NEGATIVE_LATE_COMPLETION", exit_code: receipt.exit_code,
        deadline_epoch: deadline / 1000});
      throw Error("Late exec completion is negative even with exit zero");
    }
    return receipt;
  };
  await completeExec("GATE",
    py + q(plan.case_path + "/clock.py") + " reserve " + actions + " " + observations + " 60 8388608 " + q(label) +
      " && " + py + q(plan.case_path + "/call_recorder.py") + " begin " + q(label), 1000);
  if (Date.now() >= deadline) throw Error("No native dispatch: lifecycle expired after gate completion");
  const result = await tools["mcp__computer_use__" + method]({...args, app: plan.app});
  store("native-ui-last-result", result);
  await save(label + ".json", result);
  const close = await completeExec("CLOSE",
    py + q(plan.case_path + "/call_recorder.py") + " end " + q(label) +
      " && " + py + q(plan.case_path + "/clock.py") + " status", 1200);
  text({label, retained: true, lifecycle_exit: close.exit_code});
  for (const block of result.content || []) {
    if (block.type === "text") text(block.text);
    else if (showImage && block.type === "image") image(block);
  }
  if (Date.now() >= deadline || result.isError) throw Error("Stop native authoring; retained failed/late call without replay or MCP cancellation claim");
  return result;
}
