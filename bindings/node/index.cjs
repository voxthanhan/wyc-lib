'use strict';
const native = require('./wyc_lib.node');
class WycStateError extends Error {}
class WycEngineError extends Error {}
function input(value, path) {
  if (typeof value !== 'string') {
    throw Object.assign(new TypeError(`invalid_type at ${path}`), {code: 'invalid_type', path});
  }
  return value;
}
function unwrap(response) {
  const data = JSON.parse(response);
  if (!data.ok) {
    const {kind, code, path} = data.error;
    const Type = {type: TypeError, range: RangeError, state: WycStateError, engine: WycEngineError}[kind];
    throw Object.assign(new Type(`${code} at ${path}`), {code, path});
  }
  return JSON.stringify(data.value);
}
class State {
  #native;
  constructor(request) {
    this.#native = new native.NativeState();
    unwrap(this.#native.resetJson(input(request, 'request')));
  }
  result_json() { return unwrap(this.#native.resultJson()); }
  identity_json() { return unwrap(this.#native.identityJson()); }
  update_json(update) { return unwrap(this.#native.updateJson(input(update, 'update'))); }
  reset_json(request) { return unwrap(this.#native.resetJson(input(request, 'request'))); }
  close() { this.#native.close(); }
}
module.exports = {analyze_json: request => unwrap(native.analyzeJson(input(request, 'request'))), State, WycStateError, WycEngineError};
