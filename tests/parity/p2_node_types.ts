import { analyze_batch, calculate_indicators, analyze_structure, AnalysisState,
  AnalysisRequest, ClosedBarUpdate, NumericValue, IndicatorKind, State } from 'wyc-lib';
declare const request: AnalysisRequest;
declare const update: ClosedBarUpdate;
const batch = analyze_batch(request);
const indicators = calculate_indicators(request.data, request.config, request.as_of);
const structure = analyze_structure(request.data, request.config, request.as_of);
const kind: IndicatorKind = 'macd_histogram';
const numeric: NumericValue = indicators.output.series[kind][0].result;
if (numeric.status === 'ready') numeric.value.toFixed();
else if (numeric.status === 'warmup') numeric.required.toFixed();
else numeric.reason.toUpperCase();
structure.output.pivots.map(p => p.known_at);
const state = new AnalysisState(request);
state.update_closed_bar(update); state.identity(); state.result(); state.reset(request); state.close();
new State(JSON.stringify(request)).close();
if (batch.structure) batch.structure.breaks.map(b => b.strength.status);
// @ts-expect-error request must be a DTO, not a JSON string
analyze_batch('{}');
// @ts-expect-error missing integer timestamp
analyze_structure(request.data, request.config);
// @ts-expect-error numeric status must be narrowed
numeric.value;
// @ts-expect-error closed bar shape cannot be a batch request
state.update_closed_bar(request);
// @ts-expect-error unsupported indicator key
const wrong: IndicatorKind = 'invented';
