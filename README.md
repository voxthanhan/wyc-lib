# wyc-lib

One Analytical Core, Multiple Applications. Evidence Before Prediction.

Thư viện lõi phân tích định lượng Rust dùng chung cho các ứng dụng. **Hiện trạng:
P1.3** có workspace, typed contracts/validation, view dữ liệu khả dụng,
chuẩn bị request, kiểm tra numeric series/status/time, config hash và event ID
ổn định; crate `indicators` có rolling sum/mean/min/max và population variance/std,
warm-up và lỗi overflow rõ ràng; thêm SMA/EMA/WMA, TR/ATR, ROC/Momentum,
Volume SMA/RVOL, `calculate_basic` và BasicIndicatorState với update/reset
atomic, fixtures/oracle độc lập và test harness. State tính lại batch mỗi update.
Chưa có chỉ báo P1.4, structure, API điều phối toàn P1 hoặc Node.js/Python bindings.
Package Rust `wyc-lib` được dùng với định danh `wyc_lib`.

- [Thiết kế nền tảng](WYC_LIB.md) · [Kế hoạch triển khai](IMPLEMENTATION_PLAN.md)
- [Kiến trúc](ARCHITECTURE.md) · [Contracts](CONTRACTS.md) · [API mục tiêu](docs/API.md)
- [Công thức P1](docs/algorithms/P1.md) · [Chạy kiểm tra](docs/DEVELOPMENT.md)
- [Báo cáo G0](docs/acceptance/G0.md) · [Nguồn gốc fixtures](tests/fixtures/README.md)
- [Phạm vi và kiểm chứng P1.1](docs/acceptance/P1.1.md)
- [Phạm vi và kiểm chứng P1.2](docs/acceptance/P1.2.md)
- [Phạm vi và kiểm chứng P1.3](docs/acceptance/P1.3.md)

Ứng dụng cung cấp dữ liệu và availability/revision đã chuẩn hóa. Core không
kết nối nguồn dữ liệu, tự sửa OHLCV hay quyết định chiến lược giao dịch.
Repository hiện public theo lựa chọn của người dùng; định hướng chuyển private
về sau. Các package hiện đặt `publish = false`. Git và kiểm chứng checkout sạch
đã hoàn tất cho P0; P1 bắt đầu theo ngoại lệ local được người dùng phê duyệt.
Yêu cầu P1.3 mở lại kiểm chứng CI Windows/Linux; trạng thái mới nhất nằm trong
[báo cáo P1.3](docs/acceptance/P1.3.md): local/checkout sạch Windows đạt 42 tests;
CI Windows/Linux đều đạt tại commit `d86e6ff`, run 42353272952, có hai artifact.
Điều kiện CI hoãn của G0 đã đóng; toàn G1 còn mở cho P1.4–P1.7.
Báo cáo G0 có bổ sung CI; P1.1/P1.2 giữ bằng chứng lịch sử.
