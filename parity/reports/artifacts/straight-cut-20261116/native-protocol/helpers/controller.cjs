'use strict';

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const process = require('node:process');
const types = require('node:util').types;

const RECORDER_SHA256 = '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc';
const METHODS = new Set([
  'getApp', 'getState', 'getAXState', 'getScreenshot', 'getAXStateAndScreenshot',
  'click', 'drag', 'scroll', 'selectText', 'setValue', 'performSecondaryAction',
  'paste', 'pressKey', 'typeText',
]);
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const clock = () => ({ utc: new Date().toISOString(), monotonic_ns: process.hrtime.bigint().toString() });
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', { flag: 'wx', mode: 0o600 });

function createController({ root, recorderPath, evidenceKind, callCap, elapsedCapS, baselineCalls = 1 }) {
  if (!['offline_mock', 'native'].includes(evidenceKind)
      || !Number.isSafeInteger(callCap) || callCap < 1
      || !Number.isSafeInteger(baselineCalls) || baselineCalls < 0 || baselineCalls > callCap
      || !Number.isFinite(elapsedCapS) || elapsedCapS <= 0) {
    throw new Error('Invalid controller configuration');
  }
  root = path.resolve(root);
  recorderPath = path.resolve(recorderPath);
  if (hash(fs.readFileSync(recorderPath)) !== RECORDER_SHA256) throw new Error('Recorder SHA mismatch');
  fs.mkdirSync(root, { recursive: true, mode: 0o700 });
  const config = { evidenceKind, callCap, elapsedCapS, baselineCalls, recorderPath, recorderSha256: RECORDER_SHA256 };
  const configPath = path.join(root, 'CONTEXT.json');
  if (fs.existsSync(configPath)) {
    if (JSON.stringify(JSON.parse(fs.readFileSync(configPath, 'utf8'))) !== JSON.stringify(config)) {
      throw new Error('Existing controller configuration differs');
    }
  } else {
    write(configPath, config);
  }
  const recorder = require(recorderPath).createRecorder(root);
  const clockPath = path.join(root, 'LAUNCH-CLOCK.json');
  const stopPath = path.join(root, 'STOP.json');
  const trace = path.join(root, 'trace');
  const captures = path.join(root, 'captures');

  function status() {
    const before = fs.readdirSync(trace).filter(name => /^\d+-before\.json$/.test(name));
    let elapsedS = null;
    if (fs.existsSync(clockPath)) {
      const start = JSON.parse(fs.readFileSync(clockPath, 'utf8'));
      elapsedS = Number(process.hrtime.bigint() - BigInt(start.monotonic_ns)) / 1e9;
    }
    return { evidenceKind, calls: baselineCalls + before.length, elapsedS, stopped: fs.existsSync(stopPath) };
  }

  function stop(error, context) {
    if (!fs.existsSync(stopPath)) {
      write(stopPath, { ...clock(), context, error: { name: error.name, message: error.message, stack: error.stack ?? null }, status: status() });
    }
  }

  function gate() {
    if (fs.existsSync(stopPath)) throw new Error('Controller stopped; no further target calls allowed');
    const current = status();
    let reason;
    if (current.elapsedS === null) reason = 'Launch clock not started';
    else if (current.elapsedS < 0 || current.elapsedS >= elapsedCapS) reason = 'Elapsed time cap reached';
    else if (current.calls >= callCap) reason = 'API call cap reached';
    else if (fs.readdirSync(trace).some(name => /^\d+-before\.json$/.test(name)
      && !fs.existsSync(path.join(trace, name.replace('-before.json', '-after.json'))))) reason = 'Unpaired prior target call';
    if (reason) {
      const error = new Error(reason);
      stop(error, { phase: 'before-target-call' });
      throw error;
    }
  }

  function startClock() {
    if (fs.existsSync(stopPath)) throw new Error('Controller stopped; clock cannot restart');
    const started = clock();
    write(clockPath, { ...started, evidenceKind });
    return started;
  }

  async function run(target, label, method, args, validate) {
    gate();
    if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$/.test(label) || !Array.isArray(args)) {
      const error = new Error('Invalid call label or arguments');
      stop(error, { phase: 'before-target-call' });
      throw error;
    }
    const id = recorder.before(label, method, { args, evidenceKind });
    let value;
    try {
      if (!METHODS.has(method)) throw new Error('Unknown target method: ' + method);
      if (!target || typeof target[method] !== 'function') throw new Error('Target lacks method: ' + method);
      value = await target[method](...args);
      if (validate) validate(value);
      recorder.returned(id, value);
      return { id, value };
    } catch (thrown) {
      const error = types.isNativeError(thrown) ? thrown : new Error(String(thrown));
      if (!fs.existsSync(path.join(trace, id + '-after.json'))) recorder.returned(id, value, error);
      stop(error, { phase: 'target-call-or-result', sourceRecord: id, label, method });
      throw error;
    }
  }

  function imagesFrom(value) {
    if (!value || typeof value.state !== 'string') throw new Error('Observation lacks AX text');
    const images = [];
    if (value.screenshot !== undefined) images.push({ bytes: value.screenshot });
    if (value.screenshots !== undefined) {
      if (!Array.isArray(value.screenshots)) throw new Error('Observation screenshots is not an array');
      images.push(...value.screenshots);
    }
    if (images.length === 0) throw new Error('Observation has no raw screenshots');
    return images.map((region) => {
      if (!region || !types.isUint8Array(region.bytes)) throw new Error('Screenshot is not raw byte data');
      const bytes = Buffer.from(region.bytes);
      if (bytes.length >= 3 && bytes[0] === 255 && bytes[1] === 216 && bytes[2] === 255) {
        return { bytes, extension: 'jpg', mimeType: 'image/jpeg' };
      }
      if (bytes.length >= 8 && bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))) {
        return { bytes, extension: 'png', mimeType: 'image/png' };
      }
      throw new Error('Unknown raw screenshot format');
    });
  }

  function imageArtifact(name, image, sourceId) {
    if (image.extension === 'jpg') return recorder.artifact(name + '.jpg', image.bytes, sourceId);
    // The pinned recorder excludes .png; retain PNG bytes under their real extension.
    const after = JSON.parse(fs.readFileSync(path.join(trace, sourceId + '-after.json'), 'utf8'));
    if (after.status !== 'returned') throw new Error('PNG source did not return successfully');
    const file = name + '.png';
    fs.writeFileSync(path.join(captures, file), image.bytes, { flag: 'wx', mode: 0o600 });
    const receipt = { file: 'captures/' + file, bytes: image.bytes.length, sha256: hash(image.bytes), source_record: sourceId, unedited: true, evidenceKind, ...clock() };
    write(path.join(captures, file + '.sha256.json'), receipt);
    return receipt;
  }

  async function call(target, label, method, args = []) {
    const result = await run(target, label, method, args);
    return { ...result, status: status() };
  }

  async function capture(target, label) {
    const result = await run(target, label, 'getAXStateAndScreenshot', [{ emit: false }], imagesFrom);
    try {
      const textReceipt = recorder.artifact(label + '.txt', result.value.state, result.id);
      const images = imagesFrom(result.value).map((image, i) => ({
        bytes: image.bytes,
        mimeType: image.mimeType,
        receipt: imageArtifact(label + '-' + i, image, result.id),
      }));
      write(path.join(captures, label + '-capture.json'), {
        evidenceKind, sourceRecord: result.id, textReceipt, images: images.map(image => image.receipt),
      });
      return { evidenceKind, sourceRecord: result.id, state: result.value.state, images, status: status() };
    } catch (error) {
      stop(error, { phase: 'artifact-save', sourceRecord: result.id, label });
      throw error;
    }
  }

  return { startClock, status, call, capture };
}

module.exports = { createController, RECORDER_SHA256 };
