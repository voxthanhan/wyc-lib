 'use strict';
// Host conversion tests only: injected native records input; never models Core.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
let calls = 0, captured;
const record = input => { calls++; captured = JSON.parse(input); return '{"ok":true,"value":null}'; };
const native = {analyzeJson: record, indicatorsJson: record, structureJson: record};
const moduleObject = {exports:{}};
vm.runInNewContext(fs.readFileSync(path.join(__dirname,'../../bindings/node/index.cjs'),'utf8'),
  {module: moduleObject, require: () => native, Float64Array, ArrayBuffer, Object, Array,
   TypeError, RangeError, Error, Set, Number, JSON, Reflect});
const api = moduleObject.exports;
let assertions = 0;
function bad(input, Type, code, expectedPath) {
  const before = calls;
  assert.throws(()=>api.analyze_batch(input),e=>e instanceof Type&&e.code===code&&e.path===expectedPath);
  assert.equal(calls,before); assertions++;
}
for (const value of [NaN,Infinity,-Infinity]) bad({data:{close:[value]}},RangeError,'non_finite','request.data.close[0]');
for (const value of [undefined,1n,()=>0]) bad({data:{close:[value]}},TypeError,'invalid_type','request.data.close[0]');
bad({as_of:9007199254740992},RangeError,'invalid_timestamp','request.as_of');
bad({as_of:1.5},TypeError,'invalid_type','request.as_of');
bad({data:{timestamp:new Float64Array([1.5])}},TypeError,'invalid_type','request.data.timestamp[0]');
bad({data:{close:[,1]}},TypeError,'invalid_type','request.data.close');
bad({data:{close:new Float32Array([1])}},TypeError,'invalid_type','request.data.close');
const cycle={};cycle.self=cycle;bad(cycle,TypeError,'invalid_type','request.self');
bad({toJSON(){throw Error('must not execute');}},TypeError,'invalid_type','request');
const getter={};Object.defineProperty(getter,'data',{enumerable:true,get(){throw Error('must not execute');}});
bad(getter,TypeError,'invalid_type','request');
const symbol={};symbol[Symbol()]=1;bad(symbol,TypeError,'invalid_type','request');
const extra=new Float64Array([1]);extra.toJSON=()=>[2];bad({data:{close:extra}},TypeError,'invalid_type','request.data.close');
for (const input of [null,undefined,[],1,'{}',new Date()]) bad(input,TypeError,'invalid_type','request');
const input={data:{close:new Float64Array([1,2]),timestamp:[9007199254740991]},as_of:6000};
api.analyze_batch(input);assert.deepEqual(captured,{data:{close:[1,2],timestamp:[9007199254740991]},as_of:6000});assertions++;
input.data.close[0]=99;assert.equal(captured.data.close[0],1);assertions++;
console.log(JSON.stringify({passed:true,assertions,scope:'host conversion only; no native ABI or analytical result claims'}));
