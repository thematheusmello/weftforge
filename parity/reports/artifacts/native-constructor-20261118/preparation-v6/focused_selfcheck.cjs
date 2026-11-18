"use strict";
const assert = require("node:assert/strict"), fs = require("node:fs"), path = require("node:path"), vm = require("node:vm"), crypto = require("node:crypto");
const helper = fs.readFileSync(path.join(__dirname, "native_call.js"), "utf8");
const sequence = fs.readFileSync(path.join(__dirname, "preview_sequence.js"), "utf8");
assert.equal(crypto.createHash("sha256").update(helper).digest("hex"), "9d108334cf4b1666e5e6546df221af1aba71d5c32a9a2642c2e49cac6545f890");
const observedPath = path.resolve(__dirname, "../notch-bend-integration-20261117/.local/native-constructor-current-P004-20261118/VISUAL-04-quit.json");
const observed = JSON.parse(fs.readFileSync(observedPath, "utf8"));
const PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO5p45cAAAAASUVORK5CYII=";
const clone = value => value === undefined ? undefined : JSON.parse(JSON.stringify(value));

async function scenario(name, options = {}) {
  const start = 1700000000, base = start * 1000;
  let now = base + 10000, quitCallerStart;
  const stores = new Map(), retained = new Map(), calls = [], polled = new Set(), begins = new Map();
  const terminal = (exit_code = 0, output = "stub terminal") => ({exit_code, output});
  const pending = session_id => ({session_id, output: "stub pending"});
  const started = {start_epoch: start, action_cutoff_epoch: start + 240, deadline_epoch: start + 300};
  const fixture = "different-readonly-input.weftforge";
  const lease = {pid: 42, app_path: "/stub/own.app", executable: "/stub/native", bundle_id: "org.weftforge.stub", launch_time: null,
    native_start_token: "Wed Nov 18 07:44:25 2026", parent_pid: 4242, process_group: 4242,
    direct_child_created_epoch: start + 2, native_identity_observed_epoch: start + 3,
    source_binary_sha256: "2".repeat(64), window_id: 99};
  const ready = pid => ({pid, case: "WF-V6-STUB", plan_sha256: "1".repeat(64), ready_epoch: start + 1, ...started});
  const plan = {case: "WF-V6-STUB", case_path: "/stub/" + name, python: "/stub/python", plan_sha256: "1".repeat(64),
    app: "/stub/own.app", bound_app: "/stub/own.app", native: {path: "/stub/native", sha256: lease.source_binary_sha256},
    expected_pid: 42, bundle_id: lease.bundle_id, ...started, launch_started: true, preparing: false,
    launch_exec: pending(91), launch_live: pending(91), watchdog_exec: pending(81), watchdog_live: pending(81), resource_exec: pending(82), resource_live: pending(82),
    binding: {case: "WF-V6-STUB", case_path: "/stub/" + name, plan_sha256: "1".repeat(64), app: "/stub/own.app", fixture_basename: fixture,
      lease, started, producer: {pid: 4242, pgid: 4242, case: "WF-V6-STUB", plan_sha256: "1".repeat(64), start_epoch: start},
      monitors: {WATCHDOG: ready(51), RESOURCE: ready(52)}, checked_epoch: now / 1000, sources_exact: true, same_case_ready_and_processes_live: true}};
  stores.set("nativeUIRuntimePlan", clone(plan));
  const nativeKey = suffix => "native-ui:" + plan.case_path + ":VISUAL-04-quit" + suffix;
  if (options.stale) stores.set(nativeKey(".json"), clone(observed));
  function state(phase) {
    const lines = ["App=/stub/own.app (pid 42)", 'Window: "WeftForge"', "10 text " + fixture,
      "11 text Rest mesh rebuilt from editable construction."];
    if (phase === "initial") lines.push("17 button Frame all", "46 button Create pattern");
    else if (phase === "framed") lines.push("51 button Frame all", "90 button Create pattern");
    else for (const [index, label] of ["Width m", "Height m", "Position X m", "Position Y m"].entries()) lines.push((100 + index) + " text " + label);
    return {content: [{type: "text", text: lines.join("\n")}, {type: "image", mimeType: "image/png", data: PNG}]};
  }
  const context = {Date: class extends Date {static now() {return now;}}, text() {}, image() {},
    store(key, value) {
      if (options.missingClose && key === nativeKey("-CLOSE-COMPLETION.json")) return;
      stores.set(key, clone(value));
    },
    load(key) {
      let value = clone(stores.get(key));
      if (key === "native-ui-last-result" && value?.isError) {
        if (options.lastMismatch) value.content[0].text = "Other call result";
        // Deliberately different object identities and key order from the per-label response.
        value.content = value.content.map(block => Object.fromEntries(Object.entries(block).reverse()));
        value = Object.fromEntries(Object.entries(value).reverse());
      }
      return value;
    },
    tools: {
      async apply_patch(patch) {
        const lines = patch.split("\n"), file = lines[1].slice("*** Add File: ".length);
        assert.ok(file.startsWith(plan.case_path + "/"));
        const name = file.slice(plan.case_path.length + 1);
        if (name === "QUIT-EXPECTED-42.json") {quitCallerStart = now; if (options.lateCaller) now += 2000;}
        if (options.retentionFailure && name === "VISUAL-04-quit-CLOSE-COMPLETION.json") throw Error("stub CLOSE retention failure");
        assert.ok(!retained.has(name), "Never overwrite a receipt");
        retained.set(name, JSON.parse(lines.slice(2, -1).map(line => {assert.ok(line.startsWith("+"));return line.slice(1);}).join("\n")));
        if (options.lateCaller && name === "VISUAL-04-quit-CLOSE-COMPLETION.json") now = quitCallerStart + 60001;
        return {};
      },
      async exec_command(args) {
        assert.equal(args.workdir, plan.case_path);
        if (args.cmd.includes("preview_launch.py")) {
          assert.ok(args.cmd.endsWith(" post-getter")); calls.push("post-getter");
          return terminal(0, JSON.stringify({app: plan.app, pid: 42, window_id: 99, same_observed_lease: true, checked_epoch: now / 1000}));
        }
        if (args.cmd.includes("preview_owner.py")) {
          const phase = args.cmd.split(" ").at(-1); calls.push("owner:" + phase);
          if (phase === "baseline" || phase === "close") return terminal();
          const metadata = retained.get("SEQUENCE-METADATA.json");
          const candidate = metadata && metadata.constructor_response === "VISUAL-03-create" && !metadata.preview_error && !metadata.quit_error && metadata.normal_quit_helper_completed;
          if (phase === "seal") return terminal(candidate ? 0 : 1);
          assert.equal(phase, "audit");
          return terminal(0, JSON.stringify({status: "PROVISIONAL_FINAL_AUDIT", provisional_candidate: !!candidate,
            case: plan.case, plan_sha256: plan.plan_sha256, deadline_epoch: plan.deadline_epoch, checked_epoch: now / 1000}));
        }
        const label = args.cmd.match(/'(VISUAL-[^']+)'/)?.[1]; assert.ok(label);
        const phase = args.cmd.includes(" reserve ") ? "GATE" : "CLOSE"; calls.push(phase + ":" + label);
        if (phase === "GATE") {begins.set(label, now); return terminal();}
        if (label === "VISUAL-04-quit" && options.failedClose) return terminal(1);
        const lifecycle = {label: label === "VISUAL-04-quit" && options.wrongCloseLabel ? "wrong-label" : label,
          bounded_lifecycle: true, closed_epoch: now / 1000, elapsed_through_persisted_receipt_seconds: (now - begins.get(label)) / 1000};
        return terminal(0, JSON.stringify(lifecycle) + "\n" + JSON.stringify({action_cutoff_epoch: plan.action_cutoff_epoch, deadline_epoch: plan.deadline_epoch}));
      },
      async write_stdin(args) {assert.ok([91, 81, 82].includes(args.session_id) && !polled.has(args.session_id)); polled.add(args.session_id); return terminal();},
      async mcp__computer_use__get_app_state(args) {assert.equal(args.app, plan.app);calls.push("MCP:state");now += 100;return state("initial");},
      async mcp__computer_use__click(args) {
        assert.equal(args.app, plan.app);now += 100;
        if (args.element_index === "17") {calls.push("MCP:frame");return options.frameError ? clone(observed) : state("framed");}
        assert.equal(args.element_index, "90");calls.push("MCP:create");return state("expanded");
      },
      async mcp__computer_use__press_key(args) {
        assert.equal(args.app, plan.app);assert.equal(args.key, options.wrongKey ? "super+w" : "super+q");calls.push("MCP:quit");now += 100;
        const response = clone(observed);
        if (options.otherError) response.content[0].text = "Computer Use server error -10005: Unknown error";
        if (options.extraText) response.content.push({type: "text", text: "Additional error"});
        return response;
      }
    }};
  vm.createContext(context);new vm.Script(helper).runInContext(context);new vm.Script(sequence).runInContext(context);
  if (options.wrongKey) {const original = context.nativeCall;context.nativeCall = request => {if (request.label === "VISUAL-04-quit") request.args.key = "super+w";return original(request);};}
  const task = context.continuousConstructorPreview();
  if (options.negative) await assert.rejects(task, /seal negative/);
  else {const result = await task;assert.equal(result.normal_close_accepted, true);assert.equal(result.alignment_accepted, false);}
  const metadata = retained.get("SEQUENCE-METADATA.json");assert.ok(metadata);
  assert.equal(metadata.normal_quit_helper_completed, !options.negative);
  if (!options.negative) {
    assert.equal(metadata.quit_response_qualification.kind, "EXACT_OBSERVED_APP_QUIT_ERROR_WITH_BOUNDED_CLOSE");
    assert.equal(metadata.quit_response_qualification.raw_isError, true);assert.equal(metadata.quit_error, null);
    assert.deepEqual(retained.get("VISUAL-04-quit.json"), observed);assert.ok(calls.includes("MCP:frame") && calls.includes("MCP:create"));
  } else assert.notEqual(stores.get("preview:" + plan.case_path + ":FINAL-ADMISSION")?.normal_close_accepted, true);
  assert.equal(calls.filter(call => call === "MCP:state").length, 1, "One getter only; none after Quit");
  assert.equal(calls.filter(call => call === "owner:close").length, 1);
  if (options.stale || options.frameError) assert.ok(!calls.includes("MCP:quit"));
  else assert.equal(calls.filter(call => call === "MCP:quit").length, 1);
  if (options.frameError) assert.match(metadata.preview_error, /Stop native authoring/);
  if (options.retentionFailure) assert.equal(stores.get(nativeKey("-CLOSE-COMPLETION.json")).exit_code, 0, "Stored zero alone cannot qualify failed retention");
  return {name, status: "passed_focused_stub", calls};
}

(async () => {
  const tests = [];
  for (const [name, options] of [
    ["different_bound_basename_and_exact_AppQuit_with_deserialized_receipts", {}],
    ["other_Quit_error_remains_negative", {otherError: true, negative: true}],
    ["extra_Quit_response_text_remains_negative", {extraText: true, negative: true}],
    ["missing_CLOSE_completion_remains_negative", {missingClose: true, negative: true}],
    ["caller_60s_includes_marker_preparation", {lateCaller: true, negative: true}],
    ["nonzero_CLOSE_remains_negative", {failedClose: true, negative: true}],
    ["wrong_Quit_key_remains_negative", {wrongKey: true, negative: true}],
    ["same_AppQuit_error_on_Frame_still_refused_by_generic_helper", {frameError: true, negative: true}],
    ["stale_per_label_result_refuses_Quit_replay", {stale: true, negative: true}],
    ["different_last_call_value_refuses_qualification", {lastMismatch: true, negative: true}],
    ["stored_zero_CLOSE_with_retention_failure_remains_negative", {retentionFailure: true, negative: true}],
    ["wrong_CLOSE_lifecycle_label_remains_negative", {wrongCloseLabel: true, negative: true}]
  ]) tests.push(await scenario(name, options));
  console.log(JSON.stringify({status: "PASSED_V6_FOCUSED_NODE_CALLER_CHECKS", count: tests.length, tests,
    native_call_sha256: crypto.createHash("sha256").update(helper).digest("hex"), observed_Quit_input_sha256: crypto.createHash("sha256").update(fs.readFileSync(observedPath)).digest("hex"),
    store_load_always_deserializes: true, actual_GUI_native_app_model_Cargo_network_calls: 0, node: process.version}, null, 2));
})().catch(error => {console.error(error.stack);process.exitCode = 1;});
