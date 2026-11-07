# wyc-lib

One Analytical Core, Multiple Applications. Evidence Before Prediction.

Thư viện lõi phân tích định lượng Rust dùng chung cho các ứng dụng. **Hiện trạng:
P0** có workspace, typed contracts/validation, đặc tả P1, fixtures và test harness.
Chưa có thuật toán chỉ báo/cấu trúc, API phân tích hoặc Node.js/Python bindings.
Package Rust `wyc-lib` được dùng với định danh `wyc_lib`.

- [Thiết kế nền tảng](WYC_LIB.md) · [Kế hoạch triển khai](IMPLEMENTATION_PLAN.md)
- [Kiến trúc](ARCHITECTURE.md) · [Contracts](CONTRACTS.md) · [API mục tiêu](docs/API.md)
- [Công thức P1](docs/algorithms/P1.md) · [Chạy kiểm tra](docs/DEVELOPMENT.md)
- [Báo cáo G0](docs/acceptance/G0.md) · [Nguồn gốc fixtures](tests/fixtures/README.md)

Ứng dụng cung cấp dữ liệu và availability/revision đã chuẩn hóa. Core không
kết nối nguồn dữ liệu, tự sửa OHLCV hay quyết định chiến lược giao dịch.
Source dự kiến riêng tư; các package hiện đặt `publish = false`.
