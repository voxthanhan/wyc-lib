# wyc-lib

One Analytical Core, Multiple Applications. Evidence Before Prediction.

**P2.1 đang triển khai:** đã chốt contract FFI v1 và thêm shared transport Rust
cùng mã pilot NAPI-RS/PyO3. Kiểm chứng native ABI, npm/wheel và CI chưa hoàn tất
do giới hạn môi trường; chưa nghiệm thu P2.1/G2 hoặc phát hành Foundation.
Xem [báo cáo P2.1](docs/acceptance/P2.1.md),
[contract FFI](docs/adr/0005-p2-ffi-pilot.md) và
[cách kiểm tra](docs/DEVELOPMENT.md). Các đoạn dưới ghi lịch sử G0/G1.

Thư viện lõi phân tích định lượng Rust dùng chung cho các ứng dụng. **Hiện trạng:
G1 incremental** có workspace, typed contracts/validation, view dữ liệu khả dụng,
chuẩn bị request, kiểm tra numeric series/status/time, config hash và event ID
ổn định; crate `indicators` có rolling sum/mean/min/max và population variance/std,
warm-up và lỗi overflow rõ ràng; thêm SMA/EMA/WMA, TR/ATR, ROC/Momentum,
Volume SMA/RVOL, `calculate_basic` và BasicIndicatorState với update/reset
atomic, fixtures/oracle độc lập và test harness. Bản state đầu tính lại batch; bản incremental sau G1 dùng cache cho sample mới.
P1.4 thêm RSI, MACD, DI/DX/ADX và Bollinger Bands cùng `calculate_advanced`
trả 11 series có warm-up/thời gian riêng. P1.5 thêm `market_structure` với
pivot xác nhận, HH/HL/LH/LL/EH/EL, trend, BOS/CHoCH, strength và evidence;
API `calculate_structure` dùng snapshot/config/as_of. P1.6 thêm
`calculate_indicators` (25 series), `analyze_structure`, `analyze_batch` và
`AnalysisState` với update một nến/reset atomic, identity/cursor/revision guards
và batch tính lại trên prefix khả dụng. Node.js/Python bindings thuộc P2,
benchmark/nghiệm thu toàn G1 còn chờ P1.7.
Package Rust `wyc-lib` được dùng với định danh `wyc_lib`.

- [Thiết kế nền tảng](WYC_LIB.md) · [Kế hoạch triển khai](IMPLEMENTATION_PLAN.md)
- [Kiến trúc](ARCHITECTURE.md) · [Contracts](CONTRACTS.md) · [API mục tiêu](docs/API.md)
- [Công thức P1](docs/algorithms/P1.md) · [Chạy kiểm tra](docs/DEVELOPMENT.md)
- [Báo cáo G0](docs/acceptance/G0.md) · [Nguồn gốc fixtures](tests/fixtures/README.md)
- [Phạm vi và kiểm chứng P1.1](docs/acceptance/P1.1.md)
- [Phạm vi và kiểm chứng P1.2](docs/acceptance/P1.2.md)
- [Phạm vi và kiểm chứng P1.3](docs/acceptance/P1.3.md)
- [Phạm vi và kiểm chứng P1.4](docs/acceptance/P1.4.md)
- [Phạm vi và kiểm chứng P1.5](docs/acceptance/P1.5.md)
- [Phạm vi và kiểm chứng P1.6](docs/acceptance/P1.6.md)
- [Nghiệm thu G1/P1.7](docs/acceptance/G1.md) · [Baseline Rust](benchmarks/README.md)
- [Ví dụ Rust chạy được](examples/p1_walkthrough.rs):
  `cargo run --locked --package wyc-lib --example p1_walkthrough`

Ứng dụng cung cấp dữ liệu và availability/revision đã chuẩn hóa. Core không
kết nối nguồn dữ liệu, tự sửa OHLCV hay quyết định chiến lược giao dịch.
Repository hiện public theo lựa chọn của người dùng; định hướng chuyển private
về sau. Các package hiện đặt `publish = false`. Git và kiểm chứng checkout sạch
đã hoàn tất cho P0; P1 bắt đầu theo ngoại lệ local được người dùng phê duyệt.
Yêu cầu P1.3 mở lại kiểm chứng CI Windows/Linux; kết quả đợt đó nằm trong
[báo cáo P1.3](docs/acceptance/P1.3.md): local/checkout sạch Windows đạt 42 tests;
CI Windows/Linux đều đạt tại commit `d86e6ff`, run 42353272952, có hai artifact.
P1.4 đạt local và checkout sạch Windows tại `5caf81b`, 54 tests; CI Windows/Linux
đều đạt tại `4521da9`, [run 42355665535](https://github.com/voxthanhan/wyc-lib/actions/runs/42355665535),
có hai artifact log. Báo cáo P1.4 ở trên ghi phạm vi nghiệm thu và giới hạn
tải payload artifact. Điều kiện CI hoãn của G0 đã đóng ở P1.3; toàn G1
còn mở cho P1.7 sau nghiệm thu P1.6. P1.5 đạt local/checkout sạch Windows tại `6cbc517`,
70 tests; sau khi người dùng cho phép rõ đẩy source P1.5 công khai, CI
Windows/Linux đều đạt tại `c7de17a`, [run 42393366227](https://github.com/voxthanhan/wyc-lib/actions/runs/42393366227),
hai artifact log đã upload. P1.5 hoàn thành phạm vi đã giao; bằng chứng và
giới hạn tải payload artifact nằm trong báo cáo P1.5.
Báo cáo G0 có bổ sung CI; P1.1/P1.2 giữ bằng chứng lịch sử.

P1.6 đạt local/checkout sạch Windows và CI Windows/Linux tại `4460753`,
85 integration tests; [run 42395906710](https://github.com/voxthanhan/wyc-lib/actions/runs/42395906710)
thành công và upload hai artifact. Báo cáo P1.6 ghi 60 source fingerprints,
protocol state, bằng chứng nghiệm thu và tích hợp main. P1.7/baseline/G1 còn mở.

Đã tích hợp P1.6 vào main tại `5af2383`; [CI main 42396385077](https://github.com/voxthanhan/wyc-lib/actions/runs/42396385077)
đạt cả Windows/Linux. Checkout main đã đồng bộ phần mã và báo cáo nghiệm thu.

P1.7 hoàn tất replay/regression, ví dụ Rust và baseline release CPU/time/RSS.
**G1 đạt** tại `7c57220`: local và checkout sạch Windows đạt 88 tests;
[CI 42398202869](https://github.com/voxthanhan/wyc-lib/actions/runs/42398202869)
đạt cả Windows/Linux. Báo cáo G1 ở trên ghi 1008 prefix traces, 65 source
fingerprints, raw baseline và giới hạn artifact. Các kết luận G1 còn mở trong
đoạn P1.3–P1.6 phía trên là lịch sử của từng bước. State vẫn tính lại batch;
bindings P2 và Foundation release chưa triển khai.

Đã tích hợp G1 vào main tại `223d742`;
[CI main 42399120154](https://github.com/voxthanhan/wyc-lib/actions/runs/42399120154)
đạt cả Windows/Linux. Bổ sung báo cáo sau CI chỉ thay tài liệu/log, giữ nguyên
mã đã kiểm chứng.

Đợt [incremental sau G1](docs/acceptance/G1-incremental.md) chuyển append thành
cache/sample/event mới; BasicIndicatorState cũng dùng cache chín series.
Giữ batch chuẩn, golden, contract và atomic errors/reset. Full owned output
vẫn clone O(N). Hai ZIP CI G1 cũ đã tải và kiểm digest cùng nội dung độc lập;
trạng thái kiểm chứng/tích hợp bản mới trong báo cáo bổ sung.

Đã tích hợp incremental vào main tại `507fdad`;
[CI main 42436169902](https://github.com/voxthanhan/wyc-lib/actions/runs/42436169902)
đạt Windows/Linux, mỗi payload có 93 Rust tests và năm verifier controls PASS.
Cả tám ZIP G1 cũ/mới đã kiểm digest/CRC/summary/traces/baseline độc lập.
Báo cáo bổ sung sau CI chỉ thay tài liệu/log; mã đã kiểm chứng giữ nguyên.
