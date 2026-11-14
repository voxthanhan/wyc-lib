# Parity P2.4 — Rust / Node.js / Python

`scripts/p2_parity.py` tạo danh sách action xác định, dùng cùng input/config
cho ba runtime. `p2_parity_reference` gọi trực tiếp public API `wyc_lib` và
AnalysisState, không gọi shared FFI transport. Hai driver `p2_parity.cjs` và
`p2_parity.py` chạy từ consumer ngoài source, chỉ import tarball/wheel đã cài.
Input phải giữ nguyên sau mỗi action; records giữ các kết quả cũ qua update/reset.

Ma trận có 39 ca: arithmetic, temporal, 8 rolling, 6 basic, 8 advanced và 15
structure. Tất cả chạy ba entry points, replay từng tiền tố, reset revision từ
nửa lịch sử rồi append. Structure chạy ba lựa chọn module và hai lịch
availability, gồm nhiều nến cùng khả dụng. So batch/state và full snapshot
tại `as_of` cũ ở mỗi ranh giới availability.
Các fixture primitives được đóng thành MarketData; cột định lượng dùng f64,
timestamp dùng integer. Không đổi giá trị hay file fixture/golden gốc.

Lỗi gồm shape, missing, OHLC, volume, time/availability, config/version,
modules, revision/gaps; update sai identity/cursor/metadata, nến mở/chưa khả dụng
và reset lỗi phải atomic, sau lỗi vẫn append hợp lệ được.
Keys, shape/order, null/mask/status, ID, enum, timestamp, config_hash, cursor,
min/max, giá pivot/break và error code/path phải khớp chính xác. Error class
Node/Python được chuẩn hóa type/range/state/engine; không so error message.
Số thực theo [tolerance P1](../../docs/algorithms/P1.md): variance rtol 1e-11,
đại lượng phần trăm atol 1e-10, còn lại atol/rtol 1e-12.
JSON syntax/host types ngoài DTO chung được kiểm riêng bằng smoke P2.1 và
boundary/type tests P2.2/P2.3, gồm safe integer/NaN/Infinity.

Reference Core chứng minh parity, không là oracle công thức độc lập. CI giữ
oracle/golden P1, checksum manifest và Core replay. Truth table temporal kiểm
riêng 1/2/2/3/3 nến tại 2000/2500/3499/3500/9000; evidence known_at không được
vượt as_of. Năm negative controls kiểm comparator và tính xác định của matrix.

Chạy native suite với toolchain tại [DEVELOPMENT](../../docs/DEVELOPMENT.md):

```text
python scripts/check_p2.py
```

Runner xuất `ci-evidence/p2/parity-report.json`, summary, action SHA-256,
fixture/golden SHA-256, counts và raw runtime records. CI chạy Windows x64 MSVC
và Ubuntu 24.04 x64 GNU. Core-only khi chưa có dependencies native:

```text
python scripts/test_p2_parity.py
cargo build --locked --package wyc-binding-contract --bin p2_parity_reference
python scripts/p2_parity.py --core-reference target/debug/p2_parity_reference
```

Windows dùng đuôi `.exe`. Core-only không chứng minh native ABI. Server/Next.js
thuộc P2.5; FFI baseline thuộc P2.6; G2/Foundation còn chờ các bước này.
