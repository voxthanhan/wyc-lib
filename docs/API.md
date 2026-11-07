# API hiện có và API mục tiêu

Utilities P1.1, primitives P1.2 và chỉ báo/state P1.3 bên dưới đã có implementation.
Các chữ ký analytical entry points/AnalysisState P1.6 vẫn là **thiết kế**, chưa xuất
thành stub. Result từng engine phải có kiểu rõ ràng, không trả generic JSON.

## Utilities P1.1

`wyc_lib::prepare_request(&AnalysisRequest) -> Result<PreparedAnalysis<'_>,
ContractError>` kiểm tra toàn snapshot, as_of/revision, config và module theo
thứ tự validation P0. Kết quả borrow input, không tính thuật toán. Các getter
`data()`, `config()`, `modules()`, `metadata()` chỉ đọc; modules được sắp theo
danh mục P1, không theo caller. Metadata dùng engine package version và
algorithm bundle `p1-v1`; hash bao gồm toàn bộ AnalysisConfig, không bao gồm
module selection hoặc as_of. Đổi module vẫn phải đổi state identity ở P1.6.

`contracts::AvailableData::new(&MarketData, as_of)` trả view borrow đã validate,
`len()/is_empty()/bar(index)` chỉ thấy tiền tố closed/available. `bar` trả
`Option<AvailableBar>` (None nếu ngoài prefix), các trường OHLCV là FiniteF64;
không sao chép snapshot, không có getter trả phần dữ liệu tương lai.
`known_at=max(revision_available_at, available_at[index])`.
`numeric_sample(index, NumericValue)` tạo sample đúng thời gian; ngoài prefix
trả invalid_result. `validate_numeric_series(&[NumericSample])` bắt buộc N
samples, timestamp/known_at đúng từng vị trí và warmup.available=index+1.
Ready=0 khác missing; không bỏ warm-up records. Kiểm tra required/first-valid
theo từng công thức, danh mục output đầy đủ và result module được bổ sung cùng
P1.2–P1.6; utilities không xác nhận giá trị thuật toán chưa triển khai.

`AnalysisConfig::canonical_config()` và `config_hash()` kiểm tra config trước
khi tạo canonical UTF-8 và SHA-256 theo CONTRACTS. `stable_event_id(&EventIdentity)`
kiểm tra các chuỗi định danh khác rỗng, config hash đúng định dạng config-v1,
timestamp trong miền contract và observed_at<=known_at rồi hash length-prefix
theo CONTRACTS. `EventIdentity` borrow từng trường; source_module là Module,
ordinal là u64. Không dùng as_of trong ID để cùng event giữ ID khi append/replay;
engine phát event chịu trách nhiệm known_at<=as_of và lineage của dependency.
Ordinal do quy tắc cung cấp, không có bộ đếm toàn cục hoặc phụ thuộc worker.

Các tiện ích canonical/ID và validated data view đặt tại contracts để các
crate thuật toán sau tái sử dụng, không phụ thuộc ngược vào wyc_lib.

## P1: Foundation

### Primitives P1.2 đã triển khai

Sáu hàm `rolling_sum`, `rolling_mean`, `rolling_min`, `rolling_max`,
`rolling_variance`, `rolling_std` nằm tại `indicators::rolling`, cũng truy cập
qua `wyc_lib::indicators::rolling`. Chữ ký chung:

```rust,ignore
fn rolling_sum(values: &[FiniteF64], period: usize)
    -> Result<Vec<NumericValue>, AnalysisError>;
```

Đây là API primitives, không nhận MarketData/as_of/config_hash. Caller chọn
tiền tố bằng AvailableData trước khi lấy chuỗi; NumericValue được gắn thời gian
bằng AvailableData::numeric_sample. Input borrow read-only, output owned dài N,
cửa sổ có mẫu hiện tại; n-1 warmup với required=n, available=i+1. Period ngoài
`[1,1000000]` trả invalid_config tại rolling.period, kể cả input rỗng. Input
hợp lệ rỗng trả rỗng. Kiểu FiniteF64 từ chối missing/non-finite ở biên.

Sum cộng bù Neumaier theo thời gian; mean=sum/n; min/max chính xác; variance
hai lượt với mẫu số n; std=sqrt(variance). Mỗi cửa sổ tính lại, O(N×min(n,N)),
phụ trợ O(1), output O(N); không có rolling state. Overflow trung gian trả
AnalysisError::NumericOverflow với `code()=NumericOverflow`, path
`rolling.<operation>[i]`. Không clip hoặc trả vector dở dang. Quy tắc số và
giới hạn f64 tại [đặc tả](algorithms/P1.md); kiểm chứng tại
[báo cáo P1.2](acceptance/P1.2.md).

### Chỉ báo nền/state P1.3 đã triển khai

Truy cập qua `indicators::basic` hoặc `wyc_lib::indicators::basic`:

```rust,ignore
fn sma(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError>;
// ema, wma, momentum, roc, volume_sma, rvol có cùng chữ ký.
fn true_range(high: &[FiniteF64], low: &[FiniteF64], close: &[FiniteF64])
    -> Result<Vec<NumericValue>, AnalysisError>;
fn atr(high: &[FiniteF64], low: &[FiniteF64], close: &[FiniteF64], period: usize)
    -> Result<Vec<NumericValue>, AnalysisError>;
fn calculate_basic(data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<BasicIndicatorResult>, AnalysisError>;
```

Slice APIs nhận prefix caller đã giới hạn; không tự biết as_of. Period kiểm
trước input rỗng, lỗi invalid_config tại indicators.period. High/low/close
cùng N và low<=close<=high, volume không âm. Seed/công thức/warm-up/overflow tại
[đặc tả P1.3](algorithms/P1.md). `calculate_basic` tự validate/select prefix,
trả owned metadata và BasicIndicatorResult có `series: BTreeMap<IndicatorKind,
Vec<NumericSample>>` với đúng Sma/Ema/Wma/Tr/Atr/Momentum/Roc/VolumeSma/Rvol;
không nhận modules, không chạy structure hoặc chỉ báo P1.4. Input rỗng trả đủ
chín series rỗng. Full AnalysisConfig dùng validation/hash đã có; result chỉ
là kiểu Rust, chưa thêm serialization/FFI.

`indicators::BasicIndicatorState` (cũng qua wyc_lib re-export) cung cấp:

```rust,ignore
fn new(data: &MarketData, config: &AnalysisConfig, as_of: i64) -> Result<Self, AnalysisError>;
fn result(&self) -> &ModuleResult<BasicIndicatorResult>;
fn update_closed_bar(&mut self, data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<BasicIndicatorResult>, AnalysisError>;
fn reset(&mut self, data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<BasicIndicatorResult>, AnalysisError>;
```

Update nhận **snapshot tích lũy**, khác ClosedBarUpdate dự kiến bên dưới.
State chỉ copy prefix đã khả dụng. Update yêu cầu đúng một nến mới đóng/khả
dụng; as_of không giảm; prefix cũ, config và metadata identity giữ nguyên.
Lỗi và thứ tự kiểm theo đặc tả: state_mismatch tại state.identity;
reset_required tại state.history/state.new_bar/metadata.gaps;
invalid_availability tại as_of/state.new_bar. Validation giữ code/path có sẵn.
Gap không tự suy từ timestamp. Reset có thể đổi data/config/as_of hợp lệ.
Mọi lỗi atomic, result cũ caller đã giữ không đổi. Batch và state dùng cùng
đường tính; chi phí mỗi update O(N×min(n,N)) + validation snapshot O(M),
bộ nhớ O(N). Chưa có state serialization hoặc tối ưu incremental O(1).
Kiểm chứng tại [báo cáo P1.3](acceptance/P1.3.md).

### Analytical entry points/state dự kiến P1.6

```rust,ignore
fn calculate_indicators(data: &MarketData, config: &IndicatorConfig, as_of: i64)
    -> Result<ModuleResult<IndicatorResult>, AnalysisError>;
fn analyze_structure(data: &MarketData, config: &StructureConfig, as_of: i64)
    -> Result<ModuleResult<StructureResult>, AnalysisError>;
fn analyze_batch(request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError>;
impl AnalysisState {
    fn new(request: &AnalysisRequest) -> Result<Self, AnalysisError>;
    fn update_closed_bar(&mut self, update: &ClosedBarUpdate)
        -> Result<AnalysisResult, AnalysisError>;
    fn reset(&mut self, request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError>;
}
```

Ownership: borrow input, owned immutable outputs. No input mutation; state owns
copies of necessary history. Result metadata: schema/engine/algorithm version,
symbol/timeframe, data_revision, config_hash, as_of; optional typed module outputs
chỉ có payload ở các module được yêu cầu (field module khác là null theo typed
AnalysisResult). Series output có N bản ghi tương ứng tiền tố
khả dụng, NumericSample cho mỗi vị trí (warm-up không bị bỏ). StructureResult có
pivots, labels, trend history, breaks và evidence theo danh mục P1.
Chỉ indicators không chạy structure; chỉ structure không tự tính indicators.
Dependency không có input bắt buộc trả missing_dependency; bản P1 hai module
không cần external input, nên không có trường hợp thiếu benchmark giả định.
Module chưa triển khai trả unsupported_module, không âm thầm bỏ qua.

`ClosedBarUpdate`: bar một hàng đủ OHLCV/availability, toàn metadata định danh,
schema/engine/config/module identity và `as_of`. Chỉ nhận nến closed,
timestamp > last_timestamp, available_at >= last availability và <= as_of,
as_of >= last_as_of. Không append vào state đang có nến đóng chưa khả dụng;
khởi tạo state chỉ giữ tiền tố đã khả dụng. Update trùng/đảo timestamp là lỗi,
đổi revision/config/units/source/symbol/timeframe/module là state_mismatch;
gap hoặc sửa lịch sử trả reset_required. Thất bại phải atomic: state không đổi.

`reset` xây state mới bằng batch trên snapshot hợp lệ rồi thay thế atomic, không
ghi đè các report trước mà caller đã giữ. Truy vấn as_of cũ dùng analyze_batch
và snapshot/revision đúng mốc; không có query quá khứ từ state mới hơn.
Update có thể batch tính lại trong P1; không cam kết O(1) hay throughput.
`AnalysisError` hiện bao bọc ContractError và numeric-overflow với code/path;
State errors sau này đi qua ContractError có mã state_mismatch/reset_required.
Không dùng panic cho dữ liệu hoặc lỗi dự kiến. Các typed output DTO P1 đã có
trong `contracts/src/result.rs`; engine cần validation toàn kết quả trước khi
xuất analytical entry points. P1.2 đã có
AnalysisError và validation primitives, chưa có engine tạo toàn result catalog.

## P3–P6: chữ ký khung và dependency

Các result/config/input sau còn **draft**, phải được cụ thể hóa khi tới phase.
Các hàm đều nhận borrowed typed input và trả owned Result, as_of bắt buộc.

| Phase | Chữ ký mục tiêu | Input/dependency bắt buộc |
| --- | --- | --- |
| P3 | `detect_price_zones(&ZoneInput, &ZoneConfig, as_of) -> Result<ZoneResult, AnalysisError>` | MarketData, confirmed pivots, ATR |
| P3 | `analyze_volume(&VolumeInput, &VolumeConfig, as_of) -> Result<VolumeResult, AnalysisError>` | OHLCV và rolling statistics |
| P3 | `analyze_relative_strength(&RsInput, &RsConfig, as_of) -> Result<RsResult, AnalysisError>` | symbol, benchmark, universe snapshot cùng mốc/revision policy |
| P4 | `analyze_wyckoff(&WyckoffInput, &WyckoffConfig, as_of) -> Result<WyckoffResult, AnalysisError>` | structure/zones/volume, completed higher-timeframe bars |
| P5 | `evaluate_evidence(&EvidenceInput, &EvidenceConfig, as_of) -> Result<DecisionReport, AnalysisError>` | typed facts/context/hypotheses với known_at <= as_of |
| P6 | `extract_features(&FeatureInput, &FeatureConfig, as_of) -> Result<FeatureResult, AnalysisError>` | typed results và feature schema/version |

P3+ mở rộng AnalysisRequest bằng schema version mới tương ứng với config và
auxiliary inputs có kiểu. Orchestrator chỉ chạy dependency cần thiết; caller
cung cấp dữ liệu ngoại sinh, Core không tải benchmark. Dependency bắt buộc thiếu
trả missing_dependency trước tính toán. Mỗi extension phải có batch/update/reset,
availability, golden và parity tại cổng của nó. Không ép spec tương lai thành
contract đã nghiệm thu hoặc tuyên bố phần chưa có implementation là chạy được.
