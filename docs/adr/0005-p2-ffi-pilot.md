# ADR-0005 — FFI v1 và thử nghiệm P2.1

Ngày: 14/11/2026. Trạng thái: chốt contract P2.1; chưa nghiệm thu G2.

## Quyết định

Giữ Core, schema `0.1.0`, config-v1, event-v1 và algorithm `p1-v1`.
Thêm lớp dùng chung `bindings/ffi` chỉ decode DTO, gọi `wyc_lib`, encode và
phân loại lỗi. Node dùng NAPI-RS; Python dùng PyO3/Maturin. Không có công thức
hoặc state machine phân tích ở adapter. Pilot đồng bộ, có copy, không zero-copy,
không release GIL hoặc tạo worker. Mỗi handle sở hữu một AnalysisState Rust.

P2.1 thử đường `analyze_json(request_json)` và `State(request_json)` với
`result_json()`, `identity_json()`, `update_json(update_json)`,
`reset_json(request_json)`, `close()`. Đây là API pilot nhận **chuỗi JSON**,
chưa phải typed API P2.2/P2.3, NumPy/Pandas/Polars, Next.js hoặc bản phân phối
Foundation. Tên package thử nghiệm vẫn là `wyc-lib`/import `wyc_lib`, private,
không publish. Tất cả API phân tích gọi cùng Rust Core đã nghiệm thu.

## Mapping đã chốt

| Nội dung | Rust | Node.js / Python / JSON |
| --- | --- | --- |
| Giá, volume, measured values | FiniteF64/f64 | number / float / finite binary64; giữ precision f64, không làm tròn; -0 chuẩn hóa 0 |
| Timestamp, available_at, as_of | i64 | number integer / int / JSON integer, miền `[0,9007199254740991]`; UTC ms, close semantics |
| Period, lookback | u32 | integer `[1,1000000]`; không bool, float token hoặc chuỗi số |
| Length/index/warm-up | usize | integer không âm trong miền JS safe integer; không truyền pointer/usize ABI |
| OHLCV thiếu | Option<FiniteF64> | JSON null được decode rồi Core trả missing_data theo code/path hiện có |
| Kết quả | NumericValue tagged | ready có value; warmup có required/available; missing có reason; không sentinel NaN/0/null |
| Module không chọn | Option | JSON null; module được chọn trên input rỗng vẫn có container hợp lệ |
| Config | AnalysisConfig | mọi trường tường minh; deny_unknown_fields; canonical/hash tại Rust, không default ẩn |
| Request/update/result | DTO contracts | schema và snake_case giữ nguyên; output là chuỗi JSON owned, caller có thể parse |
| Identity | StateIdentity | snapshot JSON để echo cursor khi update; không phải handle hay state serialization |

Pilot yêu cầu JSON integer token cho trường integer (`6000`, không `6000.0`).
JSON lỗi cú pháp, NaN/Infinity hoặc số overflow parser trả `invalid_json`;
sai shape/type, thiếu trường, unknown field/enum hoặc integer ngoài kiểu Rust
trả `invalid_type`. Path decode ổn định là `request` hoặc `update`; không lấy
thông báo serde làm contract. Integer decode được nhưng ngoài miền thời gian
trả `invalid_timestamp` với path Core. Trailing JSON bị từ chối. Chuỗi Unicode
giữ nguyên. Không tự sort, fill, sửa hoặc thay đổi input. Pilot không nhận
object/typed array trực tiếp nên không có JSON.stringify(NaN) thành null.

## Lỗi

Native transport nội bộ trả envelope `{"ok":true,"value":...}` hoặc
`{"ok":false,"error":{"kind","code","path"}}`. Envelope không là AnalysisResult.
Wrapper bỏ envelope, trả JSON của value hoặc throw exception có `.code`, `.path`.
Thông báo exception chỉ dùng chẩn đoán; không chứa payload đầu vào.

| kind / nhóm code | Node | Python |
| --- | --- | --- |
| type: invalid_json, invalid_type, missing_data | TypeError | TypeError |
| range: các lỗi validation Core còn lại | RangeError | ValueError |
| state: state_mismatch, reset_required, state_closed, state_uninitialized | WycStateError extends Error | WycStateError extends RuntimeError |
| engine: numeric_overflow, invalid_result, binding_failure | WycEngineError extends Error | WycEngineError extends RuntimeError |

`code` Core giữ chính xác snake_case; `path` Core giữ nguyên, không thêm prefix.
Sai kiểu argument ngoài chuỗi bị wrapper từ chối với invalid_type tại request/update.
Không trả partial result khi thất bại. Decode/validation lỗi không đổi handle.

## Ownership và vòng đời

Input string được copy/decode sang owned DTO trước khi gọi Core. Không giữ
tham chiếu JS/Python/NumPy hay con trỏ do caller cấp. Output là string mới;
sửa object đã parse không đổi state hoặc report đã trả trước. Native object
giữ `Option<AnalysisState>`; GC/finalizer drop Rust state. `close()` drop ngay,
idempotent và vĩnh viễn; mọi thao tác sau close trả state_closed **trước decode**.
Handle mới nội bộ chưa khởi tạo trả state_uninitialized khi đọc/update;
wrapper chỉ trả State sau reset khởi tạo thành công. Reset thất bại giữ state cũ.
Không registry/global ID, không raw pointer, không clone/serialize/share handle
giữa runtime/worker/process; tạo handle riêng cho mỗi workload. Node chạy sync;
PyO3 kiểm mutable borrow, giữ GIL. Không có concurrent update trên cùng handle.
Append/reset/identity/revision/atomicity dùng đúng protocol Core tại
[API](../API.md); query lịch sử dùng batch, không dùng state tương lai.

## Nền tảng và nghiệm thu

Rust 1.85.1; napi 2.16.17/derive 2.16.13/build 2.1.3 (Node-API v8),
PyO3 0.23.5 (CPython-specific ABI), Maturin 1.8.3. Dependencies exact và lockfile;
giấy phép NAPI-RS MIT, PyO3/Maturin MIT OR Apache-2.0. Mục tiêu pilot: Windows MSVC x64,
Ubuntu 24.04 GNU x64; Node 22.22.0 (CI) và Node local được ghi theo lần chạy;
CPython 3.10.9. Python 3.11–3.13, macOS/ARM64, musl, abi3, browser/Edge/WASM
chưa được công bố hỗ trợ. Wheel Linux dùng native tag, không claim manylinux.

Chạy Rust tests và smoke cài npm tarball/wheel ngoài source tree; đối chiếu
fixture/golden P1 độc lập, warm-up/missing/time, lỗi decode/range, cursor,
update/reset atomic, close/GC và output sở hữu riêng. CI chạy cùng runner trên
hai OS và lưu commit, version, command, exit code, package checksums, smoke log.
P2.4 vẫn chịu trách nhiệm parity đầy đủ mọi fixtures/API; P2.5–P2.6 chịu trách
nhiệm ma trận phân phối, server integration và baseline FFI.

## Kết quả P2.1

Hai native Cargo.lock đã lưu trong nguồn, khớp nguyên văn Windows/Linux.
Commit cf6c464 đã tích hợp main; CI Core 43578612827 và native 43578612862
đạt hai OS, không bootstrap, --locked và checkout sạch trước/sau. Local đạt
101 Rust tests + 5 controls, 1008 traces giữ nguyên; native local chưa chạy
do shell hạn chế mạng. [Báo cáo P2.1](../acceptance/P2.1.md) ghi phạm vi
nghiệm thu và giới hạn; không phải nghiệm thu G2/Foundation.
