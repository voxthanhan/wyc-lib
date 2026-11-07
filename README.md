# wyc-lib

One Analytical Core, Multiple Applications. Evidence Before Prediction.

Thư viện lõi phân tích định lượng Rust dùng chung cho các ứng dụng. **Hiện trạng:
P1.6** có workspace, typed contracts/validation, view dữ liệu khả dụng,
chuẩn bị request, kiểm tra numeric series/status/time, config hash và event ID
ổn định; crate `indicators` có rolling sum/mean/min/max và population variance/std,
warm-up và lỗi overflow rõ ràng; thêm SMA/EMA/WMA, TR/ATR, ROC/Momentum,
Volume SMA/RVOL, `calculate_basic` và BasicIndicatorState với update/reset
atomic, fixtures/oracle độc lập và test harness. State tính lại batch mỗi update.
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
