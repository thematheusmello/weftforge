'use strict';
const fs = require('node:fs');
const path = require('node:path');
const process = require('node:process');
const crypto = require('node:crypto');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const clock = () => ({ utc: new Date().toISOString(), monotonic_ns: process.hrtime.bigint().toString() });

function createRecorder(root = __dirname) {
  const trace = path.join(root, 'trace');
  const captures = path.join(root, 'captures');
  fs.mkdirSync(trace, { recursive: true, mode: 0o700 });
  fs.mkdirSync(captures, { recursive: true, mode: 0o700 });
  const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  function before(label, method, args = {}) {
    const ids = fs.readdirSync(trace).filter(n => /^\d+-before\.json$/.test(n)).map(n => Number(n.split('-')[0]));
    const id = String(Math.max(0, ...ids) + 1).padStart(4, '0');
    write(path.join(trace, id + '-before.json'), { id, label, method, arguments: args, ...clock() });
    return id;
  }
  function returned(id, result, error = null) {
    if (!/^\d{4,}$/.test(id) || !fs.existsSync(path.join(trace, id + '-before.json'))) throw new Error('Missing paired before record');
    write(path.join(trace, id + '-after.json'), { id, status: error ? 'exception' : 'returned', result: result === undefined ? null : result, error: error ? { name: error.name, message: error.message } : null, ...clock() });
    return result;
  }
  async function record(label, method, args, invoke) {
    const id = before(label, method, args);
    let result;
    try { result = await invoke(); } catch (error) { returned(id, null, error); throw error; }
    return returned(id, result);
  }
  function artifact(name, raw, sourceId) {
    if (!/^[A-Za-z0-9][A-Za-z0-9._-]*\.(txt|jpg|json)$/.test(name) || !fs.existsSync(path.join(trace, sourceId + '-after.json'))) throw new Error('Invalid artifact or unreturned source');
    const bytes = Buffer.isBuffer(raw) ? raw : Buffer.from(raw, 'utf8');
    fs.writeFileSync(path.join(captures, name), bytes, { flag: 'wx', mode: 0o600 });
    const receipt = { file: 'captures/' + name, bytes: bytes.length, sha256: hash(bytes), source_record: sourceId, unedited: true, ...clock() };
    write(path.join(captures, name + '.sha256.json'), receipt);
    return receipt;
  }
  return { before, returned, record, artifact };
}

module.exports = { createRecorder };
