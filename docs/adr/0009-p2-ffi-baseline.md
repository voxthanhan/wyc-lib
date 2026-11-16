# ADR-0009 — Baseline FFI và ứng viên Foundation

Ngày 17/11/2026. Phạm vi P2.6; không đổi public API, thuật toán hay schema.

## Phương pháp

Ba snapshot synthetic triangle-v1 dài 64/256/1024 nến, UTC milliseconds,
closed/available; cùng cấu hình period=14, MACD 12/26/9, pivot 2/2.
Input được tạo từ metadata của ví dụ đã có, lưu nguyên JSON và SHA-256.
Đây là workload hiệu năng, không phải golden kiểm đúng công thức.

Mỗi operation: hai warmups, năm samples, ba iterations/sample tuần tự;
ghi toàn bộ ns/op, median/min/max và bars/second từ median. Không đặt ngưỡng
tốc độ CI và không suy ra cải thiện khi chưa có phép đo trước/sau cùng môi trường.
Mọi binary Rust/native build release, --locked; build/setup/import/đọc input
ngoài timer. Không ép GC, không chạy worker đồng thời. Thứ tự cố định có thể
chịu JIT/cache/GC và nhiễu shared runner; không dùng để so Windows với Linux.

Rust đo direct batch, hai single-module calls, clone request, serde decode,
serialize report và shared JSON transport. Node/Python đo deep-copy proxy,
host snapshot+encode thực của package, decode JSON response, raw native JSON
transport, typed batch, typed hai single-module calls và typed-array/NumPy batch.
Python đo thêm converters Pandas/Polars khi extras đã cài. Node harness nạp
bản index.cjs đã cài và chỉ expose hàm snapshot trong bản module riêng để đo;
không sửa package, không thêm public hook. Copy proxy không đại diện chính xác
mọi allocation/copy trong FFI. Các tầng có scope khác nhau, không cộng/trừ median
để tuyên bố chi phí ABI thuần. Raw native gồm ABI/string copy, Rust decode,
validation, Core và encode; typed gồm host validation/copy/JSON/output parsing.

Batch ở đây là một snapshot với hai module; so với calculate_indicators rồi
analyze_structure trên cùng snapshot. Không phải API batch nhiều mã hoặc gộp
nhiều nến thành một update. Kiểm toàn bộ outputs trước timing bằng direct Rust
reference và comparator P2.4; kiểm input giữ nguyên sau timing. Vẫn chạy đầy đủ
parity/golden/package/server regression độc lập.

CPU/wall/peak RSS đo toàn process qua runner hiện có, gồm setup/warmups/JSON;
không phải retained heap/state hoặc leak/concurrency test. Ghi runtime/toolchain,
CPU/platform, commit/clean, binary/package/input hashes và cấu hình build.

## Phân phối

Ứng viên Foundation giữ package version 0.1.0, schema 0.1.0, config p1-v1 và
binding p2-ffi-v1. Định danh candidate bằng source SHA và checksum từng artifact;
không tạo tag/release, không publish npm/PyPI. Ma trận và giới hạn giữ theo
[ADR-0008](0008-p2-package-consumers.md). G2 chỉ đóng sau CI đúng HEAD và main,
baseline đủ cả OS, artifact kiểm được và tài liệu nghiệm thu cập nhật.
