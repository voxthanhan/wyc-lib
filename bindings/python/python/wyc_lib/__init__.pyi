from typing import Any, Generic, Literal, Sequence, TypeVar, TypedDict
from types import TracebackType
import numpy as np
from numpy.typing import NDArray

Module = Literal['indicators', 'market_structure', 'price_zones', 'volume_engine', 'relative_strength', 'wyckoff_engine', 'evidence_engine', 'feature_engine']
NumericColumn = Sequence[float | int | None] | NDArray[np.float64] | NDArray[np.int64]
TimeColumn = Sequence[int] | NDArray[np.int64]
BoolColumn = Sequence[bool] | NDArray[np.bool_]

class Gap(TypedDict):
    after_timestamp: int
    before_timestamp: int

class MarketMetadata(TypedDict):
    source: str
    timezone: str
    price_unit: str
    volume_unit: str
    adjustment_mode: Literal['raw', 'split_adjusted', 'total_return_adjusted']
    timestamp_semantics: Literal['close']
    data_revision: str
    revision_available_at: int
    snapshot_kind: Literal['point_in_time', 'revised_history']
    gaps: Sequence[Gap]

class MarketData(TypedDict):
    schema_version: str
    symbol: str
    timeframe: str
    timestamp: TimeColumn
    available_at: TimeColumn
    open: NumericColumn
    high: NumericColumn
    low: NumericColumn
    close: NumericColumn
    volume: NumericColumn
    is_closed: BoolColumn
    metadata: MarketMetadata

class IndicatorConfig(TypedDict):
    period: int
    macd_fast: int
    macd_slow: int
    macd_signal: int
    bollinger_multiplier: float

class StructureConfig(TypedDict):
    left: int
    right: int

class AnalysisConfig(TypedDict):
    config_version: str
    indicators: IndicatorConfig
    structure: StructureConfig

class AnalysisRequest(TypedDict):
    schema_version: str
    data: MarketData
    config: AnalysisConfig
    as_of: int
    modules: Sequence[Module]

class Ready(TypedDict):
    status: Literal['ready']
    value: float

class Warmup(TypedDict):
    status: Literal['warmup']
    required: int
    available: int

class Missing(TypedDict):
    status: Literal['missing']
    reason: Literal['zero_denominator', 'unavailable_dependency']

NumericValue = Ready | Warmup | Missing

class NumericSample(TypedDict):
    observed_at: int
    known_at: int
    result: NumericValue

IndicatorKind = Literal['sum', 'sma', 'ema', 'wma', 'min', 'max', 'variance', 'std', 'tr', 'atr', 'momentum', 'roc', 'volume_sma', 'rvol', 'rsi', 'macd_line', 'macd_signal', 'macd_histogram', 'plus_di', 'minus_di', 'dx', 'adx', 'bollinger_middle', 'bollinger_upper', 'bollinger_lower']

class IndicatorResult(TypedDict):
    series: dict[IndicatorKind, list[NumericSample]]

class ResultMetadata(TypedDict):
    schema_version: str
    engine_version: str
    algorithm_version: str
    symbol: str
    timeframe: str
    data_revision: str
    config_hash: str
    as_of: int

T = TypeVar('T')
class ModuleResult(TypedDict, Generic[T]):
    metadata: ResultMetadata
    output: T

class Pivot(TypedDict):
    pivot_id: str
    kind: Literal['high', 'low']
    label: Literal['unclassified', 'hh', 'hl', 'lh', 'll', 'eh', 'el']
    level: float
    observed_at: int
    known_at: int

class TrendSample(TypedDict):
    observed_at: int
    known_at: int
    trend: Literal['unknown', 'uptrend', 'downtrend', 'sideways']

class StructureBreak(TypedDict):
    event_id: str
    reference_pivot_id: str
    kind: Literal['bos', 'choch']
    direction: Literal['up', 'down']
    level: float
    close: float
    strength: NumericValue
    observed_at: int
    known_at: int

class Evidence(TypedDict):
    evidence_id: str
    event_type: str
    observed_at: int
    known_at: int
    source_module: Module
    measured_values: dict[str, float]
    rule_id: str
    status: Literal['watching', 'candidate', 'confirmed', 'invalidated']
    layer: Literal['observation', 'context', 'hypothesis', 'confirmation']

class StructureResult(TypedDict):
    pivots: list[Pivot]
    trends: list[TrendSample]
    breaks: list[StructureBreak]
    evidence: list[Evidence]

class AnalysisResult(TypedDict):
    metadata: ResultMetadata
    indicators: IndicatorResult | None
    structure: StructureResult | None

class StateIdentity(TypedDict):
    schema_version: str
    engine_version: str
    algorithm_version: str
    symbol: str
    timeframe: str
    data_revision: str
    config_hash: str
    modules: Sequence[Module]
    last_timestamp: int | None
    last_as_of: int

class ClosedBar(TypedDict):
    timestamp: int
    available_at: int
    open: float | None
    high: float | None
    low: float | None
    close: float | None
    volume: float | None
    is_closed: bool

class ClosedBarUpdate(TypedDict):
    identity: StateIdentity
    metadata: MarketMetadata
    bar: ClosedBar
    as_of: int

def analyze_batch(request: AnalysisRequest) -> AnalysisResult: ...
def calculate_indicators(data: MarketData, config: AnalysisConfig, as_of: int) -> ModuleResult[IndicatorResult]: ...
def analyze_structure(data: MarketData, config: AnalysisConfig, as_of: int) -> ModuleResult[StructureResult]: ...
def market_data_from_pandas(frame: Any, *, symbol: str, timeframe: str, metadata: MarketMetadata, schema_version: str = ...) -> MarketData: ...
def market_data_from_polars(frame: Any, *, symbol: str, timeframe: str, metadata: MarketMetadata, schema_version: str = ...) -> MarketData: ...

class AnalysisState:
    def __init__(self, request: AnalysisRequest) -> None: ...
    def result(self) -> AnalysisResult: ...
    def identity(self) -> StateIdentity: ...
    def update_closed_bar(self, update: ClosedBarUpdate) -> AnalysisResult: ...
    def reset(self, request: AnalysisRequest) -> AnalysisResult: ...
    def close(self) -> None: ...
    def __enter__(self) -> AnalysisState: ...
    def __exit__(self, exc_type: type[BaseException] | None, exc_value: BaseException | None, traceback: TracebackType | None) -> None: ...

def analyze_json(request: str) -> str: ...
class State:
    def __init__(self, request: str) -> None: ...
    def result_json(self) -> str: ...
    def identity_json(self) -> str: ...
    def update_json(self, update: str) -> str: ...
    def reset_json(self, request: str) -> str: ...
    def close(self) -> None: ...
class WycStateError(RuntimeError):
    code: str
    path: str
class WycEngineError(RuntimeError):
    code: str
    path: str
