# wyc-lib — Kiến trúc thực thi

## Hiện trạng P1.7

Workspace có bốn package: `contracts` (wire types, cấu hình, kiểm tra contract,
AnalysisError), `indicators` (rolling/statistics P1.2, chín chỉ báo nền và
BasicIndicatorState P1.3, RSI/MACD/ADX/Bollinger P1.4), `market_structure`
(pivot/labels/trend và BOS/CHoCH P1.5) và `wyc-lib`
(`wyc_lib` trong Rust, re-export ba crate, `prepare_request`, ba entry points
và AnalysisState P1.6). API P3–P6 trong `WYC_LIB.md` còn là mục tiêu;
bindings thuộc P2. Không tạo crate rỗng cho các phase sau.

```text
Application → bindings (P2) → wyc_lib → các crate thuật toán (P1–P6)
                                  ↘ contracts ↗
P1.1: wyc_lib → contracts → serde / sha2
P1.2: wyc_lib → indicators → contracts
P1.5: wyc_lib → market_structure → contracts
```

`contracts` không phụ thuộc ngược vào `wyc_lib`. Các thuật toán tương lai dùng
contracts; orchestrator chuyển typed results theo dependency graph của kế hoạch.
Wyckoff không phụ thuộc Feature Engine; Evidence Engine không gọi ngược Wyckoff.
Ứng dụng cung cấp dữ liệu, lịch giao dịch, snapshot, authentication và lưu trữ.
Core không gọi mạng, lấy giờ hệ thống, đọc secrets, tự điều chỉnh hoặc sửa dữ liệu.

## Test và dependency

`crates/wyc_lib/Cargo.toml` khai báo test target `foundation` trỏ tới
`tests/foundation.rs`, chạy thực sự trong `cargo test --workspace`.
Test target `p1_foundation` chạy `tests/p1_foundation.rs` cho phần runtime P1.1.
`indicators` có target `p1_rolling` trỏ tới `tests/p1_rolling.rs` cho sáu
primitives và replay cùng AvailableData; target `p1_basic` chạy chín chỉ báo,
batch/state, identity/reset và causal replay. Tất cả chạy trong workspace tests.
Target `p1_advanced` kiểm 11 components P1.4, golden độc lập, warm-up,
availability/revision và replay mọi tiền tố. Target `p1_structure` của wyc-lib
kiểm P1.5 trên 15 fixtures và ca lỗi. Target `p1_orchestration` kiểm API/state
toàn P1, golden cũ, chọn module, identity và atomic errors; verifier buộc bảy targets.
`tests/support/mod.rs` chứa so sánh JSON/số và replay contract, không phải engine.
`serde` phục vụ wire types; `sha2` là runtime dependency của `contracts` cho
canonical config hash và event ID theo contract. `serde_json` và `sha2` vẫn là
dev-dependencies của `wyc-lib`, dùng round-trip/schema/checksum fixtures.
Phiên bản chính xác và dependency bắc cầu nằm trong `Cargo.lock`.
Các dependency trực tiếp hiện chọn có giấy phép MIT OR Apache-2.0;
kiểm kê toàn bộ dependency trước phát hành thuộc G6. Chưa chọn giấy phép phát
hành source của wyc-lib; `publish = false` cho cả bốn package.

## Quyết định nền tảng

- [ADR-0001: thời gian và revision](docs/adr/0001-time-and-revisions.md).
- [ADR-0002: phạm vi, số và nền tảng](docs/adr/0002-workspace-and-wire.md).
- [Contracts](CONTRACTS.md), [API mục tiêu](docs/API.md).
- [Công thức và quy tắc P1](docs/algorithms/P1.md).
- [Nền tảng và cách kiểm tra](docs/DEVELOPMENT.md).

Phần kiểm tra contract nhỏ được thực thi ngay ở P0 để fixture và phản ví dụ có
thể chạy. P1.1 tái sử dụng validation, bổ sung `AvailableData` read-only,
numeric series/status/time checks và canonical/hash/ID tại `contracts` để mọi
crate thuật toán dùng chung. `wyc_lib::prepare_request` chuẩn hóa tập module và
tạo metadata; không trả kết quả phân tích rỗng giả thành công. Không tạo thêm
crate. Wire schema và seed/rule không đổi; dependency lock không đổi phiên bản.
AnalysisState và orchestration toàn P1 được triển khai tại P1.6 bên dưới.

P1.2 thêm crate `indicators` có dependency runtime duy nhất là `contracts`;
không thêm dependency bên ngoài. `indicators::rolling` nhận slice FiniteF64,
trả NumericValue với warm-up đầy đủ. Từng cửa sổ dùng tổng bù Neumaier và
variance hai lượt; overflow trả AnalysisError với code/path. Caller chọn
tiền tố khả dụng trước và gắn NumericSample bằng contracts. Không đưa phép
tính vào bindings hoặc contracts. Bản chuẩn O(N×min(n,N)), không có state,
không tuyên bố tối ưu hoặc batch–incremental. `wyc_lib` re-export crate;
analytical entry points vẫn thuộc P1.6.

P1.3 thêm `indicators::basic`, tái sử dụng mean/tổng bù của P1.2. Module này
không phụ thuộc wyc_lib; `calculate_basic` trả typed BasicIndicatorResult với
metadata và đúng chín series, không giả thành toàn bộ IndicatorResult.
BasicIndicatorState lưu prefix owned, config và result; snapshot tích lũy
được kiểm identity/history/availability trước mỗi update đúng một nến.
Update/reset tính lại batch và thay state atomic; không serialize, không
cam kết O(1). Bộ nhớ O(N), mỗi update O(N×min(n,N)), cộng validation toàn
snapshot O(M) với M gồm future. Không thêm dependency/schema/config version.

P1 bắt đầu theo [ngoại lệ G0 local](docs/acceptance/G0.md).
P1.4 thêm `indicators::advanced`, tái sử dụng helper smoothing có offset,
True Range và rolling mean/std. `calculate_advanced` trả đúng 11 series có
metadata/time; không chạy chỉ báo nền. BasicIndicatorState vẫn chỉ quản lý
chín series P1.3; state/orchestrator toàn P1 thuộc P1.6. Không thêm dependency,
đổi schema/config/hash/seed. Kết quả tại [báo cáo P1.4](docs/acceptance/P1.4.md).
Yêu cầu P1.3 mở lại kiểm chứng CI; kết quả tại [báo cáo P1.3](docs/acceptance/P1.3.md).
Phạm vi và kiểm chứng P1.1 tại [báo cáo](docs/acceptance/P1.1.md).
Phạm vi, kiểm chứng local và giới hạn P1.2 tại [báo cáo](docs/acceptance/P1.2.md).

P1.5 thêm crate `market_structure`, runtime chỉ phụ thuộc contracts, không cần
indicators vì strength dùng range nến hiện tại. `calculate_structure` validate
toàn snapshot/config rồi tính AvailableData; trả StructureResult có kiểu và
wire serialization hiện có. Break dùng reference/trend của nến trước, rồi mới
xác nhận pivot high/low và tính trend. Evidence/ID giữ lineage và thứ tự ổn định.
Không đổi contracts/schema/config/hash; Cargo.lock chỉ thêm package nội bộ và
cạnh dependency. Chi phí quét O(N×min(L+R+1,N)), sort evidence O(N log N), bộ nhớ
O(N), validation toàn snapshot O(M). State chung còn thuộc P1.6.
Kiểm chứng tại [báo cáo P1.5](docs/acceptance/P1.5.md).

P1.6 thêm orchestration/state tại wyc_lib, DTO ClosedBar/ClosedBarUpdate tại
contracts; không thêm package/dependency. `calculate_indicators` ghép 9 basic,
11 advanced và 5 rolling thành 25 series, kiểm shape/time/warm-up; `analyze_structure`
gọi engine đã có. `analyze_batch` chỉ chạy module được chọn theo thứ tự canonical;
hai module P1 không phụ thuộc nhau. AnalysisState giữ prefix owned, config,
modules/identity/metadata và result, mỗi append tính lại batch. Identity/cursor
phải echo chính xác; gap cần reset, revision mới dùng reset/replay. Replacement
chỉ commit sau validation/tính toán thành công. Không serialize state hoặc giữ
future, không có query quá khứ từ state mới. Chi phí O(N×min(n,N)) cho indicators,
O(N×min(L+R+1,N)+N log N) cho structure, bộ nhớ O(N); còn lặp validation và
primitives qua các engine. [ADR-0003](docs/adr/0003-p1-orchestration-state.md)
ghi chữ ký khác draft và protocol; [báo cáo P1.6](docs/acceptance/P1.6.md)
ghi bằng chứng thực tế. P1.7 thêm replay/regression target và hai executable
example/baseline, không đổi dependency graph hoặc Core. Runner local/CI đo
CPU/time/RSS bên ngoài Core; không đưa clock vào quyết định phân tích.
Kết quả toàn G1 tại [báo cáo G1](docs/acceptance/G1.md).
