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
function value(response) {
  const data = JSON.parse(response);
  if (!data.ok) {
    const {kind, code, path} = data.error;
    const Type = {type: TypeError, range: RangeError, state: WycStateError, engine: WycEngineError}[kind];
    throw Object.assign(new Type(`${code} at ${path}`), {code, path});
  }
  return data.value;
}
function unwrap(response) { return JSON.stringify(value(response)); }
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
function fail(Type, code, path) {
  throw Object.assign(new Type(`${code} at ${path}`), {code, path});
}
const integerFields = new Set(['as_of', 'timestamp', 'available_at', 'revision_available_at',
  'after_timestamp', 'before_timestamp', 'last_timestamp', 'last_as_of',
  'period', 'macd_fast', 'macd_slow', 'macd_signal', 'left', 'right']);
const timeFields = new Set(['as_of', 'timestamp', 'available_at', 'revision_available_at',
  'after_timestamp', 'before_timestamp', 'last_timestamp', 'last_as_of']);
function snapshot(input, root) {
  const active = new Set();
  function copy(v, path, field, depth) {
    if (depth > 64) fail(TypeError, 'invalid_type', path);
    if (v === null || typeof v === 'string' || typeof v === 'boolean') return v;
    if (typeof v === 'number') {
      if (!Number.isFinite(v)) fail(RangeError, 'non_finite', path);
      if (integerFields.has(field) && !Number.isSafeInteger(v)) {
        fail(timeFields.has(field) && Number.isInteger(v) ? RangeError : TypeError,
          timeFields.has(field) && Number.isInteger(v) ? 'invalid_timestamp' : 'invalid_type', path);
      }
      return v;
    }
    if (typeof v !== 'object' || active.has(v)) fail(TypeError, 'invalid_type', path);
    active.add(v);
    try {
      if (v instanceof Float64Array) {
        if (Object.getPrototypeOf(v) !== Float64Array.prototype ||
            Reflect.ownKeys(v).some(k => typeof k !== 'string' || !/^(0|[1-9][0-9]*)$/.test(k))) {
          fail(TypeError, 'invalid_type', path);
        }
        if (!['timestamp', 'available_at', 'open', 'high', 'low', 'close', 'volume'].includes(field)) {
          fail(TypeError, 'invalid_type', path);
        }
        return Array.from(v, (x, i) => copy(x, `${path}[${i}]`, field, depth + 1));
      }
      const array = Array.isArray(v);
      if (!array && Object.getPrototypeOf(v) !== Object.prototype && Object.getPrototypeOf(v) !== null) {
        fail(TypeError, 'invalid_type', path);
      }
      const descriptors = Object.getOwnPropertyDescriptors(v);
      const keys = Reflect.ownKeys(descriptors);
      const out = array ? [] : Object.create(null);
      if (array && keys.length !== v.length + 1) fail(TypeError, 'invalid_type', path);
      for (const key of keys) {
        if (array && key === 'length') continue;
        const d = descriptors[key];
        if (typeof key !== 'string' || !d.enumerable || !('value' in d) || key === 'toJSON' ||
            (array && !/^(0|[1-9][0-9]*)$/.test(key))) fail(TypeError, 'invalid_type', path);
        out[key] = copy(d.value, array ? `${path}[${key}]` : `${path}.${key}`, array ? field : key, depth + 1);
      }
      return out;
    } finally { active.delete(v); }
  }
  if (!input || typeof input !== 'object' || Array.isArray(input) || ArrayBuffer.isView(input)) {
    fail(TypeError, 'invalid_type', root);
  }
  return JSON.stringify(copy(input, root, '', 0));
}
function single(call, data, config, as_of) {
  return value(call(snapshot({schema_version: '0.1.0', data, config, as_of, modules: []}, 'request')));
}
class AnalysisState {
  #native;
  #closed = false;
  constructor(request) {
    this.#native = new native.NativeState();
    value(this.#native.resetJson(snapshot(request, 'request')));
  }
  #live() { if (this.#closed) fail(WycStateError, 'state_closed', 'state'); }
  result() { this.#live(); return value(this.#native.resultJson()); }
  identity() { this.#live(); return value(this.#native.identityJson()); }
  update_closed_bar(update) { this.#live(); return value(this.#native.updateJson(snapshot(update, 'update'))); }
  reset(request) { this.#live(); return value(this.#native.resetJson(snapshot(request, 'request'))); }
  close() { this.#native.close(); this.#closed = true; }
}
module.exports = {
  analyze_batch: request => value(native.analyzeJson(snapshot(request, 'request'))),
  calculate_indicators: (data, config, as_of) => single(native.indicatorsJson, data, config, as_of),
  analyze_structure: (data, config, as_of) => single(native.structureJson, data, config, as_of),
  AnalysisState,
  analyze_json: request => unwrap(native.analyzeJson(input(request, 'request'))), State, WycStateError, WycEngineError};
