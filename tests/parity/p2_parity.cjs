'use strict';
// Copy this driver into the external consumer; resolve only the installed npm package.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const api = require('wyc-lib');
const actions = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
let state;
const records = [];
for (const a of actions) {
  const before = structuredClone(a.input);
  try {
    let value;
    const r = a.input;
    switch (a.op) {
      case 'batch': value = api.analyze_batch(r); break;
      case 'indicators': value = api.calculate_indicators(r.data, r.config, r.as_of); break;
      case 'structure': value = api.analyze_structure(r.data, r.config, r.as_of); break;
      case 'new': if (state) state.close(); state = new api.AnalysisState(r); value = state.result(); break;
      case 'reset': value = state.reset(r); break;
      case 'update': {
        const update = structuredClone(r);
        update.identity = {...state.identity(), ...(a.identity_patch || {})};
        value = state.update_closed_bar(update); break;
      }
      case 'result': value = state.result(); break;
      case 'identity': value = state.identity(); break;
      default: throw Error('unknown parity operation');
    }
    records.push({name:a.name, value});
  } catch (e) {
    const exception_kind = e instanceof api.WycStateError ? 'state' : e instanceof api.WycEngineError ? 'engine' :
      e instanceof TypeError ? 'type' : e instanceof RangeError ? 'range' : 'unexpected';
    records.push({name:a.name, error:{code:e.code, path:e.path}, exception_kind});
  }
  assert.deepStrictEqual(a.input, before, `input changed: ${a.name}`);
}
if (state) state.close();
process.stdout.write(JSON.stringify(records));
