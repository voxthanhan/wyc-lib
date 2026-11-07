# wyc-lib — Kiến trúc thực thi

## Hiện trạng P1.4

Workspace có ba package: `contracts` (wire types, cấu hình, kiểm tra contract,
AnalysisError), `indicators` (rolling/statistics P1.2, chín chỉ báo nền và
BasicIndicatorState P1.3, RSI/MACD/ADX/Bollinger P1.4) và `wyc-lib`
(`wyc_lib` trong Rust, re-export contracts/indicators và `prepare_request`). Chưa có orchestrator
phân tích toàn P1, AnalysisState hoặc bindings. Các hàm trong `WYC_LIB.md`
là mục tiêu, không phải API đang chạy. Không tạo crate rỗng cho các phase sau.

```text
Application → bindings (P2) → wyc_lib → các crate thuật toán (P1–P6)
                                  ↘ contracts ↗
P1.1: wyc_lib → contracts → serde / sha2
P1.2: wyc_lib → indicators → contracts
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
availability/revision và replay mọi tiền tố; verifier buộc đủ năm targets.
`tests/support/mod.rs` chứa so sánh JSON/số và replay contract, không phải engine.
`serde` phục vụ wire types; `sha2` là runtime dependency của `contracts` cho
canonical config hash và event ID theo contract. `serde_json` và `sha2` vẫn là
dev-dependencies của `wyc-lib`, dùng round-trip/schema/checksum fixtures.
Phiên bản chính xác và dependency bắc cầu nằm trong `Cargo.lock`.
Các dependency trực tiếp hiện chọn có giấy phép MIT OR Apache-2.0;
kiểm kê toàn bộ dependency trước phát hành thuộc G6. Chưa chọn giấy phép phát
hành source của wyc-lib; `publish = false` cho cả ba package.

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
AnalysisState và orchestration toàn P1 thuộc P1.6, chưa triển khai.

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
