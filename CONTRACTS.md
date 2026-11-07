# wyc-lib — Contracts 0.1.0

## Phạm vi và phiên bản

Nguồn kiểu Rust: `crates/contracts/src/`. JSON UTF-8 dùng tên trường snake_case,
enum là chuỗi snake_case; field lạ, thiếu field bắt buộc và enum lạ bị từ chối
khi deserialize. Deserialize kiểm tra shape/type; gọi `validate()` để kiểm tra
ngữ nghĩa trước khi dùng. Hai bước không được coi là tương đương. Rust structs
có field công khai để xây dữ liệu; không được coi struct vừa tạo là đã validate.
`schema_version = "0.1.0"`, `config_version = "p1-v1"`; bản đầu trước 1.0,
thay đổi kết quả/contract phải đổi phiên bản thích hợp và ghi changelog.
Evidence được module P1.5 phát từ quy tắc đã chốt; bộ tổng hợp Evidence Engine
và DecisionReport vẫn là thiết kế P5, chưa có bộ đánh giá tổng hợp.

## MarketData

Shape dạng cột: `symbol`, `timeframe`, `timestamp`, `available_at`, `open`, `high`,
`low`, `close`, `volume`, `is_closed`, `metadata`, `schema_version`.
Mọi cột có cùng độ dài `N`; `N=0` hợp lệ (không đồng nghĩa lỗi hay số 0).
Mã, timeframe và metadata chuỗi phải khác rỗng sau trim; giữ nguyên chuỗi đầu vào.
Timeframe/timezone là nhãn do ứng dụng chuẩn hóa, không được Core diễn giải thành
lịch hoặc tự xác minh cơ sở dữ liệu timezone. `source` chỉ dùng truy vết.

| Trường | Quy định |
| --- | --- |
| `timestamp[i]` | Unix milliseconds UTC tại biên **đóng nến**, khoảng nến `[open_time, timestamp)` |
| `available_at[i]` | Mốc dữ liệu nến có thể dùng; không sớm hơn `timestamp[i]` |
| `is_closed[i]` | Trạng thái đóng trong snapshot; `true` chưa đủ chứng minh khả dụng tại `as_of` |
| OHLC | `Option<FiniteF64>`; JSON null nhận diện được và validate trả `missing_data` |
| `volume` | Như giá nhưng không âm; 0 hợp lệ |
| `metadata` | `source`, `timezone`, `price_unit`, `volume_unit`, `adjustment_mode`, `timestamp_semantics`, `data_revision`, `revision_available_at`, `snapshot_kind`, `gaps` |

`timestamp_semantics` chỉ nhận `close`; đầu vào timestamp mở phải được ứng dụng
chuyển trước. Đơn vị ví dụ `VND/share`, `share`; không có phép đổi đơn vị trong
Core. Giá hữu hạn có thể bằng 0 hoặc âm (để không khóa contract vào cổ phiếu);
`low <= open,close <= high`, `low <= high`. Công thức yêu cầu mẫu số khác 0 phải
trả missing khi không xác định, không biến thành lỗi OHLC.
`adjustment_mode`: `raw`, `split_adjusted`, `total_return_adjusted`.

Thứ tự validation xác định: version → metadata → revision timestamp → độ dài
cột → từng hàng theo thứ tự timestamp/availability/missing/OHLC/volume → gaps.
Trả lỗi đầu tiên với `code` và `path`, không sắp xếp, loại trùng hay sửa input.
Timestamp tăng nghiêm ngặt; availability không giảm; không được có nến đóng sau
một nến chưa đóng. Đây là contract **prefix liên tục**, chưa hỗ trợ late arrival
chen vào giữa. `available_len(as_of)` trả độ dài tiền tố thỏa cả ba điều kiện
`timestamp <= as_of`, `available_at <= as_of`, `is_closed = true`.
Validate toàn snapshot kể cả phần tương lai: invariance chỉ áp dụng khi phần nối
thêm vẫn là dữ liệu hợp lệ. Kết quả không dùng giá/volume tương lai để tính toán.

`gaps` gồm `{after_timestamp, before_timestamp}` theo thứ tự biên tăng. Bản đầu
từ chối mọi gap khai báo với `declared_gap`; ứng dụng cung cấp đoạn liên tục mới
và reset/replay. Chênh timestamp tự nó **không** chứng minh thiếu nến; cuối tuần,
ngày nghỉ và session không được suy ra. Danh sách gaps rỗng là tuyên bố của caller.

## Khả dụng và revision

Ứng dụng đặt `data_revision` ổn định cho một phiên bản lịch sử; append tương lai
không sửa lịch sử có thể giữ revision. `revision_available_at` là mốc **phiên bản
lịch sử** đó được biết, không phải mốc tải toàn file. `snapshot_kind` là
`point_in_time` khi ứng dụng có bằng chứng availability, hoặc `revised_history`
khi đang phân tích lại dữ liệu sửa/điều chỉnh. Core không xác minh nguồn gốc này.
Yêu cầu `as_of >= revision_available_at`; không dùng revision mới trả lời quá khứ.
Không tuyên bố point-in-time cho dữ liệu chỉ có lịch sử đã chỉnh hiện tại.

Ví dụ trong `tests/fixtures/temporal.json`: nến tại 2000 đã đóng nhưng đến 2500
mới khả dụng. Tại 2000 chỉ nến 1000 được dùng; tại 2500 được hai nến. Nến 4000
chưa đóng vẫn bị loại tại 9000. Revision mới biết tại 6000 bị từ chối ở 5500;
ở 6000 có thể reset/replay trên revision mới. Core không lưu kho lịch sử revision.
Pivot ở 2000 cần nến phải 3000 có availability 3500: `observed_at=2000`,
`known_at=3500`; không được có bằng chứng xác nhận trước 3500.

## AnalysisRequest và cấu hình P1

`{schema_version, data, as_of, modules, config}`. Caller giữ quyền sở hữu dữ liệu;
API nhận borrow read-only, kết quả tương lai là owned. Request validate gọi data
validation/availability rồi config và module selection. Danh sách module khác
rỗng, không trùng; P0 validate nhận `indicators`, `market_structure`, còn module
tương lai trả `unsupported_module`. P1.1 có `validate_and_select()` và
`wyc_lib::prepare_request()` để chuẩn bị context; **chưa có API phân tích**.
Modules trong prepared context có thứ tự `indicators`, `market_structure`,
không sửa thứ tự input của caller. `AvailableData` chỉ cho đọc tiền tố hợp lệ,
`bar(index)` trả None ngoài prefix. Metadata được tạo owned và chỉ có getter
read-only, không dùng giờ hệ thống hoặc dữ liệu tương lai để chọn kết quả.

`AnalysisConfig`: `config_version`, `indicators`, `structure`.
`IndicatorConfig`: period=14, macd_fast=12, macd_slow=26, macd_signal=9,
bollinger_multiplier=2. `StructureConfig`: left=2, right=2.
Các period/left/right thuộc `[1, 1000000]`, fast < slow, multiplier hữu hạn >0.
JSON phải ghi đầy đủ field; `Default` Rust chỉ là tiện ích tạo cấu hình tường minh.
P1 indicators trả toàn bộ danh mục P1 với cấu hình chung này; structure chỉ dùng
OHLC nên không có dependency indicator bắt buộc. P3 trở đi chốt typed config
và auxiliary inputs tại cổng tương ứng; không nhét dữ liệu tùy ý vào P1.

## Giá trị số và lỗi

`FiniteF64` từ chối NaN/±Infinity khi xây hoặc deserialize; chuẩn hóa -0 về +0.
Không rounding giá hoặc số trung gian. Mọi phép toán phân tích phải kiểm tra
overflow/non-finite và trả lỗi có thể xử lý, không panic/Infinity hoặc JSON null
giả thành công. Dung sai công thức trong `docs/algorithms/P1.md`.

`NumericValue` là tagged union, không dùng NaN hoặc sentinel 0:

```json
[
  {"status":"ready","value":0.0},
  {"status":"warmup","required":3,"available":2},
  {"status":"missing","reason":"zero_denominator"}
]
```

Warm-up: thiếu số nến cần cho seed/lookback, `available < required`.
`NumericValue::validate()` kiểm tra `required > 0` và bất biến này;
`NumericSample::validate_at()` kiểm tra thời gian và trạng thái, trả
`invalid_result` khi không hợp lệ. P1.1 thêm `AvailableData::numeric_sample()`
và `validate_numeric_series()`: N samples, observed_at đúng nến, known_at bằng
max(revision_available_at, available_at[i]), warmup.available=i+1. Không bỏ
warm-up hoặc thay missing thành 0. Các result container khác vẫn là DTO; engine
P1 chịu trách nhiệm kiểm tra required/first-valid theo công thức, danh mục output
và toàn bộ shape/lineage trước khi trả analytical results ở P1.2–P1.6.
P1.4 có `calculate_advanced` trả riêng 11 series RSI, MACD line/signal/histogram,
PlusDi/MinusDi/Dx/Adx và Bollinger middle/upper/lower. RSI/DI/DX required=n+1,
ADX=2n, MACD line=slow, signal/histogram=slow+signal-1, bands=n. Chuỗi hằng
RSI=50, DI/DX/ADX=0 là ready theo quy ước, không missing. Chữ ký/shape/lỗi
và giới hạn sai số Bollinger gần triệt tiêu ở thang rất lớn được ghi tại
[catalog P1](docs/algorithms/P1.md). BasicIndicatorState không tự mở rộng sang
P1.4; state chung còn thuộc P1.6. Schema/config/hash không đổi.

Missing: đủ nến nhưng phép tính không xác định hoặc dependency chưa có;
`unavailable_dependency` là missing của một output, khác lỗi request thiếu input
bắt buộc. Invalid input: `Result::Err(ContractError {code,path})`, không trả
NumericValue thay thế. `NumericSample` gắn `observed_at` và `known_at`.

| Nhóm | Phản ví dụ | Kết quả |
| --- | --- | --- |
| Schema/type | field lạ/thiếu, enum lạ, số vượt f64 | deserialize error |
| Version/metadata | version khác, symbol rỗng | `unsupported_version` / `invalid_metadata` |
| Shape | close có N-1 phần tử | `length_mismatch`, path `close` |
| Timestamp | âm, >2^53-1; trùng; đảo | `invalid_timestamp` / `duplicate_timestamp` / `unordered_timestamp` |
| Availability | available < close hoặc giảm; nến đóng sau nến mở | `invalid_availability` |
| Missing | close=null | `missing_data`, không tự điền |
| Numeric | NaN/Infinity | `non_finite` khi xây FiniteF64; JSON không hợp lệ khi decode |
| OHLC/volume | high < low; volume=-1 | `invalid_ohlc` / `negative_volume` |
| Gaps/revision | gap khai báo; revision đến sau as_of | `declared_gap` / `revision_not_available` |
| Config/modules | period=0; module lặp/tương lai | `invalid_config` / `duplicate_module` / `unsupported_module` |

Error enum còn dự trù `missing_dependency`, `state_mismatch`, `reset_required`
cho P1+; semantics tại `docs/API.md`. Chưa có binding ánh xạ lỗi ở P0.

P1.2 bổ sung `AnalysisError` cho phép tính: `Contract(ContractError)` giữ nguyên
lỗi validation, `NumericOverflow {path}` có code `numeric_overflow` để phân
biệt với `non_finite` của input. `code()`/`path()` là định danh machine-readable;
Display chỉ để chẩn đoán, không là contract. `ErrorCode` thêm numeric_overflow;
không đổi shape JSON, config/hash, schema hoặc algorithm bundle đã có. Bindings
P2 sẽ ánh xạ lỗi; hiện chưa có wire serialization riêng cho AnalysisError.

Primitives `indicators::rolling` nhận `&[FiniteF64]` và period `[1,1000000]`,
trả `Vec<NumericValue>` dài N với n-1 warm-up, không có missing tự sinh. Không
nhận as_of hoặc MarketData; caller phải dùng prefix khả dụng. Sum có bù số,
variance dân số hai lượt, từng cửa sổ tính lại độc lập; lỗi trung gian không
hữu hạn hủy toàn phép gọi, không sửa input. Quy tắc cửa sổ, underflow, overflow,
độ phức tạp và thứ tự số tại [đặc tả P1.2](docs/algorithms/P1.md).

P1.3 bổ sung `indicators::basic` cho SMA/EMA/WMA/TR/ATR/Momentum/ROC/Volume SMA/
RVOL. `calculate_basic(data, config, as_of)` validate toàn data/config, chỉ
tính prefix khả dụng và trả `ModuleResult<BasicIndicatorResult>` với đúng
chín series trong BTreeMap theo IndicatorKind. Result Rust có kiểu, chưa có
wire serialization riêng; không thay schema IndicatorResult của toàn P1.
Warm-up n cho các trung bình/ATR/RVOL, n+1 cho Momentum/ROC, 1 cho TR.
ROC/RVOL có missing zero_denominator và có thể ready trở lại ở nến tiếp theo.
Không dùng sentinel 0/NaN. Samples, metadata/config hash và overflow theo
contract trên; công thức và thứ tự số tại đặc tả P1.3 cùng tài liệu thuật toán.

BasicIndicatorState giữ prefix owned, toàn AnalysisConfig, MarketMetadata và
result/version. Update nhận snapshot tích lũy cùng config/as_of, yêu cầu đúng
một nến mới, history/identity không đổi và as_of không giảm; chỉ giữ prefix,
không giữ future. Lỗi identity là state_mismatch, sửa/xóa history hoặc gap cần
reset_required; lỗi validation giữ nguyên code/path. Update/reset thất bại
atomic, reset thành công tính lại batch trên revision/config mới. State tính
lại batch mỗi update; không phải AnalysisState/ClosedBarUpdate của P1.6.
Không có state serialization để tái dùng qua phiên bản engine/algorithm.

## Result có kiểu cho P1

`AnalysisResult` có `metadata: ResultMetadata`, `indicators: Option<IndicatorResult>`,
`structure: Option<StructureResult>`; None/JSON null nghĩa module không được yêu
cầu. Module được yêu cầu trên input rỗng có container rỗng hợp lệ khi P1 triển
khai; P0 không có hàm tạo kết quả phân tích. Metadata gồm schema/engine/algorithm
version, symbol, timeframe, data_revision, config_hash, as_of.
API đơn module dùng `ModuleResult<T> {metadata, output}` với cùng metadata.

IndicatorResult dùng `series: BTreeMap<IndicatorKind, Vec<NumericSample>>`; enum
định danh đúng từng output trong catalog P1. Mỗi series có độ dài bằng tiền tố
khả dụng, có đầy đủ warm-up records. Thứ tự key wire của serde/BTreeMap là thứ
tự enum khai báo trong `result.rs`, ổn định theo schema; không dựa hash map.
StructureResult có pivots (ID/kind/label/level/time), trends (trend/time tại mỗi
nến), breaks (ID/reference/kind/direction/level/close/strength/time), evidence.
Label enum bao gồm HH/HL/LH/LL/EH/EL và unclassified theo rule P1. Các output có
kiểu này là contract cho analytical entry points; P1.3 chỉ tạo
BasicIndicatorResult riêng cho chín chỉ báo nền, chưa tạo toàn bộ
IndicatorResult từ orchestrator. P1.5 `market_structure::calculate_structure`
đã tạo ModuleResult<StructureResult>: pivot/label/trend, break, strength và evidence
confirmed theo `structure-strict-close-v1`. Validate toàn snapshot/config trước,
N trend samples, không xuất pivot chưa xác nhận. Pivot/break ID dùng event-v1,
evidence cùng ID/time; rule/event_type/ordinal, thứ tự, missing strength và lỗi
xung đột/overflow tại [catalog P1](docs/algorithms/P1.md). Giữ wire shape/version
cũ, không có state P1.5 hoặc serialization state.

## Evidence và DecisionReport

Evidence có `evidence_id`, `event_type`, `observed_at`, `known_at`, `source_module`,
`measured_values` (BTreeMap tên → FiniteF64), `rule_id`, `status`, `layer`.
`layer` phân biệt observation/context/hypothesis/confirmation; `status` nhận
watching/candidate/confirmed/invalidated. `validate_at` kiểm tra ID/rule/type khác
rỗng và `0 <= observed_at <= known_at <= as_of <= 2^53-1`.
Module tạo evidence chịu trách nhiệm `known_at >= max(availability của mọi input
và xác nhận cần thiết)`. P1.5 bảo đảm lineage từ cửa sổ pivot và reference/trend
đã biết trước nến break; không phải bộ kiểm lineage tổng quát của các DTO caller tạo.

DecisionReport nền gồm report_id, schema/engine/algorithm version, symbol,
timeframe, as_of, data_revision, config_hash, hypothesis, status, evidence,
counter_evidence, pending_conditions, confirmation, invalidation. Hai điều kiện
sau là null hoặc `{rule_id,evidence_ids,observed_at,known_at}`. Thời gian null khi
chưa xảy ra. P5 phải chốt integrity references và guards chuyển trạng thái trước
khi đánh giá; P0 chỉ kiểm tra serialization mẫu, không xác nhận report hợp lệ
về nghiệp vụ. `confirmed` chỉ là thỏa quy tắc; không phải xác suất hay dự báo.

## Hash, ID runtime P1.1; state và FFI dự kiến P1/P2

Canonical config v1: UTF-8 ASCII, LF, mỗi dòng `key=value`, theo đúng thứ tự
`config_version`, `period`, `macd_fast`, `macd_slow`, `macd_signal`,
`bollinger_multiplier_bits`, `left`, `right`, có LF cuối file. Số nguyên decimal
không leading zero; multiplier dùng 16 chữ hex lowercase của `f64::to_bits()`
(vd 2.0 = `4000000000000000`). Không phụ thuộc thứ tự JSON hoặc locale.
`config_hash = "sha256:config-v1:" + lowercase_hex(SHA256(bytes))`.
P1.1 thực thi bằng `AnalysisConfig::canonical_config()`/`config_hash()`,
validate trước hashing; bao gồm toàn bộ config, không phụ thuộc modules/as_of.
Config sai không được cấp hash; không đổi canonical spec đã chốt tại P0.

Event ID v1: `sha256:event-v1:` + SHA256 của serialization length-prefixed từng
trường UTF-8: engine_version, algorithm_version, symbol, timeframe, data_revision,
config_hash, source_module, rule_id, event_type, observed_at, known_at, ordinal.
Prefix mỗi trường là chiều dài byte decimal + `:`; timestamps/ordinal decimal.
Ordinal là thứ tự phân giải quy tắc đã đặc tả, không phải thứ tự worker/hash map.
P1.1 cung cấp `stable_event_id(&EventIdentity)`, timestamps i64 và ordinal u64;
module dùng wire string chính xác. Chuỗi định danh không rỗng sau trim, giữ
nguyên UTF-8 đầu vào; length là số bytes, không phải số ký tự. Hash config phải
đúng prefix config-v1 và 64 hex lowercase. Sai metadata trả invalid_metadata,
timestamp sai trả invalid_timestamp, observed_at>known_at trả invalid_evidence.
Hàm không kiểm chứng lineage/as_of: module phát event phải kiểm tra những điều
đó. Bộ vectors độc lập gồm Unicode, dấu hai chấm và u64 ordinal cực đại.
Result order: observed_at, known_at, source_module wire string, rule_id, event_id;
output module order theo danh mục phase, không theo thứ tự caller.

`StateIdentity` ghi version, symbol/timeframe, revision, config_hash, tập module
chuẩn hóa, last_timestamp, last_as_of. State phải ràng buộc thêm toàn metadata
đầu vào; đổi source/units/adjustment cũng reset. Protocol trong API spec.

FFI dự kiến: giá f64, timestamp i64 bị chặn trong miền JS integer chính xác,
độ dài là usize nội bộ và integer không âm qua wire; mảng cột một chiều,
owned/copy-safe ở P2, không zero-copy mặc định. Node dùng number + typed status,
Python float/int + typed status; JSON dùng tagged object như trên. Input null
được báo missing_data. Mapping dự kiến Node TypeError/RangeError cho type/range,
Python TypeError/ValueError tương ứng, kèm code/path; engine/state errors có
error class riêng. Chi tiết ABI và handle đóng/finalize phải chốt ở P2.1.
