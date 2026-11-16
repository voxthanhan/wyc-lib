'use strict';
// Run after installing the target-specific tarball in this directory.
const api = require('wyc-lib');
const request = require('./request.json');
const batch = api.analyze_batch(request);
const state = new api.AnalysisState(request);
try {
  const indicators = api.calculate_indicators(request.data, request.config, request.as_of);
  const structure = api.analyze_structure(request.data, request.config, request.as_of);
  console.log(JSON.stringify({batch, indicators, structure, state: state.result()}));
} finally {
  state.close();
}
