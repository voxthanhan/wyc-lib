/** P1 owned/copy-safe DTOs. Timestamps are safe integer UTC milliseconds. */
export type Module = 'indicators' | 'market_structure' | 'price_zones' | 'volume_engine' | 'relative_strength' | 'wyckoff_engine' | 'evidence_engine' | 'feature_engine';
export type NumericColumn = readonly (number | null)[] | Float64Array;
export type TimeColumn = readonly number[] | Float64Array;
export interface MarketMetadata {
  readonly source: string; readonly timezone: string; readonly price_unit: string; readonly volume_unit: string;
  readonly adjustment_mode: 'raw' | 'split_adjusted' | 'total_return_adjusted';
  readonly timestamp_semantics: 'close'; readonly data_revision: string;
  readonly revision_available_at: number; readonly snapshot_kind: 'point_in_time' | 'revised_history';
  readonly gaps: readonly { readonly after_timestamp: number; readonly before_timestamp: number }[];
}
export interface MarketData {
  readonly schema_version: string; readonly symbol: string; readonly timeframe: string;
  readonly timestamp: TimeColumn; readonly available_at: TimeColumn;
  readonly open: NumericColumn; readonly high: NumericColumn; readonly low: NumericColumn;
  readonly close: NumericColumn; readonly volume: NumericColumn; readonly is_closed: readonly boolean[];
  readonly metadata: MarketMetadata;
}
export interface AnalysisConfig {
  readonly config_version: string;
  readonly indicators: { readonly period: number; readonly macd_fast: number; readonly macd_slow: number;
    readonly macd_signal: number; readonly bollinger_multiplier: number };
  readonly structure: { readonly left: number; readonly right: number };
}
export interface AnalysisRequest {
  readonly schema_version: string; readonly data: MarketData; readonly config: AnalysisConfig;
  readonly as_of: number; readonly modules: readonly Module[];
}
export type NumericValue = { status: 'ready'; value: number } |
  { status: 'warmup'; required: number; available: number } |
  { status: 'missing'; reason: 'zero_denominator' | 'unavailable_dependency' };
export interface NumericSample { observed_at: number; known_at: number; result: NumericValue; }
export type IndicatorKind = 'sum' | 'sma' | 'ema' | 'wma' | 'min' | 'max' | 'variance' | 'std' |
  'tr' | 'atr' | 'momentum' | 'roc' | 'volume_sma' | 'rvol' | 'rsi' | 'macd_line' | 'macd_signal' |
  'macd_histogram' | 'plus_di' | 'minus_di' | 'dx' | 'adx' | 'bollinger_middle' | 'bollinger_upper' | 'bollinger_lower';
export interface IndicatorResult { series: Record<IndicatorKind, NumericSample[]>; }
export interface ResultMetadata {
  schema_version: string; engine_version: string; algorithm_version: string; symbol: string;
  timeframe: string; data_revision: string; config_hash: string; as_of: number;
}
export interface ModuleResult<T> { metadata: ResultMetadata; output: T; }
export interface Pivot {
  pivot_id: string; kind: 'high' | 'low'; label: 'unclassified' | 'hh' | 'hl' | 'lh' | 'll' | 'eh' | 'el';
  level: number; observed_at: number; known_at: number;
}
export interface TrendSample { observed_at: number; known_at: number; trend: 'unknown' | 'uptrend' | 'downtrend' | 'sideways'; }
export interface StructureBreak {
  event_id: string; reference_pivot_id: string; kind: 'bos' | 'choch'; direction: 'up' | 'down';
  level: number; close: number; strength: NumericValue; observed_at: number; known_at: number;
}
export interface Evidence {
  evidence_id: string; event_type: string; observed_at: number; known_at: number; source_module: Module;
  measured_values: Record<string, number>; rule_id: string;
  status: 'watching' | 'candidate' | 'confirmed' | 'invalidated';
  layer: 'observation' | 'context' | 'hypothesis' | 'confirmation';
}
export interface StructureResult { pivots: Pivot[]; trends: TrendSample[]; breaks: StructureBreak[]; evidence: Evidence[]; }
export interface AnalysisResult { metadata: ResultMetadata; indicators: IndicatorResult | null; structure: StructureResult | null; }
export interface StateIdentity {
  readonly schema_version: string; readonly engine_version: string; readonly algorithm_version: string;
  readonly symbol: string; readonly timeframe: string; readonly data_revision: string; readonly config_hash: string;
  readonly modules: readonly Module[]; readonly last_timestamp: number | null; readonly last_as_of: number;
}
export interface ClosedBar {
  readonly timestamp: number; readonly available_at: number; readonly open: number | null;
  readonly high: number | null; readonly low: number | null; readonly close: number | null;
  readonly volume: number | null; readonly is_closed: boolean;
}
export interface ClosedBarUpdate {
  readonly identity: StateIdentity; readonly metadata: MarketMetadata; readonly bar: ClosedBar; readonly as_of: number;
}
export function analyze_batch(request: AnalysisRequest): AnalysisResult;
export function calculate_indicators(data: MarketData, config: AnalysisConfig, as_of: number): ModuleResult<IndicatorResult>;
export function analyze_structure(data: MarketData, config: AnalysisConfig, as_of: number): ModuleResult<StructureResult>;
export class AnalysisState {
  constructor(request: AnalysisRequest);
  result(): AnalysisResult;
  identity(): StateIdentity;
  update_closed_bar(update: ClosedBarUpdate): AnalysisResult;
  reset(request: AnalysisRequest): AnalysisResult;
  close(): void;
}
/** Backward-compatible P2.1 JSON pilot. */
export function analyze_json(request: string): string;
export class State {
  constructor(request: string);
  result_json(): string;
  identity_json(): string;
  update_json(update: string): string;
  reset_json(request: string): string;
  close(): void;
}
export class WycStateError extends Error { code: string; path: string; }
export class WycEngineError extends Error { code: string; path: string; }
