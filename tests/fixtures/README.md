# Fixtures nền P0–P1.5

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
- `p1-identity-v1`: config mặc định và bốn event identities tổng hợp, gồm UTF-8,
  dấu hai chấm, ordinal u64 cực đại và cặp trường có cùng nối chuỗi nếu thiếu
  length-prefix. Golden giữ canonical text, từng preimage và SHA-256; được tính
  lần đầu bằng Python standard-library hashlib, không lấy từ Rust runtime.
  Script oracle đối chiếu lại `struct.pack` cho f64 bits, UTF-8 byte lengths và
  hash, chỉ đọc file. Harness nhận fixture này là identity data, không phải OHLCV.
- `p1-rolling-v1`: tám chuỗi số tính tay: ramp, âm/trùng, n=1, hằng, zero,
  offset lớn, triệt tiêu và số rất nhỏ; golden giữ mọi cửa sổ của cả sáu phép.
  Null trong golden là marker warm-up; test dựng đúng required/available,
  không dùng null làm dữ liệu thiếu hoặc giá trị số của output. Oracle dùng
  Fraction chính xác và sqrt Decimal 80 chữ số, không gọi Rust; số rất nhỏ
  dùng kiểm tra tương đối để sai số tuyệt đối không che kết quả sai bằng 0.
  Input numeric series không phải MarketData; kiểm thời gian dùng lại temporal
  và ramp. Biên period=14/1000000, overflow/underflow, missing/invalid, bất biến,
  revision và tương lai thay đổi được kiểm trong `tests/p1_rolling.rs`.

- `p1-basic-v1`: sáu chuỗi OHLCV tổng hợp: nonlinear/gap giá, âm/zero và hồi
  phục mẫu số, n=1, hằng 0.1 với volume=0, kiểm lag, số rất nhỏ. Golden mới
  được tạo ban đầu bằng Fraction độc lập với Rust; EMA/ATR dùng tổng hình học
  đóng thay vì recurrence f64. `scripts/basic_oracle.py` không ghi file;
  verifier chỉ đọc/so expected và checksum. Null là warm-up, chuỗi
  zero_denominator là missing, số là ready. Test so mask/time chính xác và số
  theo tolerance P1; tiny dùng relative-only. Mọi tiền tố, period=14/1000000,
  overflow/underflow, state identity/history/reset và tính atomic được kiểm
  trong `tests/p1_basic.rs`. Không sửa golden hoặc checksum của bộ trước.

- `p1-advanced-v1`: tám chuỗi tổng hợp cho 11 components RSI/MACD/ADX/Bollinger:
  nonlinear/gap, âm/zero, n=1/signal=1, hằng 0.1, DM tie/inside/outside,
  lookback mặc định 45 nến, tiny 1e-100 và large 1e100. Golden được tạo ban
  đầu bằng oracle Fraction/Decimal độc lập, không lấy output Rust. Smoothing
  dạng tổng hình học, RSI=gain/(gain+loss), variance bằng khoảng cách cặp.
  Null chỉ warm-up, số là ready. Tiny kiểm tương đối; bands large gần triệt
  tiêu giữ golden và kiểm thêm ngân sách sai số làm tròn theo thang input
  được công bố trong catalog P1. Script kiểm chỉ đọc; không đổi bộ golden cũ.
  Prefix/revision/availability và biên overflow được kiểm trong p1_advanced.

- `p1-structure-v1`: 15 chuỗi OHLCV tường minh: đảo chiều và chuỗi phản chiếu,
  equal/dual pivot, plateau, L/R bất đối xứng, cửa sổ mặc định, gap/range=0,
  strength>1, unknown, hằng/rỗng/một nến, tiny/large. Golden tạo trước khi kiểm
  Rust bằng oracle liệt kê cửa sổ và tập pivot tại mỗi tiền tố, Fraction strength,
  hashlib config/event ID. Mọi trường/ID/time/evidence được đối chiếu, có các
  anchor tính tay độc lập trong p1_structure. Các ca conflict/overflow/underflow,
  revision/delayed availability và future mutation bổ sung trực tiếp trong tests.
  Script kiểm chỉ đọc; fixture/golden/checksum các bộ trước giữ nguyên.

Harness Rust kiểm SHA và shape/type/temporal expectations; hàm so sánh báo path
cụ thể khi số, status, field hoặc timestamp khác. Ca âm sửa có chủ đích một
expected/actual để xác nhận harness phát hiện drift và checksum sai.

`tests/regression` dành các case lỗi bổ sung có version; `tests/parity` dành
fixtures xuyên ngôn ngữ từ P2. Hiện chưa có bindings/parity engine.
