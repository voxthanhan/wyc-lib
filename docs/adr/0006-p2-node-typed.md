# ADR-0006 — Node.js typed API P2.2

Ngày: 14/11/2026. Phạm vi: Node.js P1, chưa phải G2/Foundation.

Thêm `analyze_batch(request)`, `calculate_indicators(data, config, as_of)`,
`analyze_structure(data, config, as_of)` và `AnalysisState(request)` với
`result()`, `identity()`, `update_closed_bar(update)`, `reset(request)`, `close()`.
Giữ nguyên API JSON pilot và `State` cũ. Single-module gọi đúng hàm Rust,
trả ModuleResult; batch/state trả AnalysisResult. Không thay Core/schema/hash.

JS nhận plain record, array đặc và Float64Array cho cột số. Copy sang JSON DTO
trước native call; kiểm tra descriptor để từ chối getter, symbol, toJSON,
undefined, BigInt, function, cycle, sparse array và class instance. Không ép kiểu.
NaN/Infinity trả RangeError non_finite tại đường dẫn JS trước JSON.stringify;
integer không an toàn/fractional bị từ chối trước khi mất precision. Các kiểm
tra shape/unknown fields/enum và nghiệp vụ tiếp tục do Rust serde/Core xử lý.
Typed arrays khác Float64Array không được nhận; boolean columns dùng array.
Giới hạn depth chuyển đổi là 64; vượt giới hạn trả invalid_type.
JSON pilot giữ nguyên integer-token semantics. JS object không phân biệt 6000
với 6000.0; number integer hợp lệ được encode thành JSON integer.

Output là object mới, không giữ reference input/output của caller. Sync trên
Node thread, không worker/async/zero-copy. State đóng vĩnh viễn và idempotent;
mọi typed operation sau close trả state_closed trước khi kiểm input. Lỗi giữ
class/code/path của ADR-0005; sai host shape là invalid_type, integer host sai
là invalid_type, timestamp ngoài safe integer là invalid_timestamp.

TypeScript mô tả đầy đủ DTO P1 và discriminated NumericValue; readonly input
không ngụ ý freeze output. TypeScript không thay runtime validation. Compiler
pin 5.7.3 là dev-only, không thêm dependency runtime. CI cài tarball ngoài tree,
compile consumer strict và chạy native tests trên Windows/Linux Node 22.22.0.
Node 24.20.0 chỉ được ghi hỗ trợ sau khi native local thực chạy. P2.3–P2.6 còn mở.
