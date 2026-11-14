# ADR-0007 — Python API P1 và adapters P2.3

Ngày: 14/11/2026. Phạm vi P2.3, chưa phải G2/Foundation.

Python thêm `analyze_batch(request)`, `calculate_indicators(data, config, as_of)`,
`analyze_structure(data, config, as_of)` và `AnalysisState(request)` với
`result/identity/update_closed_bar/reset/close`. Giữ JSON pilot P2.1. Các hàm
đơn module gọi public API Rust tương ứng qua shared transport đã có; không
triển khai lại phép tính. Output là dict/list owned, schema TypedDict đầy đủ,
tagged ready/warmup/missing giữ nguyên; có `py.typed` và stub PEP 561.

Input là dict/list/tuple builtin; key là str, scalar là str/bool/int/float/None.
Từ chối subclass, cycle, depth >64 và kiểu không hỗ trợ trước JSON. Không chạy
custom serializer, không ép string/Decimal/datetime thành số. Trường integer
chỉ nhận int (không bool/float); mọi int nằm trong ±(2^53−1) để giữ contract
chung với Node. Timestamp ngoài vùng này là ValueError/invalid_timestamp;
integer khác ngoài vùng an toàn là ValueError/invalid_type. NaN/Infinity là
ValueError/non_finite có host path; None vẫn tới Rust để báo missing_data.
Chuỗi/key không encode được UTF-8 trả TypeError/invalid_type tại host path.
Rust giữ trách nhiệm kiểm schema/enum/shape/OHLC/config/availability.

NumPy là optional: nhận ndarray đúng loại, 1D, timestamp/available_at int64,
OHLCV float64 hoặc int64, is_closed bool. Chấp nhận read-only/strided/endian
vì copy `tolist()`; không giữ view/reference, không zero-copy. Không nhận
masked/object/complex/float32/uint64/subclass/2D arrays hoặc NumPy scalar riêng.
Không flatten, cast dtype, sort, fill missing hay thay đổi đơn vị.

`market_data_from_pandas` và `market_data_from_polars` nhận DataFrame đúng loại,
đúng tám cột timestamp/available_at/OHLCV/is_closed (thứ tự tùy ý). Metadata,
symbol, timeframe do caller truyền tường minh; index Pandas không là timestamp.
Copy từng cột sang list, giữ integer/bool, giữ None và từ chối NaN/Infinity;
không đồng nhất heterogeneous DataFrame thành một ma trận NumPy. Dtypes cột
phải thuộc cùng tập int64/float64/bool trên; nullable Pandas extension dtypes,
datetime/string và cột thừa/trùng/thiếu bị từ chối. Polars null trở thành None;
Rust xác định missing_data khi phân tích. Converter chỉ chuyển đổi host, không
tuyên bố MarketData đã qua validation nghiệp vụ.

Không dependency runtime bắt buộc mới; extras pin NumPy 2.2.6, Pandas 2.2.3,
Polars 1.30.0 cho CPython 3.10.9. mypy 1.15.0 chỉ dùng dev/CI. Native PyO3
0.23.5/Maturin 1.8.3 và Cargo.lock giữ nguyên. CI cài wheel ngoài source,
kiểm plain Python khi chưa cài extras, rồi NumPy/Pandas/Polars và strict types.

Lời gọi đồng bộ, giữ GIL; không nhả GIL hoặc quảng bá concurrent state mutation.
State copy-owned và lỗi update/reset atomic theo Rust; close vĩnh viễn,
idempotent, ưu tiên state_closed trước input error. Context manager đóng handle;
GC smoke chỉ kiểm không crash, không chứng minh leak-free. Full fixture parity,
Next.js, FFI benchmark và Foundation còn ở P2.4–P2.6.

## Kiểm chứng

P2.3 đạt tại e4424af trên CI Windows/Linux: wheel cài ngoài source, 72 plain
và 79 extras native Python assertions/OS, 67 host assertions/OS, mypy strict,
Node/TypeScript/JSON regression PASS; --locked/checkout sạch và 97 source blobs
khớp local. Native local chưa xác minh. Chưa tích hợp main; bàn giao PR.
[Báo cáo P2.3](../acceptance/P2.3.md) lưu SHA/run/checksums và giới hạn bằng chứng.
