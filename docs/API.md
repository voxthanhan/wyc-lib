# API hiện có và API mục tiêu

## Pilot P2.1

Contract đã chốt tại [ADR-0005](adr/0005-p2-ffi-pilot.md). Shared Rust transport
`wyc_binding_contract::analyze_json` và `Handle` được test trong root workspace;
`p2_reference` gọi trực tiếp Core để đối chiếu transport với cùng request.
Native package Node `wyc-lib` và Python import `wyc_lib` đã có mã adapter thử:

| API pilot | Input / output |
| --- | --- |
| `analyze_json(request_json)` | AnalysisRequest JSON string → AnalysisResult JSON string |
| `State(request_json)` | Khởi tạo state owned từ snapshot khả dụng |
| `state.result_json()` | Report owned hiện tại |
| `state.identity_json()` | StateIdentity JSON string, echo trong ClosedBarUpdate |
| `state.update_json(update_json)` | ClosedBarUpdate JSON string → report mới; lỗi atomic |
| `state.reset_json(request_json)` | Snapshot/revision mới → report mới; lỗi atomic |
| `state.close()` | Drop state, idempotent; không mở lại handle |

Đây là surface thử nghiệm JSON đồng bộ, không phải typed adapter P2.2/P2.3
hoặc API Foundation. Native ABI và cài npm/wheel đã chạy trên CI Windows/Linux;
local kiểm Rust/shared transport. Xem [trạng thái nghiệm thu](acceptance/P2.1.md). Wrapper chỉ chuyển
JSON/exception, không tính lại chỉ báo. Dùng chính protocol Core bên dưới;
không serialize cache/state và không dùng identity như con trỏ hoặc handle.

## Nền tảng Rust đã nghiệm thu

Utilities P1.1, primitives P1.2, chỉ báo/state P1.3, chỉ báo P1.4, structure P1.5
và analytical entry points/AnalysisState P1.6 đã có implementation Rust.
Result có kiểu rõ ràng; bindings thuộc P2. Replay, baseline và ví dụ Rust P1.7
được kiểm chứng trong [báo cáo G1](acceptance/G1.md).

## Ví dụ Rust P1.7

[p1_walkthrough.rs](../examples/p1_walkthrough.rs) nhận fixture OHLCV local,
tạo AnalysisRequest với config tường minh, khởi tạo state rỗng, append từng
nến đóng và đối chiếu batch cùng as_of. Sau đó khai revision mới, sửa lịch sử
và reset/replay, xuất cả report cũ và mới bằng JSON:

```text
cargo run --locked --package wyc-lib --example p1_walkthrough
```

Input ứng dụng thay fixture phải giữ đủ metadata/time/units theo CONTRACTS;
không dùng revision mới để trả lời as_of cũ. Mỗi update echo identity hiện tại;
snapshot sửa lịch sử dùng reset. Ví dụ có assertion chạy thật trong runner CI.
Protocol/đo baseline ở [benchmarks](../benchmarks/README.md), không đổi public API.

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

### RSI/MACD/ADX/Bollinger P1.4 đã triển khai

Truy cập qua `indicators::advanced` hoặc `wyc_lib::indicators::advanced`:

```rust,ignore
fn rsi(values: &[FiniteF64], period: usize) -> Result<Vec<NumericValue>, AnalysisError>;
fn macd(values: &[FiniteF64], fast: usize, slow: usize, signal: usize)
    -> Result<MacdResult, AnalysisError>;
fn adx(high: &[FiniteF64], low: &[FiniteF64], close: &[FiniteF64], period: usize)
    -> Result<AdxResult, AnalysisError>;
fn bollinger_bands(values: &[FiniteF64], period: usize, multiplier: FiniteF64)
    -> Result<BollingerResult, AnalysisError>;
fn calculate_advanced(data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<AdvancedIndicatorResult>, AnalysisError>;
```

MacdResult có line/signal/histogram; AdxResult có plus_di/minus_di/dx/adx;
BollingerResult có middle/upper/lower, mỗi field `Vec<NumericValue>` dài N.
AdvancedIndicatorResult có `series: BTreeMap<IndicatorKind, Vec<NumericSample>>`
với đúng 11 components. Slice APIs yêu cầu prefix đã giới hạn; calculate_advanced
validate toàn snapshot/config, chọn AvailableData và tự gắn lineage/hash.
Rỗng hợp lệ trả 11 series rỗng. Warm-up riêng từng component, lỗi và giới hạn
số tại [catalog](algorithms/P1.md). Kết quả là kiểu Rust, chưa serialization/FFI.
Không mở rộng BasicIndicatorState; state và điều phối toàn P1 còn ở P1.6.
Kiểm chứng tại [báo cáo P1.4](acceptance/P1.4.md).

### Market Structure P1.5 đã triển khai

Truy cập qua `market_structure` hoặc `wyc_lib::market_structure`:

```rust,ignore
fn calculate_structure(data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<StructureResult>, AnalysisError>;
```

Nhận full AnalysisConfig để validation/hash nhất quán với các engine P1.3/P1.4;
chỉ thực thi structure, không tính indicators. Input borrow read-only, output
owned; validate toàn snapshot/config rồi chọn prefix closed/available. Rỗng
hợp lệ trả bốn vector rỗng, ngắn trả N trend unknown samples và không bịa pivot.
Pivot strict L/R, labels, trend, close break, one-shot reference và strength
theo [catalog P1](algorithms/P1.md). StructureResult có serialization hiện có;
evidence pivot/break confirmed cùng ID/time, chỉ break có reference_pivot_id.
Lỗi input/config giữ code/path, xung đột hai break trả invalid_result,
overflow trả numeric_overflow; không trả output dở dang. Không có mutable state.
Re-export crate này chưa phải `wyc_lib::analyze_structure` của P1.6.
Kiểm chứng tại [báo cáo P1.5](acceptance/P1.5.md).

### Analytical entry points/state P1.6 đã triển khai

```rust,ignore
fn calculate_indicators(data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<IndicatorResult>, AnalysisError>;
fn analyze_structure(data: &MarketData, config: &AnalysisConfig, as_of: i64)
    -> Result<ModuleResult<StructureResult>, AnalysisError>;
fn analyze_batch(request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError>;
impl AnalysisState {
    fn new(request: &AnalysisRequest) -> Result<Self, AnalysisError>;
    fn result(&self) -> &AnalysisResult;
    fn identity(&self) -> &StateIdentity;
    fn history_len(&self) -> usize;
    fn update_closed_bar(&mut self, update: &ClosedBarUpdate)
        -> Result<AnalysisResult, AnalysisError>;
    fn reset(&mut self, request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError>;
}
```

Cả ba entry points và AnalysisState được export trực tiếp từ `wyc_lib`.
Hai API đơn module nhận **toàn AnalysisConfig**, khác chữ ký draft P0 dùng
IndicatorConfig/StructureConfig: giữ validation/hash nhất quán với engine đã
có và batch; không tạo default cấu hình ẩn. Quyết định trước code tại
[ADR-0003](adr/0003-p1-orchestration-state.md). API engine cũ vẫn giữ nguyên.

`calculate_indicators` trả đúng 25 IndicatorKind: Sum/Sma/Ema/Wma/Min/Max/
Variance/Std/Tr/Atr/Momentum/Roc/VolumeSma/Rvol/Rsi/MacdLine/MacdSignal/
MacdHistogram/PlusDi/MinusDi/Dx/Adx/BollingerMiddle/BollingerUpper/BollingerLower.
Rolling dùng close/period; Sma là rolling mean. Tái sử dụng basic/advanced/rolling,
không sao chép công thức. Kiểm 25 keys, N samples, lineage và required/first-valid
trước xuất kết quả. Structure gọi engine P1.5 với evidence đã validate.

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

`ClosedBarUpdate {identity: StateIdentity, metadata: MarketMetadata,
bar: ClosedBar, as_of: i64}`; `ClosedBar` gồm timestamp, available_at,
open/high/low/close/volume (Option<FiniteF64>) và is_closed. Hai DTO có serde,
deny_unknown_fields; JSON null OHLCV được decode rồi báo missing_data khi update.
Caller echo chính xác `state.identity().clone()` gồm last_timestamp/last_as_of
và modules canonical; cursor cũ trả state_mismatch. Chỉ nhận nến closed,
timestamp > last_timestamp, available_at >= last availability và <= as_of,
as_of >= last_as_of. Không append vào state đang có nến đóng chưa khả dụng;
khởi tạo state chỉ giữ tiền tố đã khả dụng. Update trùng/đảo timestamp là lỗi,
đổi revision/config/units/source/symbol/timeframe/module là state_mismatch;
gap khai báo trả reset_required. Protocol append không có history payload:
không thể sửa history qua update; caller phải đổi revision và reset khi sửa
lịch sử ngoài state. Core không phát hiện caller khai sai revision. Thất bại
atomic: toàn state không đổi. State chỉ copy prefix, không clone/giữ future.

Thứ tự lỗi update: gap → reset_required tại metadata.gaps; identity/metadata
khác → state_mismatch tại state.identity; as_of ngoài miền → invalid_timestamp
tại as_of; as_of giảm → invalid_availability tại as_of; nến chưa đóng →
invalid_availability tại state.new_bar. Batch validation sau append giữ code/path
của input (index trong history tích lũy); timestamp trùng/đảo không tự sửa.
Nến chưa khả dụng trả invalid_availability tại state.new_bar. Equality
available_at/as_of hợp lệ, chênh timestamp không tự được xem là gap.

`reset` xây state mới bằng batch trên snapshot hợp lệ rồi thay thế atomic, không
ghi đè các report trước mà caller đã giữ. Truy vấn as_of cũ dùng analyze_batch
và snapshot/revision đúng mốc; không có query quá khứ từ state mới hơn.
Update P1.6 **tính lại batch** trên history retained; indicators
O(N×min(n,N)), structure O(N×min(L+R+1,N)+N log N), bộ nhớ/output O(N), thêm
validation và sao chép. Engine được gọi lại có thể lặp validation/primitives;
không cam kết O(1) hoặc hiệu năng chưa đo. Không serialize mutable state.
`AnalysisError` hiện bao bọc ContractError và numeric-overflow với code/path;
State errors đi qua ContractError có mã state_mismatch/reset_required.
Không dùng panic cho dữ liệu hoặc lỗi dự kiến. Các typed output DTO P1 đã có
trong `contracts/src/result.rs`; engine cần validation toàn kết quả trước khi
xuất analytical entry points. P1.2 đã có
AnalysisError và validation primitives; toàn result catalog được ghép tại P1.6.

## State incremental sau G1

[ADR-0004](adr/0004-incremental-state.md) thay cách tính lại batch của P1.6
đã mô tả ở trên, giữ nguyên chữ ký và wire contracts. Append AnalysisState
validate một hàng bằng cùng helper batch, stage cache indicators/structure,
commit delta sau khi cả hai thành công; đường thành công không gọi analyze_batch.
EMA/Wilder giữ previous/seed, rolling chỉ dùng cửa sổ hiện tại theo đúng thứ tự
số. Structure giữ reference/consumption/trend và xác nhận pivot mới đủ R nến.
New/reset vẫn validate bằng batch rồi dựng cache từ prefix khả dụng.

BasicIndicatorState cũng dùng cache chín series, không tính advanced; snapshot
API vẫn validate toàn input/so history nên có chi phí O(M). Lỗi stage có thể
chạy batch để giữ canonical error priority, không thay cache/history/result.
AnalysisState giữ history/output O(N), clone report O(N), chèn evidence O(E);
chi phí tính sample O(period + seed windows), structure O(L+R+1), không O(1)
cho tổng API. Low-level IndicatorMemory::new/prepare và
StructureMemory::default/prepare cùng market_structure::append_delta là integration API Rust cho
prefix AvailableBar đã validate, append-only, cùng identity/config/revision;
caller không được sửa prefix hoặc tái dùng cache cho identity khác. Đây không
phải state serialization hay FFI handle. AvailableBar::from_closed chia sẻ
row validation với batch; không tự chứng minh provenance của metadata/history.
[Kết quả và baseline](acceptance/G1-incremental.md).

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
