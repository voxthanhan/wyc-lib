'use strict';
// Run from the consumer directory: require the installed tarball, never source.
const fs = require('node:fs');
const api = require('wyc-lib');
const actions = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const states = new Map();
const records = [];
for (const action of actions) {
  try {
    let value = null;
    const state = states.get(action.id || 'a');
    switch (action.op) {
      case 'analyze': value = JSON.parse(api.analyze_json(action.input)); break;
      case 'new': states.set(action.id || 'a', new api.State(action.input)); break;
      case 'result': value = JSON.parse(state.result_json()); break;
      case 'identity': value = JSON.parse(state.identity_json()); break;
      case 'reset': value = JSON.parse(state.reset_json(action.input)); break;
      case 'update': {
        let input = action.input;
        if (action.echo) {
          const update = JSON.parse(input);
          update.identity = JSON.parse(state.identity_json());
          if (action.wrong_identity) update.identity.symbol = 'different';
          input = JSON.stringify(update);
        }
        value = JSON.parse(state.update_json(input));
        break;
      }
      case 'mutate': {
        const output = JSON.parse(state.result_json());
        output.metadata.symbol = 'caller-mutation';
        value = JSON.parse(state.result_json());
        break;
      }
      case 'close': state.close(); break;
      case 'gc':
        for (let i = 0; i < 100; ++i) new api.State(action.input);
        if (global.gc) global.gc();
        value = 'gc-no-crash';
        break;
      default: throw new Error('unknown smoke operation');
    }
    records.push({name: action.name, value});
  } catch (error) {
    const type = error instanceof api.WycStateError ? 'state' :
      error instanceof api.WycEngineError ? 'engine' :
      error instanceof TypeError ? 'type' : error instanceof RangeError ? 'range' : 'unexpected';
    records.push({name: action.name, error: {kind: type, code: error.code, path: error.path}});
  }
}
process.stdout.write(JSON.stringify(records));
