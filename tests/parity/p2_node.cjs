 'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const api = require('wyc-lib');
const {actions, references} = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const clone = x => structuredClone(x);
const full = JSON.parse(actions.find(x => x.name === 'batch-full').input);
let assertions = 0;
function equal(a, b) { assert.deepStrictEqual(a, b); assertions++; }
function error(call, Type, code, path) {
  assert.throws(call, e => e instanceof Type && e.code === code && e.path === path); assertions++;
}
function prefix(n) {
  const r = clone(full);
  for (const k of ['timestamp','available_at','open','high','low','close','volume','is_closed']) r.data[k] = r.data[k].slice(0,n);
  r.as_of = n * 1000; return r;
}
for (const name of ['batch-full','batch-empty','unicode','zero-volume']) {
  const r = JSON.parse(actions.find(x => x.name === name).input);
  const before = clone(r), expected = references[name];
  equal(api.analyze_batch(r), expected); equal(r, before);
  equal(api.calculate_indicators(r.data,r.config,r.as_of), {metadata: expected.metadata,output: expected.indicators});
  equal(api.analyze_structure(r.data,r.config,r.as_of), {metadata: expected.metadata,output: expected.structure});
}
const typed = clone(full);
for (const k of ['timestamp','available_at','open','high','low','close','volume']) typed.data[k] = new Float64Array(typed.data[k]);
equal(api.analyze_batch(typed), references['batch-full']);
for (const modules of [['indicators'],['market_structure']]) {
  const r = clone(full); r.modules = modules; const result = api.analyze_batch(r);
  equal(result[modules[0] === 'indicators' ? 'structure' : 'indicators'],null);
}
for (const v of [undefined, null, '{}', [], 1, true, new Date()]) error(() => api.analyze_batch(v),TypeError,'invalid_type','request');
for (const v of [NaN,Infinity,-Infinity]) {
  const r = clone(full); r.data.close[0] = v;
  error(() => api.analyze_batch(r),RangeError,'non_finite','request.data.close[0]');
}
for (const v of [undefined, 1n, () => 1]) {
  const r = clone(full); r.data.close[0] = v;
  error(() => api.analyze_batch(r),TypeError,'invalid_type','request.data.close[0]');
}
for (const v of [1.5, 1n]) { const r=clone(full);r.as_of=v;error(()=>api.analyze_batch(r),TypeError,'invalid_type','request.as_of'); }
{const r=clone(full);r.as_of=9007199254740992;error(()=>api.analyze_batch(r),RangeError,'invalid_timestamp','request.as_of');}
{const r=clone(full);delete r.data.close[0];error(()=>api.analyze_batch(r),TypeError,'invalid_type','request.data.close');}
{const r=clone(full);r.data.close=new Float32Array(r.data.close);error(()=>api.analyze_batch(r),TypeError,'invalid_type','request.data.close');}
{const r=clone(full);r.self=r;error(()=>api.analyze_batch(r),TypeError,'invalid_type','request.self');}
{const r=clone(full);Object.defineProperty(r,'as_of',{get(){throw Error('getter ran');},enumerable:true});error(()=>api.analyze_batch(r),TypeError,'invalid_type','request');}
{const r=clone(full);r.toJSON=()=>full;error(()=>api.analyze_batch(r),TypeError,'invalid_type','request');}
{const r=clone(full);r[Symbol('hidden')]=1;error(()=>api.analyze_batch(r),TypeError,'invalid_type','request');}
// Rust controls shape, enum, unknown fields and domain validation.
for (const [change,Type,code,path] of [
  [r=>{r.extra=1;},TypeError,'invalid_type','request'],
  [r=>{delete r.config;},TypeError,'invalid_type','request'],
  [r=>{r.modules=['unknown'];},TypeError,'invalid_type','request'],
  [r=>{r.modules=['price_zones'];},RangeError,'unsupported_module','modules'],
  [r=>{r.data.close.pop();},RangeError,'length_mismatch','close'],
  [r=>{r.data.close[0]=null;},TypeError,'missing_data','close[0]'],
  [r=>{r.data.volume[0]=-1;},RangeError,'negative_volume','volume[0]'],
  [r=>{r.data.timestamp[1]=r.data.timestamp[0];},RangeError,'duplicate_timestamp','timestamp[1]'],
  [r=>{r.config.indicators.period=0;},RangeError,'invalid_config','config.indicators.period'],
  [r=>{r.data.close[0]=1e308;r.data.high[0]=1e308;},api.WycEngineError,'numeric_overflow',null],
]) {
  const r=clone(full);change(r);
  if(path) error(()=>api.analyze_batch(r),Type,code,path);
  else { assert.throws(()=>api.analyze_batch(r),e=>e instanceof Type&&e.code===code);assertions++; }
}
const original=prefix(3);const state=new api.AnalysisState(original), other=new api.AnalysisState(original);
original.data.close[0]=999; equal(state.result(),references['initial-result']);
const saved=state.result();saved.metadata.symbol='mutated';const identity=state.identity();identity.symbol='mutated';
equal(state.result(),references['initial-result']);
for(let n=3;n<6;n++) {
  const r=prefix(n+1), bar={};for(const k of ['timestamp','available_at','open','high','low','close','volume','is_closed']) bar[k]=r.data[k][n];
  const update={identity:state.identity(),metadata:r.data.metadata,bar,as_of:r.as_of};
  const before=state.result();const cursor=state.identity();
  const bad=clone(update);bad.bar.close=NaN;
  error(()=>state.update_closed_bar(bad),RangeError,'non_finite','update.bar.close');
  const mismatch=clone(update);mismatch.identity.symbol='wrong';
  error(()=>state.update_closed_bar(mismatch),api.WycStateError,'state_mismatch','state.identity');
  equal(state.result(),before);equal(state.identity(),cursor);
  equal(state.update_closed_bar(update),references[`append-${n}`]);
  equal(state.result(),api.analyze_batch(r));equal(before.metadata.as_of,n*1000);
}
equal(other.result(),references['initial-result']);
const invalid=clone(full);invalid.config.indicators.period=0;
error(()=>state.reset(invalid),RangeError,'invalid_config','config.indicators.period');
equal(state.result(),references['batch-full']);
const revised=JSON.parse(actions.find(x=>x.name==='revision-reset').input);
equal(state.reset(revised),references['revision-reset']);equal(other.result(),references['initial-result']);
state.close();state.close();
for(const call of [()=>state.result(),()=>state.identity(),()=>state.reset(undefined),()=>state.update_closed_bar(undefined)]) error(call,api.WycStateError,'state_closed','state');
other.close();
for(let i=0;i<100;i++) new api.AnalysisState(prefix(1));
if(global.gc)global.gc();
console.log(JSON.stringify({passed:true,assertions,gc:'100 abandoned handles; no crash, not a leak measurement'}));
