# wyc-lib — Kiến trúc thực thi

## Hiện trạng P0

Workspace có hai package: `contracts` (wire types, cấu hình, kiểm tra contract) và
`wyc-lib` (`wyc_lib` trong Rust, hiện re-export contracts). Chưa có thuật toán,
orchestrator phân tích, state engine hoặc bindings. Các hàm trong `WYC_LIB.md`
là mục tiêu, không phải API đang chạy. Không tạo crate rỗng cho các phase sau.

```text
Application → bindings (P2) → wyc_lib → các crate thuật toán (P1–P6)
                                  ↘ contracts ↗
P0: wyc_lib → contracts → serde
```

`contracts` không phụ thuộc ngược vào `wyc_lib`. Các thuật toán tương lai dùng
contracts; orchestrator chuyển typed results theo dependency graph của kế hoạch.
Wyckoff không phụ thuộc Feature Engine; Evidence Engine không gọi ngược Wyckoff.
Ứng dụng cung cấp dữ liệu, lịch giao dịch, snapshot, authentication và lưu trữ.
Core không gọi mạng, lấy giờ hệ thống, đọc secrets, tự điều chỉnh hoặc sửa dữ liệu.

## Test và dependency

`crates/wyc_lib/Cargo.toml` khai báo test target `foundation` trỏ tới
`tests/foundation.rs`, chạy thực sự trong `cargo test --workspace`.
`tests/support/mod.rs` chứa so sánh JSON/số và replay contract, không phải engine.
`serde` phục vụ wire types; `serde_json` và `sha2` chỉ là dev-dependencies của
`wyc-lib`, dùng round-trip, so sánh schema và checksum fixtures.
Phiên bản chính xác và dependency bắc cầu nằm trong `Cargo.lock`.
Các dependency trực tiếp hiện chọn có giấy phép MIT OR Apache-2.0;
kiểm kê toàn bộ dependency trước phát hành thuộc G6. Chưa chọn giấy phép phát
hành source của wyc-lib; `publish = false` cho cả hai package.

## Quyết định nền tảng

- [ADR-0001: thời gian và revision](docs/adr/0001-time-and-revisions.md).
- [ADR-0002: phạm vi, số và nền tảng](docs/adr/0002-workspace-and-wire.md).
- [Contracts](CONTRACTS.md), [API mục tiêu](docs/API.md).
- [Công thức và quy tắc P1](docs/algorithms/P1.md).
- [Nền tảng và cách kiểm tra](docs/DEVELOPMENT.md).

Phần kiểm tra contract nhỏ được thực thi ngay ở P0 để fixture và phản ví dụ có
thể chạy. P1.1 tái sử dụng phần này và bổ sung result selection, config hash,
stable event ID cùng orchestration; P0 chưa triển khai các thành phần đó.
