'use strict';
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const lib = require('wyc-lib');
const location = require.resolve('wyc-lib');
const native = require(path.join(path.dirname(location), 'wyc_lib.node'));
// Same installed source and realm; expose only in this private measurement module.
const probe = new Module(location, module);
probe.filename = location;
probe.paths = module.paths;
probe._compile(fs.readFileSync(location, 'utf8') + '\nmodule.exports.probe = snapshot;', location);
const requests = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const rows = [], outputs = [];
let sink;
function measure(bars, operation, call) {
  for (let i = 0; i < 2; i++) sink = call();
  const samples = [];
  for (let i = 0; i < 5; i++) {
    const start = process.hrtime.bigint();
    for (let j = 0; j < 3; j++) sink = call();
    samples.push(Number(process.hrtime.bigint() - start) / 3);
  }
  rows.push({bars, operation, ns_per_op: samples});
}
for (const r of requests) {
  const before = JSON.stringify(r), bars = r.data.timestamp.length;
  const encoded = before, response = native.analyzeJson(encoded);
  if (!JSON.parse(response).ok) throw Error('native baseline failed');
  const batch = lib.analyze_batch(r);
  outputs.push({batch, indicators: lib.calculate_indicators(r.data, r.config, r.as_of),
    structure: lib.analyze_structure(r.data, r.config, r.as_of), raw: JSON.parse(response).value});
  const typed = structuredClone(r);
  for (const k of ['timestamp', 'available_at', 'open', 'high', 'low', 'close', 'volume']) {
    typed.data[k] = new Float64Array(r.data[k]);
  }
  outputs.at(-1).array_batch = lib.analyze_batch(typed);
  measure(bars, 'deepcopy_proxy', () => structuredClone(r));
  measure(bars, 'host_snapshot_encode', () => probe.exports.probe(r, 'request'));
  measure(bars, 'output_decode', () => JSON.parse(response));
  measure(bars, 'native_json_transport', () => native.analyzeJson(encoded));
  measure(bars, 'typed_batch', () => lib.analyze_batch(r));
  measure(bars, 'typed_two_calls', () => [lib.calculate_indicators(r.data, r.config, r.as_of),
    lib.analyze_structure(r.data, r.config, r.as_of)]);
  measure(bars, 'array_batch', () => lib.analyze_batch(typed));
  if (before !== JSON.stringify(r)) throw Error('input mutated');
  for (const k of ['timestamp', 'available_at', 'open', 'high', 'low', 'close', 'volume']) {
    if (JSON.stringify(Array.from(typed.data[k])) !== JSON.stringify(r.data[k])) throw Error('typed input mutated');
  }
}
if (sink === undefined) throw Error('missing sink');
console.log(JSON.stringify({runtime: 'node', version: process.version, rows, outputs,
  warmups: 2, samples: 5, iterations: 3, profile: 'release'}));
