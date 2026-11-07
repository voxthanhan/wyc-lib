# Fixtures nền P0

Manifest `manifest.json` nhận diện file input/golden bằng SHA-256 của **bytes
nguyên trạng**; không chuẩn hóa line endings trước hash. Không tự cập nhật hash
hoặc expected từ output engine. Sửa file phải review nguyên nhân và cập nhật
manifest/version có chủ đích. Mọi dữ liệu ở đây được tạo tổng hợp bằng tay, không
có dữ liệu thị trường hoặc bí mật; không dùng ngẫu nhiên (seed: không áp dụng).

- `temporal-v1`: bốn nến, đếm thủ công nến thỏa closed + close_time/available_at
  <= as_of; pivot high tại 2000 cần nến 3000 chỉ có lúc 3500. Golden minh họa
  evidence, không phải output từ thuật toán pivot chưa triển khai.
- `p1-arithmetic-v1`: close 1..6, OHLC/volume theo công thức ghi trong đặc tả;
  seed và giá trị tính tay. `scripts/verify_oracles.py` đối chiếu bằng phân số
  chính xác/Decimal, độc lập với Rust Core, không ghi expected. Bộ golden này
  chuẩn bị cho P1, chưa chứng minh P1 đã chạy hay đủ toàn bộ ca biên G1.

Harness Rust kiểm SHA và shape/type/temporal expectations; hàm so sánh báo path
cụ thể khi số, status, field hoặc timestamp khác. Ca âm sửa có chủ đích một
expected/actual để xác nhận harness phát hiện drift và checksum sai.

`tests/regression` dành các case lỗi bổ sung có version; `tests/parity` dành
fixtures xuyên ngôn ngữ từ P2. Hiện chưa có bindings/parity engine.
