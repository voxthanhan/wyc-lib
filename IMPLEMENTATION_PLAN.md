# wyc-lib V1.0 — Kế hoạch triển khai

## 1. Mục đích, hiện trạng và phạm vi

Kế hoạch này cụ thể hóa `WYC_LIB.md`, tuân thủ `AGENTS.md`, giữ các giai đoạn P0–P6 và ba mốc Foundation, Analysis, V1.0 Stable. Mỗi giai đoạn có điều kiện đầu vào, công việc theo dependency, sản phẩm và cổng nghiệm thu riêng.

Tên thư viện/repository và package phân phối là `wyc-lib`; định danh Rust/Python là `wyc_lib`, crate điều phối đặt tại `crates/wyc_lib/`. Khởi tạo manifest và bindings theo quy ước tên trong `WYC_LIB.md`.

**Hiện trạng khi lập kế hoạch:** dự án có tài liệu thiết kế và nguyên tắc làm việc; chưa có Cargo Workspace, mã thuật toán, bindings, fixtures hay CI. Mọi đường dẫn sản phẩm, test suite và báo cáo dưới đây là hạng mục sẽ tạo, không phải kết quả đã hoàn thành. Tất cả cổng nghiệm thu ban đầu ở trạng thái chưa đạt.

wyc-lib chịu trách nhiệm tính toán, quản lý trạng thái thuật toán, xuất evidence/features và cung cấp API dùng chung. **Nguồn dữ liệu, thư viện truy xuất, xác thực, làm sạch, điều chỉnh giá, lịch giao dịch và lưu trữ do từng ứng dụng tự quyết định.** Core chỉ kiểm tra và xử lý dữ liệu theo contract; kế hoạch không chỉ định nguồn dữ liệu cho bất kỳ hệ thống nào.

Ngoài phạm vi V1.0: giao diện, chiến lược giao dịch, quản trị danh mục, mô phỏng khớp lệnh, huấn luyện ML, dự báo lợi nhuận và WASM. Internal/External Structure chỉ triển khai khi có định nghĩa và trường hợp kiểm thử riêng; các hướng mở rộng sau V1.0 không phải điều kiện chặn bản đầu.

## 2. Nguyên tắc tổ chức triển khai

1. **Đặc tả → fixture độc lập → batch đúng → replay/incremental đúng → tích hợp → đo hiệu năng.** Áp dụng theo từng thuật toán; không chờ làm xong cả module mới kiểm thử.
2. Nghiệm thu phạm vi đã triển khai theo từng bước nhỏ. Không xuất API giả, kết quả rỗng giả thành công hoặc bộ nhận diện chưa có định nghĩa kiểm chứng được.
3. Contract của phần sắp triển khai phải được chốt trước khi viết thuật toán/binding. Contract tương lai có thể ở trạng thái dự thảo; mọi thay đổi sau khi phát hành phải được quản lý phiên bản.
4. CI bắt đầu ở P0, baseline bắt đầu ở P1/P2, incremental bắt đầu ở P1. P6 hoàn thiện và tối ưu các năng lực này thay vì khởi động chúng muộn.
5. Bằng chứng dạng dữ liệu và thời gian khả dụng được định nghĩa tại P0, được các module phát sinh từ P1. Bộ tổng hợp Evidence/DecisionReport hoàn chỉnh triển khai ở P5.
6. Các đại lượng dùng bởi Wyckoff đến từ indicators/structure/zones/volume. Feature Engine ở P6 chỉ chuẩn hóa việc xuất đặc trưng; Wyckoff không phụ thuộc vào Feature Engine.
7. Tính đúng của phần mềm, tính phù hợp của quy tắc phân tích và hiệu quả đầu tư là ba việc khác nhau. Kiểm thử phần mềm không chứng minh lợi nhuận hoặc xác suất thành công của một thiết lập.

## 3. Dependency và đường triển khai

### 3.1. Dependency giữa các giai đoạn

| Giai đoạn | Điều kiện bắt đầu | Sản phẩm chính | Cổng |
| --- | --- | --- | --- |
| P0 — Nền móng | Thiết kế và nguyên tắc dự án | Workspace, contracts, đặc tả API, test harness, fixtures nền, CI cơ bản | G0 |
| P1 — Rust nền tảng | G0; ngoại lệ local ngày 07/11/2026 tại mục 12 | Indicators, Market Structure, orchestration và state nền | G1 |
| P2 — Đa ngôn ngữ | G1; contract FFI đã chốt | Node.js/Python packages, parity, kiểm tra cài đặt | G2 / Foundation |
| P3 — Giá, khối lượng, RS | G2; đặc tả từng module | Price Zones, Volume/VSA/Profile, Relative Strength và bindings tương ứng | G3 |
| P4 — Wyckoff và nhiều timeframe | G3; bảng quy tắc Wyckoff và contract nhiều timeframe | Wyckoff Engine, replay nhiều timeframe, bindings | G4 / Analysis |
| P5 — Tổng hợp bằng chứng | G4; dữ liệu evidence từ các module | Evidence Engine, DecisionReport hoàn chỉnh | G5 |
| P6 — Features và hoàn thiện phát hành | G5 | Feature Engine, benchmark/tối ưu có căn cứ, phân phối V1.0 | G6 / V1.0 Stable |

Đường phát hành chính: `G0 → G1 → G2 → G3 → G4 → G5 → G6`. Có thể chuẩn bị đặc tả hoặc khảo sát công cụ cho giai đoạn sau, nhưng không tuyên bố nghiệm thu trước khi dependency đã đạt.

Trong P2, hai bindings có thể phát triển độc lập sau khi adapter contract ổn định. Trong P3, ba module có thể phát triển độc lập trên P1 và contracts đã chốt. P4 chờ các đầu vào structure/zones/volume cần thiết được nghiệm thu. Việc tổ chức các nhánh công việc không làm thay đổi thứ tự tích hợp và release.

### 3.2. Dependency tính toán giữa các module

Bảng sau mô tả dữ liệu cần dùng, không bắt buộc mọi dependency dữ liệu trở thành dependency crate trực tiếp.

| Module | Đầu vào/phụ thuộc tính toán |
| --- | --- |
| `contracts` | Không phụ thuộc crate thuật toán hoặc bindings |
| `indicators` | MarketData và cấu hình từ `contracts` |
| `market_structure` | Giá đã kiểm tra; indicators nếu quy tắc cần chuẩn hóa biến động |
| `price_zones` | Pivot/cấu trúc đã khả dụng; ATR và cấu hình vùng |
| `volume_engine` | OHLCV; rolling statistics và indicators khối lượng |
| `relative_strength` | Chuỗi giá, benchmark, tập mã cùng thời điểm do bên gọi cung cấp; rolling statistics |
| `wyckoff_engine` | Quan sát giá, cấu trúc, vùng và khối lượng đã khả dụng; state của chính nó |
| `evidence_engine` | Facts, context và giả thuyết có kiểu từ các module; bộ quy tắc tổng hợp |
| `feature_engine` | Kết quả module đã có cùng metadata về thời gian khả dụng; chỉ dùng DecisionReport khi feature yêu cầu |
| `wyc_lib` | Điều phối các module đã triển khai theo dependency graph |
| Bindings | Public API của `wyc_lib`, không chứa thuật toán phân tích |

Kiểu trao đổi chung đặt trong `contracts`; orchestration truyền kết quả giữa các module. Không cho crate thuật toán phụ thuộc ngược vào `wyc_lib`, binding hoặc lớp ứng dụng. Evidence Engine không gọi ngược Wyckoff để bổ sung dữ liệu trong lúc đánh giá; Feature Engine không trở thành nguồn đầu vào bắt buộc của các engine phân tích.

### 3.3. Phân bổ public API theo giai đoạn

Tên dưới đây là API mục tiêu từ thiết kế; chữ ký, kiểu kết quả và semantics state được đặc tả trong P0.4 trước khi triển khai. Chỉ công bố API đã có implementation và test, không xuất trước toàn bộ danh sách bằng các hàm giả.

| API mục tiêu | Giai đoạn Rust | Bindings và nghiệm thu |
| --- | --- | --- |
| `calculate_indicators()` | P1 | G1 cho Rust; G2 cho Node.js/Python |
| `analyze_structure()` | P1 | G1 cho Rust; G2 cho Node.js/Python |
| `detect_price_zones()` | P3 | G3 cho cả ba ngôn ngữ |
| `analyze_volume()` | P3 | G3 cho cả ba ngôn ngữ, gồm VSA/Profile đã công bố |
| `analyze_relative_strength()` | P3 | G3 cho cả ba ngôn ngữ |
| `analyze_wyckoff()` | P4 | G4 cho cả ba ngôn ngữ |
| `evaluate_evidence()` | P5 | G5 cho cả ba ngôn ngữ |
| `extract_features()` | P6 | G6 cho cả ba ngôn ngữ |
| `analyze_batch()` | P1, mở rộng P3–P6 | G2 cho phạm vi Foundation; mỗi module mới được tích hợp tại cổng của nó |
| `update_closed_bar()` | P1, mở rộng P3–P6 | G2 cho state Foundation; mỗi module mới phải kiểm tra batch–incremental và parity |

## 4. Các quyết định nền tảng phải chốt trong P0

| Chủ đề | Quyết định cần ghi trong `CONTRACTS.md`/ADR | Lý do và cách kiểm chứng |
| --- | --- | --- |
| Thời gian nến | Đơn vị UTC, timestamp đầu/cuối nến, thời điểm đóng và thời điểm khả dụng | `is_closed` tại hiện tại không đủ chứng minh nến đã khả dụng tại `as_of` quá khứ |
| Khả dụng và revision | Cách biểu diễn `available_at`, `data_revision` hoặc metadata tương đương; snapshot do bên gọi cung cấp | Phân biệt phân tích dữ liệu đã biết lúc đó với phân tích lại lịch sử đã sửa |
| Dữ liệu lỗi/thiếu | Mảng lệch, trùng timestamp, OHLC sai, số không hữu hạn; missing và warm-up có trạng thái riêng | Không tự sửa hoặc biến thiếu dữ liệu thành kết quả hợp lệ |
| Khoảng thiếu | Cách ứng dụng khai báo khoảng dữ liệu bị thiếu và chính sách từ chối/reset/xử lý theo cấu hình | Core không tự suy ra lịch giao dịch từ chênh lệch timestamp |
| Công thức | Seed/smoothing, lookback, mẫu số thống kê, quy tắc đồng hạng, breakout và confirmation | Cùng tên chỉ báo/sự kiện có thể có nhiều biến thể hợp lệ |
| Kết quả số | `f64`, missing mask/status, quy tắc JSON, tolerance theo từng đại lượng | Ngăn khác biệt ngôn ngữ và sai lệch bị che bằng tolerance quá rộng |
| State | State gắn với symbol, timeframe, config, engine, data revision và mốc đã xử lý; semantics update/reset/truy vấn quá khứ | Không dùng lại state cho dữ liệu/cấu hình khác hoặc dùng state tương lai để trả lời quá khứ |
| Phiên bản và ID | Schema/config/engine/feature version; canonical config hash và ID ổn định | Có thể tái lập và truy vết kết quả |
| Nền tảng hỗ trợ | Rust toolchain/MSRV, Node/Python, OS/CPU/ABI và môi trường server dự kiến | Chỉ công bố hỗ trợ khi có build và kiểm tra chạy thực tế |

Tên trường bổ sung ở bảng trên là yêu cầu cần đặc tả tại P0, không phải khẳng định schema gốc đã có. Chốt phương án đủ đơn giản cho bản đầu, ghi quyết định và ví dụ cụ thể; tránh xây một hệ thống quản lý dữ liệu lịch sử bên trong Core.

Mặc định an toàn để bắt đầu: dữ liệu đầu vào bất hợp lệ bị từ chối, nến chưa khả dụng không được đưa vào tính toán, cập nhật trùng/đảo thứ tự bị từ chối, lịch sử sửa đổi cần reset và replay. Chỉ dùng phương án khác khi đã đặc tả và kiểm thử trước khi bật.

Với mọi kết quả dẫn xuất, `known_at` không được sớm hơn thời điểm khả dụng muộn nhất của các đầu vào và điều kiện xác nhận mà kết quả đó cần. Truy vấn `as_of` cũ phải dùng snapshot hợp lệ hoặc replay đến mốc đó; bản đầu có thể replay, không bắt buộc xây hệ thống lưu snapshot, nhưng tuyệt đối không dùng state đã chứa thông tin muộn hơn mốc truy vấn.

## 5. Triển khai từng giai đoạn

### P0 — Contracts, kiến trúc và nền tảng kiểm thử

**Mục tiêu:** loại bỏ các quyết định mơ hồ trước khi chúng lan vào nhiều engine và bindings.

**Đầu vào:** `WYC_LIB.md`, `AGENTS.md` và yêu cầu ranh giới dữ liệu của ứng dụng.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P0.1 | Kiểm kê môi trường; chọn tập nền tảng phát triển tối thiểu và ma trận hỗ trợ mục tiêu; ghi phiên bản toolchain/dependency được chọn khi triển khai | Không |
| P0.2 | Tạo Cargo Workspace tối thiểu với `contracts`, `wyc_lib` và test harness; tạo `ARCHITECTURE.md`, quy tắc dependency và các ADR cần thiết | P0.1 |
| P0.3 | Viết `CONTRACTS.md` và kiểu Rust cho MarketData, AnalysisRequest, result/status/error, Evidence và khung DecisionReport; chốt các quyết định ở mục 4 | P0.2 |
| P0.4 | Đặc tả API mục tiêu, ownership, `as_of`, batch/update/reset, lựa chọn module và lỗi thiếu dependency; phân biệt API sẽ có theo từng phase | P0.3 |
| P0.5 | Tạo danh mục đặc tả thuật toán và quy tắc: công thức, tham số, miền hợp lệ, warm-up, thời điểm khả dụng, trạng thái, ví dụ và tolerance; hoàn tất phần cần cho P1 | P0.3 |
| P0.6 | Tạo fixtures tổng hợp và golden nền đã tính tay hoặc kiểm chứng bằng cách độc lập; dựng replay, so sánh số, kiểm tra schema và báo cáo khác biệt | P0.3, P0.5 |
| P0.7 | Thiết lập CI format/lint/build/contract tests; hướng dẫn chạy cục bộ và mẫu báo cáo nghiệm thu gắn với commit | P0.2, P0.6 |

`tests/fixtures`, `tests/golden`, `tests/regression`, `tests/parity` là nơi lưu tài nguyên dùng chung. Phải có test target/harness thuộc một package trong workspace để thực sự chạy chúng; không chỉ tạo thư mục ở gốc virtual workspace rồi giả định Cargo tự phát hiện test.

**Nghiệm thu G0:**

- [x] Workspace build và các test contract hiện có chạy được từ checkout sạch trên nền tảng phát triển đã chọn.
- [x] Contract có ví dụ hợp lệ và phản ví dụ cho từng nhóm lỗi; có phân biệt missing, warm-up và invalid input.
- [x] Có ví dụ `as_of`, nến đóng nhưng chưa khả dụng, pivot cần xác nhận muộn và lịch sử được sửa; giải thích kết quả mong đợi.
- [x] Đặc tả P1 có công thức/seed/warm-up/tolerance cụ thể; không còn quyết định làm thay đổi kết quả bị để trống trước khi bắt đầu P1.
- [x] Fixtures nền có mã định danh, metadata, checksum, expected result và cách xác minh; harness phát hiện được sai lệch cố ý trong một ca kiểm tra âm.
- [x] Dependency không có vòng; CI cơ bản đã chạy; ma trận hỗ trợ ghi rõ mục tiêu chưa kiểm chứng. Bổ sung tại P1.3: run 42353272952, commit d86e6ff đạt cả Windows/Linux; xem báo cáo G0/P1.3.

**Bằng chứng cần lưu:** contracts/ADR, log CI của commit nghiệm thu, manifest fixtures và báo cáo G0 trong `docs/acceptance/`.

### P1 — Indicators và Market Structure trong Rust

**Mục tiêu:** tạo đường xử lý hoàn chỉnh đầu tiên từ input hợp lệ đến kết quả có thời gian khả dụng, chạy được batch và cập nhật theo nến đóng.

**Điều kiện đầu vào:** G0 đạt hoặc ngoại lệ local được phê duyệt tại mục 12;
công thức và quy tắc cho từng bước đã được đặc tả.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P1.1 | Bộ kiểm tra input/as_of, chọn dữ liệu khả dụng, result mask/status, canonical config hash và event ID ổn định | G0 |
| P1.2 | Phép toán rolling/statistics: sum/mean, min/max, variance/std; kiểm soát ổn định số và cửa sổ | P1.1 |
| P1.3 | SMA/EMA/WMA; TR/ATR; ROC/Momentum; Volume SMA/RVOL; batch và state cập nhật tương ứng | P1.2 |
| P1.4 | RSI, MACD, ADX, Bollinger Bands dựa trên primitives đã nghiệm thu; quy định warm-up của từng thành phần | P1.3 |
| P1.5 | Swing High/Low, HH/HL/LH/LL, trend; sau đó BOS/CHoCH, strength và trạng thái xác nhận theo quy tắc | P1.1, các indicators thực sự dùng từ P1.3–P1.4 |
| P1.6 | `calculate_indicators()`, `analyze_structure()`, `analyze_batch()` và state/update/reset trong `wyc_lib`; chỉ thực thi module được yêu cầu và dependency cần thiết | P1.4, P1.5 |
| P1.7 | Replay từng nến, batch–incremental, revision/reset, golden/regression và benchmark baseline Rust; tài liệu API và ví dụ Rust | P1.6 |

Quy tắc cấu trúc phải định nghĩa cách xử lý equal highs/lows, nến đồng thời chạm hai điều kiện, gap, phá vỡ theo wick/close và precedence khi nhiều sự kiện cùng xuất hiện. Các tham số này là cấu hình có phiên bản, không được quyết định tùy trường hợp trong mã.

State bản đầu ưu tiên reset và replay toàn bộ khi lịch sử sửa đổi. Thuật toán chưa có cập nhật tối ưu có thể dùng batch tính lại nếu được khai báo rõ độ phức tạp; `update_closed_bar()` vẫn phải đúng và không được quảng bá là O(1) khi chưa đạt.

**Nghiệm thu G1:**

- [x] Tất cả chỉ báo liệt kê trong P1.3–P1.4 có đặc tả và tests; kiểm tra trước/tại/sau vị trí hợp lệ đầu tiên, bao gồm cấu hình lookback khác mặc định.
- [x] Chuỗi rỗng/ngắn/hằng, volume bằng 0, mẫu số bằng 0, giá trị lớn/nhỏ và input lỗi có kết quả hoặc lỗi đúng contract; input không bị sửa.
- [x] Pivot có `r` nến xác nhận bên phải chỉ khả dụng sau khi đủ `r` nến đã đóng và khả dụng. Test không cho phép xuất tín hiệu xác nhận tại vị trí pivot trước thời điểm đó.
- [x] BOS/CHoCH và trend không dùng pivot chưa xác nhận; các trường hợp đồng hạng và xung đột có kết quả ổn định.
- [x] Replay và batch tương đương ở mọi tiền tố được kiểm thử; nối dữ liệu tương lai không thay đổi đánh giá tại `as_of` cũ với cùng revision.
- [x] Reset/replay sau sửa lịch sử cho kết quả như batch trên revision mới; update sai symbol/config/thứ tự bị xử lý đúng contract.
- [x] Ví dụ Rust chạy từ input đến output; CI đạt; có baseline CPU/thời gian/bộ nhớ phù hợp khả năng đo hiện tại, ghi rõ số liệu chưa đo được.

**Bằng chứng cần lưu:** golden đã đối chiếu công thức, replay traces, regression report, baseline Rust và báo cáo G1.

### P2 — Node.js và Python Bindings; Foundation Release

**P2.3 đang kiểm chứng (14/11/2026):** Python API P1, AnalysisState, NumPy và
converters Pandas/Polars đã triển khai theo [ADR-0007](docs/adr/0007-p2-python-typed.md).
CI thêm plain/extras/type checks trên wheel cài ngoài source Windows/Linux.
Kết quả thực chạy và giới hạn tại [báo cáo P2.3](docs/acceptance/P2.3.md).
P2.4–P2.6/G2/Foundation còn mở.

**Mục tiêu:** cùng một thuật toán Rust có thể dùng ổn định từ hai ngôn ngữ mà không thay đổi semantics dữ liệu hoặc thời gian.

**Điều kiện đầu vào:** G1 đạt; contract FFI và ma trận hỗ trợ của Foundation đã chốt, có phiên bản chính xác trước khi build package.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P2.1 | Chốt mapping số/timestamp/missing/error/config/result, state handle và ownership; làm thử đường gọi nhỏ để phát hiện lỗi ABI/đóng gói trước khi mở rộng | G1 |
| P2.2 | NAPI-RS adapter và khai báo kiểu TypeScript cho API P1; kiểm tra lifetime, input shape và lỗi | P2.1 |
| P2.3 | PyO3 + Maturin adapter cho API P1; đầu vào mảng/NumPy và đầu ra có schema rõ; adapter Pandas/Polars chuyển đổi đơn thuần | P2.1 |
| P2.4 | Bộ parity Rust–Node.js–Python dùng cùng fixtures và cấu hình, bao gồm batch/update/reset và lỗi | P2.2, P2.3 |
| P2.5 | Build npm package/wheel; cài vào môi trường sạch ngoài source tree; chạy examples và smoke test Node.js server/Next.js server trên target công bố | P2.4 |
| P2.6 | Đo chi phí FFI/chuyển đổi/copy và batch so với nhiều lần gọi; tài liệu tích hợp, compatibility và bản ứng viên Foundation | P2.5 |

**Tiến độ P2.1 ngày 14/11/2026:** mapping/state/ownership đã chốt tại
[ADR-0005](docs/adr/0005-p2-ffi-pilot.md). **P2.1 đạt phạm vi pilot** tại
`cf6c464`, đã tích hợp main: local 101 Rust tests + 5 Python controls; CI Core
43578612827 và native 43578612862 đều PASS Windows/Linux. Native locks khớp;
--locked, checkout sạch, 40 actions/runtime/OS, 14 Rust references và 20 ca lỗi.
Native local chưa xác minh do mạng/Git shell bị hạn chế. Chi tiết tại
[báo cáo P2.1](docs/acceptance/P2.1.md).

**P2.2 đạt phạm vi Node P1** tại `5946286`: ba object APIs và AnalysisState,
TypeScript DTOs, array/Float64Array, input shape/error/lifetime checks.
Local 102 Rust tests + 5 controls + 24 host assertions PASS; CI Core 43580931140
và native 43580931063 đạt Windows/Linux, typed Node 83 assertions/OS,
TypeScript strict, JSON Node/Python smoke và locked/clean acceptance.
Đã tích hợp main cùng commit, CI main Core/native đều PASS Windows/Linux. Native local chưa xác minh;
P2.3–P2.6/G2/Foundation còn mở. [Báo cáo P2.2](docs/acceptance/P2.2.md).

Khởi đầu với đường chuyển đổi có copy và ownership rõ ràng. Adapter Pandas/Polars tái sử dụng đường NumPy/mảng khi phù hợp; không viết lại indicators bằng các thư viện đó. Quy định rõ vùng timestamp an toàn cho từng biểu diễn Node.js và từ chối giá trị không thể biểu diễn chính xác theo contract.

Nếu cần chạy phép tính dài ngoài luồng chính Node.js hoặc nhả GIL Python, phải đặc tả quyền sở hữu dữ liệu và state trước. Không cho hai lời gọi đồng thời sửa cùng state khi chưa có cơ chế bảo vệ được kiểm thử. Không mặc định native addon chạy trong Edge Runtime/browser.

**Nghiệm thu G2 — Foundation:**

- [ ] Hai binding chỉ gọi Rust; mọi API P1 được công bố đều có mapping và ví dụ chạy được.
- [ ] Parity đạt cho số, mask, shape, status, error code, ID và timestamp; các giá trị rời rạc phải khớp chính xác, số thực theo tolerance đã chốt.
- [ ] Package cài và chạy được từ artifact trên toàn bộ ma trận Foundation công bố; mục tiêu chưa có kết quả chạy không được ghi là đã hỗ trợ.
- [ ] Kiểm tra input không hợp lệ, vùng nhớ/lifetime và state handle bị dùng sai cho lỗi kiểm soát được; không crash tiến trình trong các ca kiểm thử này.
- [ ] Node.js server/Next.js server và các adapter Python công bố đã có smoke tests; không cần ứng dụng thị trường hoàn chỉnh để nghiệm thu.
- [ ] CI chạy parity và kiểm tra package; baseline FFI có số liệu; README ghi đúng tính năng đang có, giới hạn và hướng dẫn cài đặt.

**Bằng chứng cần lưu:** parity report, artifact manifest/checksum, log cài sạch trên từng target, baseline FFI và báo cáo G2. Foundation là phạm vi phát hành sau P2, không đồng nghĩa toàn bộ wyc-lib V1.0 đã hoàn thành.

### P3 — Price Zones, Volume/VSA/Profile và Relative Strength

**Mục tiêu:** cung cấp ba nhóm kết quả độc lập, có vòng đời và ngữ nghĩa định lượng đủ rõ để trở thành đầu vào của Wyckoff.

**Điều kiện đầu vào:** G2 đạt. Trước mỗi module phải chốt đặc tả công thức, cấu hình, thời điểm khả dụng và fixtures của module đó.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P3.1 | Đặc tả và fixtures cho zones, volume và RS; mở rộng contracts/result types theo phiên bản | G2 |
| P3.2 | Zones: clustering từ pivot đã biết, độ rộng thích nghi ATR, strength, test count, breakout/retest và lifecycle | P3.1; structure/ATR của P1 |
| P3.3 | Volume/VSA: spread, close location, progress, effort/result, RVOL, expansion/contraction/dry-up; các candidate absorption/no supply/no demand/stopping/climax | P3.1; rolling/volume indicators của P1 |
| P3.4 | Volume Profile xấp xỉ: bins, phân bổ volume, POC/VAH/VAL/LVN, thuật toán Value Area và xử lý đồng hạng | P3.1; primitives dùng lại từ P3.3 |
| P3.5 | RS: Price Relative, Mansfield RS, momentum/slope, relative performance đa kỳ và ranking theo tập mã do bên gọi cung cấp | P3.1; rolling indicators của P1 |
| P3.6 | Tích hợp API, state/update/reset, Node.js/Python bindings, parity và baseline cho cả ba nhóm | P3.2–P3.5 |

Đặc tả zones phải xác định lúc tạo/kích hoạt, cách đếm một lần test, cập nhật biên/strength và hành vi sau break/invalidation, với các trạng thái `candidate`, `active`, `tested`, `broken`, `invalidated` theo thiết kế. Việc gộp vùng về sau tạo revision/sự kiện ở thời điểm hiện tại; không sửa lại lịch sử như thể vùng mới đã tồn tại từ trước.

RS phải đặc tả chính sách ghép mốc thời gian, benchmark bằng 0/thiếu, số thành viên hợp lệ tối thiểu, xử lý đồng hạng và cách ghi tập mã được đánh giá. Không tự kéo dữ liệu, dùng giá tương lai để lấp thiếu hoặc thay tập mã lịch sử bằng tập mã hiện tại. Chỉ tính một snapshot ranking khi các đầu vào cần thiết của snapshot đã khả dụng.

**Nghiệm thu G3:**

- [ ] Zones có tests cho toàn bộ chuyển trạng thái hợp lệ, chuyển trạng thái bị cấm, test lặp, break/retest và invalidation; `known_at` đúng khi biên vùng thay đổi.
- [ ] VSA xuất observation, interpretation/candidate và điều kiện confirmation tách biệt; input không đủ bối cảnh không tạo kết luận chắc chắn.
- [ ] Profile ghi phương pháp xấp xỉ; tổng volume đã phân bổ bảo toàn trong tolerance, bao gồm nến không có spread, volume bằng 0 và giá ở biên bin.
- [ ] POC/VAH/VAL/LVN đạt golden theo định nghĩa đã chọn; đồng hạng và profile rỗng không cho kết quả tùy thứ tự duyệt.
- [ ] RS đạt fixture công thức; ranking ổn định khi đổi thứ tự input, xử lý thiếu/đồng hạng đúng contract và loại dữ liệu chưa khả dụng tại snapshot.
- [ ] Mỗi module đạt golden, prefix/replay, batch–incremental, revision/reset và parity trên API đã công bố; có ca âm chống tín hiệu sai.
- [ ] API/docs/bindings được cập nhật đồng thời; CI đạt, có baseline cho workload đại diện và báo cáo phạm vi kiểm định quy tắc theo mục 7.4.

**Bằng chứng cần lưu:** lifecycle traces của zones, đối chiếu bảo toàn volume, RS snapshot fixtures và báo cáo G3. RS kiểu IBD chỉ được bổ sung dưới định danh/công thức riêng khi đã có đặc tả; không phải lý do để trì hoãn bộ RS cơ bản.

### P4 — Wyckoff State Machine và Multi-Timeframe; Analysis Release

**Mục tiêu:** nhận diện diễn biến có điều kiện từ các quan sát đã biết, thay vì gán nhãn toàn bộ một đoạn giá sau khi biết kết quả cuối cùng.

**Điều kiện đầu vào:** G3 đạt; structure/zones/volume trả được dữ liệu và evidence có thời điểm khả dụng. Không cần chờ bộ tổng hợp Evidence Engine của P5.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P4.1 | Viết bảng quy tắc Trading Range, từng event và Phase A–E: tiền điều kiện, bằng chứng, phản chứng, confirmation, invalidation và precedence | G3 |
| P4.2 | Trading Range candidate từ quan sát cơ học và lifecycle; biên, chất lượng, độ ổn định và cập nhật theo thời gian, không đòi hỏi chuỗi event Wyckoff hoàn chỉnh | P4.1; structure/zones/volume |
| P4.3 | Event detectors: Preliminary Support/Supply; Selling/Buying Climax; Automatic Rally/Reaction; Secondary Test; Spring/Upthrust; SOS/SOW; Last Point of Support/Supply; Back-up/Test | P4.2 |
| P4.4 | State machine Phase A–E, nhiều giả thuyết, lịch sử chuyển trạng thái, điều kiện hủy/khởi tạo giả thuyết | P4.3 |
| P4.5 | Ghép kết quả nhiều timeframe theo `as_of` và availability; bối cảnh khung lớn, cấu trúc khung trung, quan sát khung nhỏ | P4.1, P4.4; temporal contract P0 |
| P4.6 | `analyze_wyckoff()`, batch/update/reset, bindings, scenario replay và tài liệu giải thích từng đường chuyển | P4.5 |

Không yêu cầu mọi Trading Range phải đi đủ một chuỗi sự kiện mẫu. Những bước có thể bỏ qua, nhánh thay thế, trạng thái chưa xác định và giới hạn số giả thuyết phải được định nghĩa trước. Trạng thái phase, trạng thái event và trạng thái báo cáo là các khái niệm riêng, không suy ra trực tiếp lẫn nhau.

Tiền điều kiện về range được quy định riêng cho từng event. Các sự kiện giai đoạn đầu có thể xuất hiện khi range chưa xác định đầy đủ; không bắt mọi detector phải chờ range đã hoàn tất. State machine kết hợp candidate range và event theo thời gian, không tạo vòng yêu cầu range đã xác nhận để phát hiện chính những event cần cho việc xác nhận range.

Ứng dụng cung cấp các chuỗi timeframe đã chuẩn hóa cùng thời điểm đóng/khả dụng. Core kiểm tra và ghép theo contract, không tự tải dữ liệu hoặc quyết định lịch tổng hợp nến. Khi chưa có nến khung lớn mới đã đóng, chỉ dùng bối cảnh cũ còn hiệu lực hoặc báo thiếu bối cảnh theo cấu hình.

**Nghiệm thu G4 — Analysis:**

- [ ] Mỗi event trong P4.3 có rule ID/version, fixture dương tính, fixture phản ví dụ và kiểm thử thiếu dữ liệu/bối cảnh.
- [ ] Phase A–E có bảng transition/guard/precedence thực thi được; test bao phủ từng đường chuyển và từng guard từ chối đã đặc tả.
- [ ] Có kịch bản range không đủ bằng chứng, false breakout, giả thuyết bị vô hiệu và hai giả thuyết cùng tồn tại; không ép thành accumulation/distribution.
- [ ] Replay chỉ tạo/xác nhận event khi đủ dữ liệu; invalidation sau đó không xóa dấu vết event đã biết trước đó.
- [ ] Multi-timeframe có tests ngay trước/tại/sau thời điểm đóng và thời điểm khả dụng của nến lớn; kết quả không sử dụng giá đóng cuối kỳ trước lúc khả dụng.
- [ ] Snapshot đang phân tích chỉ chứa evidence có `known_at <= as_of`; batch và replay ở mọi mốc kiểm tra tương đương, kể cả ranh giới timeframe và reset lịch sử.
- [ ] Rust–Node.js–Python đạt parity cho event/phase/state; docs và báo cáo kiểm định theo mục 7.4 thể hiện giả thuyết có điều kiện cùng giới hạn của bộ quy tắc.

**Bằng chứng cần lưu:** rule catalog, transition matrix, replay timelines, multi-timeframe boundary report và báo cáo G4. Analysis Release gồm API phân tích đến Wyckoff; chưa được mô tả là đã có bộ tổng hợp DecisionReport hoàn chỉnh của P5.

### P5 — Evidence Engine và DecisionReport

**Mục tiêu:** tổng hợp dữ liệu từ các module thành báo cáo có thể kiểm tra lại từng nhận định.

**Điều kiện đầu vào:** G4 đạt; các module đã có facts/evidence/hypotheses theo schema P0 và các mở rộng đã quản lý phiên bản.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P5.1 | Chốt catalog quy tắc tổng hợp và mapping Facts → Context → Hypothesis → Confirmation → DecisionReport; dependency và điều kiện thiếu của từng quy tắc | G4 |
| P5.2 | Tập hợp, định danh, truy vết và loại evidence trùng theo quy tắc; giữ số đo, tham chiếu dữ liệu, rule version và availability | P5.1 |
| P5.3 | Đánh giá support/counter-evidence/pending conditions; lifecycle `watching`, `candidate`, `confirmed`, `invalidated` và giải thích transition | P5.2 |
| P5.4 | `evaluate_evidence()`, DecisionReport có schema/version, serialization JSON, kiểm tra reference và lịch sử trạng thái | P5.3 |
| P5.5 | Tích hợp orchestration/bindings; golden reports, replay, parity và ví dụ truy ngược từ kết luận đến dữ liệu | P5.4 |

DecisionReport phải chứa các trường của thiết kế: `schema_version`, `symbol`, `timeframe`, `as_of`, `engine_version`, `config_hash`, `hypothesis`, `status`, `evidence`, `counter_evidence`, `pending_conditions`, `confirmation`, `invalidation`. Metadata bổ sung phải đi qua phiên bản schema.

Quy tắc tổng hợp phải xác định cách xử lý bằng chứng tương quan hoặc dẫn xuất từ cùng quan sát; không đếm một dữ kiện thành nhiều bằng chứng độc lập. Thiếu module, thiếu dữ liệu và điều kiện phản bác không được chuyển thành xác nhận mặc định. Bản đầu không cần điểm số; nếu thêm điểm theo quy tắc thì phải ghi đúng bản chất và không gọi là xác suất thành công.

**Nghiệm thu G5:**

- [ ] Mỗi nhận định có đường truy vết đến rule ID/version, input hoặc số đo nguồn và `known_at`; không có tham chiếu evidence bị mất.
- [ ] Report bao gồm đủ bằng chứng ủng hộ/phản bác, điều kiện chưa đạt, xác nhận và vô hiệu theo schema; không bỏ phản chứng để giữ kết luận mong muốn.
- [ ] Từng trạng thái và đường chuyển có fixtures; thay đổi một điều kiện then chốt tạo ra thay đổi trạng thái dự kiến trong ca kiểm thử phản chứng.
- [ ] Evidence tương lai, evidence trùng và input module còn thiếu được xử lý theo quy tắc; đổi thứ tự evidence không đổi kết quả.
- [ ] JSON round-trip giữ schema/semantics; missing/non-finite tuân thủ contract; ID/config hash ổn định qua các bindings.
- [ ] Golden, replay/batch–incremental, revision/reset và parity đạt; cùng snapshot/config/version tái lập được cùng báo cáo.

**Bằng chứng cần lưu:** các report mẫu có giải thích, schema validation, provenance/trace tests, parity report và báo cáo G5.

### P6 — Feature Engine, hiệu năng và V1.0 Stable

**Mục tiêu:** xuất đặc trưng không rò rỉ tương lai và hoàn thiện một bộ thư viện có thể cài đặt, đo lường, bảo trì và phát hành.

**Điều kiện đầu vào:** G5 đạt; baseline P1/P2/P3 và các phiên bản contract hiện tại được lưu.

| Bước | Công việc và sản phẩm | Dependency |
| --- | --- | --- |
| P6.1 | Chốt feature catalog: tên, kiểu, đơn vị, công thức/nguồn module, version, warm-up, missing và thời điểm khả dụng | G5 |
| P6.2 | `extract_features()` batch/incremental; chọn feature/dependency, định dạng mảng/schema và bindings; adapter NumPy/Pandas/Polars tái sử dụng đường dữ liệu đã có | P6.1 |
| P6.3 | Bộ benchmark chuẩn cho một mã, watchlist, batch/update/replay, FFI và bộ nhớ; đo trên bản đúng hiện tại, chốt ngân sách tài nguyên/phạm vi workload | P6.2; baseline các phase trước |
| P6.4 | Tối ưu bottleneck có số đo: rolling, cấp phát, reuse, FFI, cache; chỉ xét song song/SIMD/zero-copy khi cần và giữ một đường chuẩn để đối chiếu | P6.3 |
| P6.5 | Hoàn thiện CI build/test/package trên ma trận hỗ trợ, kiểm tra artifacts, checksums, release candidate và cài sạch | P6.2, P6.4 |
| P6.6 | Hoàn thiện tài liệu, migration/changelog, kiểm tra traceability V1.0 và diễn tập release/rollback; lập hồ sơ nghiệm thu | P6.5 |

Thời điểm khả dụng của một feature không được sớm hơn đầu vào muộn nhất mà nó cần và các điều kiện xác nhận liên quan. Có thể ghi vị trí sự kiện gốc trong quá khứ, nhưng không điền giá trị xác nhận vào hàng feature quá khứ như thể nó đã biết từ lúc đó. Nhãn tương lai và chuẩn hóa bằng thống kê toàn bộ tập dữ liệu không thuộc feature causal của Core.

Ngân sách hiệu năng được chốt sau baseline và trước tối ưu, theo phần cứng/workload công bố. Không đặt mục tiêu tùy ý như “nhanh hơn X lần” trước khi đo. Nếu tối ưu không có lợi ích ổn định, giữ bản đơn giản đã đúng và công bố giới hạn; nếu bản hiện tại chưa đạt ngân sách đã chốt, chưa nghiệm thu G6 cho phạm vi đó.

**Nghiệm thu G6 — V1.0 Stable:**

- [ ] Feature catalog và mọi feature công bố có tests công thức/mapping, warm-up, missing, version và availability; không có nhãn tương lai lẫn vào API.
- [ ] Batch/incremental và Rust–Node.js–Python đạt parity cho feature values, names, ordering, mask và thời điểm khả dụng.
- [ ] Replay/prefix invariance đạt trên toàn pipeline; fixtures lịch sử sửa đổi và dữ liệu nhiều timeframe đạt theo revision semantics đã chốt.
- [ ] Có báo cáo thời gian, throughput, CPU và peak memory cho workload công bố; phân biệt chi phí Rust, FFI và adapter. Kết quả đạt ngân sách đã chốt trong P6.3.
- [ ] State/cache có giới hạn hoặc mô hình tăng trưởng được tài liệu hóa và kiểm tra bằng replay dài; không nhầm bộ nhớ output do ứng dụng giữ với bộ nhớ state bị giữ ngoài dự kiến.
- [ ] Mọi tối ưu được giữ lại đều có đối chiếu tính đúng và số đo trước/sau; các cấu hình worker được hỗ trợ giữ kết quả xác định.
- [ ] Tất cả API trong thiết kế có triển khai và ví dụ tương ứng; các binary/npm package/wheel cài sạch và chạy đúng trên toàn ma trận V1.0 công bố.
- [ ] G0–G5 còn đạt ở commit release candidate; ma trận nghiệm thu V1.0 ở mục 8 đủ bằng chứng, không còn lỗi chặn về nhân quả, công thức, state hoặc parity.
- [ ] Release candidate có version/tag dự kiến, changelog, artifact checksums, thông tin build và hướng dẫn migration/rollback; chưa publish nếu chưa được ủy quyền phát hành.

**Bằng chứng cần lưu:** feature leakage report, benchmark trước/sau, memory/replay report, CI/installation matrix, release manifest và báo cáo G6.

## 6. Quy tắc nghiệm thu áp dụng cho mọi giai đoạn

Một bước công việc chỉ được đánh dấu hoàn thành khi sản phẩm tồn tại, được tích hợp, có kiểm chứng phù hợp và tài liệu phản ánh đúng hành vi. Một giai đoạn chỉ đạt khi toàn bộ tiêu chí bắt buộc của cổng tương ứng đạt trên cùng commit nghiệm thu.

Hồ sơ mỗi cổng đặt tại `docs/acceptance/G0.md` đến `G6.md`, gồm:

- Commit, engine/schema/config version, fixture checksum và nền tảng/toolchain thực tế.
- Tiêu chí, test case hoặc bằng chứng tương ứng; lệnh đã chạy, kết quả, số ca đạt/thất bại/bỏ qua và nơi xem log.
- Sai lệch số quan sát được, tolerance đã dùng và giải thích thay đổi golden nếu có.
- Hạn chế, quyết định còn mở, phạm vi hỗ trợ và kết luận đạt/chưa đạt của người phụ trách nghiệm thu.

Không dùng “đã viết test” thay cho “test đã chạy và đạt”. Ca bị bỏ qua hoặc thiếu môi trường không được tính là đạt cho nền tảng tương ứng. Không chấp nhận ngoại lệ cho look-ahead, gán ngược xác nhận, sai công thức, phá schema, state không hợp lệ hoặc parity thất bại. Muốn thu hẹp phạm vi phát hành phải cập nhật kế hoạch/ma trận hỗ trợ trước, nêu tác động và được quyết định rõ ràng; không đổi tiêu chí sau khi thấy test thất bại để làm cổng đạt.

Các kiểm tra Rust nền tảng sau chỉ áp dụng sau khi workspace đã được tạo; cấu hình target/features và kiểm tra bindings phải theo manifest/CI thực tế:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI phải chạy đúng các test target của workspace và các môi trường Node.js/Python cần thiết. Với native bindings, ghi rõ yêu cầu link/runtime; không vô hiệu hóa test vì máy phát triển thiếu môi trường. Bổ sung runner phù hợp hoặc ghi nhận cổng chưa đạt.

## 7. Chiến lược kiểm chứng

### 7.1. Ma trận test tối thiểu

Các mã T01–T10 dưới đây là nhóm kiểm chứng dự kiến, không phải tên lệnh/test suite đã tồn tại.

| Mã | Nhóm kiểm chứng | Ca bắt buộc và điều kiện đạt | Bắt đầu |
| --- | --- | --- | --- |
| T01 | Contract/input | Rỗng/ngắn, mảng lệch, trùng/đảo timestamp, OHLC sai, volume âm, non-finite, cấu hình sai; kết quả/lỗi đúng schema, không sửa input | P0 |
| T02 | Công thức/golden | Ví dụ tính độc lập; vị trí hợp lệ đầu tiên; lookback khác nhau; giá/volume biên; sai lệch nằm trong tolerance đã chốt | P1 |
| T03 | Causal/non-repainting | Tiền tố, dữ liệu tương lai, nến chưa đóng/chưa khả dụng, pivot muộn, ranh giới timeframe; không xuất sớm tín hiệu | P1, mở rộng P4 |
| T04 | Batch–incremental | So sánh mọi mốc replay của fixtures ngắn và mốc lấy mẫu cố định ở chuỗi dài; state/reset/revision cho cùng kết quả | P1, mọi module mới |
| T05 | Lifecycle/invariants | Chuyển trạng thái hợp lệ/cấm, duplicate update, config mismatch, counter-evidence và invalidation; không có trạng thái bất hợp lệ | P1, mở rộng P3–P5 |
| T06 | Cross-language | Cùng fixture/config/as_of qua Rust–Node.js–Python; so schema, ID, trạng thái, mask, thời gian và số | P2, mọi API mới |
| T07 | Tính xác định | Lặp chạy, đảo thứ tự tập mã/evidence, reset, các worker được hỗ trợ; kết quả chuẩn hóa tương đương | P1, mở rộng P3/P6 |
| T08 | Packaging/runtime | Build, cài artifact vào môi trường sạch ngoài source tree, import/load và chạy API; đủ từng target công bố | P2 |
| T09 | Hiệu năng/tài nguyên | Batch/update, scaling theo số nến/mã, FFI/copy, CPU, peak memory và state retention; đạt ngân sách công bố | Baseline P1/P2; hoàn thiện P6 |
| T10 | Report/feature lineage | Truy ngược evidence/feature đến input/version và availability; JSON round-trip; không trộn nhãn tương lai | Evidence nền P1; P5/P6 |

Từ P0, fixture manifest phải có ít nhất: mã fixture, mục đích, phương pháp tạo/nguồn gốc được phép sử dụng, timezone/time semantics, revision, cấu hình, checksum và phương pháp xác minh kỳ vọng. Dữ liệu tổng hợp dùng seed cố định. Fixture thực tế nếu có phải được phép lưu/chia sẻ trong phạm vi repository; không cần kết nối nguồn trực tiếp để chạy test.

### 7.2. Tolerance và oracle độc lập

- Số thực được so theo quy tắc đã khai báo cho từng đầu ra, ví dụ `abs(a-b) <= atol + rtol * max(abs(a), abs(b))`. Giá trị `atol`/`rtol` phải được xác lập bằng công thức, thang đo và ca kiểm thử; không dùng một cặp tùy ý cho mọi chỉ báo.
- Enum, ID, timestamp, warm-up/missing mask và trạng thái phải khớp chính xác. Không dùng tolerance số thực để bỏ qua sự khác biệt về loại sự kiện hoặc thời điểm xác nhận.
- Golden được xác minh bằng phép tính tay, bảng tính đơn giản hoặc implementation tham chiếu độc lập theo cùng đặc tả. Nếu thư viện tham chiếu khác seed/smoothing thì phải giải thích sự khác biệt trước khi dùng làm oracle.
- Rule/state machine cần cả ca có sự kiện và ca gần giống nhưng không đủ điều kiện. Property tests bổ sung các bất biến, không thay thế kiểm chứng công thức.
- Khi golden thay đổi có chủ đích, lưu expected cũ/mới, nguyên nhân, rule/config version và ảnh hưởng tương thích. Không lấy output mới làm expected rồi mặc định đó là bằng chứng đúng.

### 7.3. Replay có tính nhân quả

1. Chọn snapshot và revision cố định; xác định các mốc `as_of`, gồm thời điểm đóng, thời điểm khả dụng và thời điểm xác nhận sự kiện.
2. Chạy batch trên dữ liệu chỉ khả dụng đến từng mốc; ghi kết quả chuẩn hóa.
3. Chạy cùng truy vấn `as_of` trên input có thêm tương lai. Kết quả phải như bước 2; Core phải tự tuân thủ giới hạn khả dụng.
4. Replay các update theo protocol đã định nghĩa, so kết quả tại từng mốc với batch. Dữ liệu đến muộn/sửa lịch sử phải đi qua nhánh revision/reset, không bị coi như append hợp lệ.
5. Thay đổi mạnh các nến tương lai và chạy lại truy vấn quá khứ; kết quả quá khứ không đổi khi revision quá khứ không đổi.
6. Với revision lịch sử mới, reset/replay và so với batch trên revision mới; ghi nhận thay đổi hợp lệ, không đánh đồng nó với repaint do sử dụng tương lai.

Đối với fixtures ngắn, so mọi tiền tố; với stress fixtures dài, dùng mốc lấy mẫu cố định có chứa mọi ranh giới quan trọng. Không chỉ so đầu ra cuối chuỗi, vì cách đó có thể bỏ sót tín hiệu xuất hiện quá sớm rồi biến mất.

### 7.4. Kiểm định quy tắc phân tích

Ở P3–P5, lập báo cáo đánh giá quy tắc bên cạnh unit/golden tests. Báo cáo phân biệt kết quả được suy ra chắc chắn từ công thức với cách diễn giải thị trường còn là giả thuyết.

Nếu điều chỉnh ngưỡng/tham số từ dữ liệu thực nghiệm, tách tập phát triển và kiểm định theo thời gian, cố định cấu hình trước khi đánh giá và không tinh chỉnh lại trên tập kiểm định rồi tiếp tục gọi nó là độc lập. Ghi phạm vi thị trường/timeframe, chất lượng snapshot, số trường hợp, lỗi thường gặp và giới hạn khái quát hóa; kế hoạch không quy định nguồn dữ liệu của các tập này.

Với nhãn Wyckoff/VSA có tính diễn giải, phải có tiêu chí gán nhãn và cách xử lý bất đồng/không chắc chắn. Chỉ công bố precision/recall khi có nhãn tham chiếu phù hợp và mô tả cách tạo nhãn. Khi chưa đủ cơ sở thực nghiệm, công bố kết quả là bộ quy tắc phân tích chưa hiệu chuẩn; không gán tỷ lệ thắng, độ chính xác dự báo hoặc xác suất thành công. Nghiệm thu logic phần mềm không thay thế bước nghiên cứu này.

### 7.5. Benchmark có thể tái lập

Workload khởi điểm đề xuất để đo: một mã với 1.000 nến, một mã với 100.000 nến, watchlist 100 mã × 10.000 nến và replay dài để kiểm tra state. Đây là kích thước thử nghiệm, không phải cam kết dung lượng/tốc độ; chốt lại cùng phần cứng và ngân sách trước nghiệm thu hiệu năng.

Mỗi lần đo phải ghi dataset/config/version, toolchain/target, CPU/RAM/OS, build profile, số worker, chế độ cold/warm và số lần lặp. Dùng build tối ưu cho số liệu hiệu năng; không so debug của bản này với release của bản khác. Tách thời gian chuẩn bị/chuyển đổi dữ liệu, Rust computation và serialization; ghi cả chi phí end-to-end.

Đo nhiều lần sau warm-up, công bố độ phân tán và trung vị; chỉ dùng percentile khi số mẫu đủ ý nghĩa. Đo scaling theo số mã/nến, peak memory và sự tăng bộ nhớ qua replay, thay vì chỉ một số thời gian tổng. Tối ưu chỉ được giữ khi lợi ích vượt nhiễu đo và toàn bộ kiểm tra đúng liên quan còn đạt.

## 8. Đối chiếu tiêu chí V1.0 với cổng nghiệm thu

| Tiêu chí từ `WYC_LIB.md` | Cổng thiết lập/hoàn tất | Bằng chứng chính |
| --- | --- | --- |
| Pure Rust Core độc lập framework | G0/G1; kiểm tra lại G6 | Dependency/architecture check; ví dụ Rust chạy độc lập |
| Module có trách nhiệm và dependency rõ ràng | G0; duy trì G1–G6 | ADR, dependency graph, manifest và review ranh giới |
| Public API và versioned contracts | G0; hoàn tất G6 | Contracts, API catalog, schema/config compatibility tests |
| Node.js/Python dùng chung thuật toán | G2; duy trì G3–G6 | Adapter review và parity trên cùng Rust implementation |
| Cross-language parity | G2; hoàn tất G6 | T06 trên toàn API và ma trận hỗ trợ |
| Market Structure không nhìn trước | G1; kiểm tra lại G6 | T03, pivot/BOS/CHoCH replay |
| Zones và Wyckoff Events có lifecycle | G3/G4 | T05, transition matrices và timeline fixtures |
| Phân biệt observation/hypothesis/confirmation | G3/G4/G5 | Schema, rule catalog và report fixtures |
| DecisionReport truy vết evidence | G5 | T10, reference integrity và report mẫu |
| Batch–incremental tương đương | G1; mở rộng đến G6 | T04 cho mọi module và toàn pipeline |
| Feature API không data leakage | G6 | T03/T10, feature catalog và availability tests |
| Golden và regression tests | G0/G1; duy trì đến G6 | Fixture manifests, oracle notes và regression reports |
| Benchmark CPU/RAM/throughput | Baseline G1/G2; hoàn tất G6 | T09 và báo cáo benchmark theo workload |
| Build/phân phối trên target | G2; hoàn tất G6 | T08, CI matrix và release artifacts |
| Tài liệu API/config/examples | Từng phase; hoàn tất G6 | Ví dụ chạy trong CI, README và migration guide |
| Kết quả tái lập theo dữ liệu/config/version | G0/G1; hoàn tất G6 | Snapshot/revision/hash manifests, T07 và replay reports |

G6 yêu cầu kiểm tra lại toàn bộ bảng trên ở commit release candidate, không chỉ dựa vào báo cáo của các commit cũ.

## 9. Rủi ro và cách kiểm soát

| Rủi ro | Biện pháp thực thi | Thời điểm xử lý |
| --- | --- | --- |
| Cùng tên chỉ báo nhưng khác công thức | Chốt seed/smoothing/warm-up và oracle trước coding | P0/P1 |
| Timestamp/is_closed không đủ chứng minh khả dụng | Đặc tả close/available time và snapshot semantics; từ chối tuyên bố point-in-time khi thiếu bằng chứng | P0; kiểm tra mọi phase |
| Wyckoff phụ thuộc vòng vào Evidence/Features | Định nghĩa kiểu evidence sớm; Wyckoff dùng typed observations; aggregation/export triển khai sau | P0/P4/P5/P6 |
| Incremental lệch batch hoặc giữ state cũ | Replay theo tiền tố; state identity; reset/replay là đường phục hồi mặc định | P1, mọi module mới |
| Quy tắc Wyckoff/VSA chủ quan hoặc ép nhãn | Guard/precedence và ca phản ví dụ; trạng thái chưa xác định; báo cáo giới hạn thực nghiệm | P3–P5 |
| Ranking dùng sai tập mã/thời gian | Snapshot input cùng as_of; metadata tập mã và revision do bên gọi cung cấp | P3 |
| ABI/FFI/package chỉ chạy trong máy phát triển | Thử đường gọi sớm, copy-safe trước, cài sạch ngoài source tree trên từng target | P2; duy trì đến P6 |
| Core tự đoán lịch giao dịch hoặc nến thiếu | Nhận thông tin chuẩn hóa/gap từ ứng dụng; xử lý theo contract | P0/P1/P4 |
| Tối ưu làm thay đổi sự kiện/tính nhân quả | Giữ đường chuẩn; benchmark trước/sau và chạy lại golden/replay/parity | Mọi tối ưu; tổng hợp P6 |
| Tăng phạm vi quá nhanh hoặc trì hoãn CI đến cuối | Lát cắt nhỏ, CI từ P0, cập nhật bindings/tests cùng module, gates theo dependency | Xuyên suốt |

## 10. Quản lý tiến độ, thay đổi và phát hành

- Dùng các mã P0.1–P6.6 làm đầu việc theo dõi; mỗi đầu việc có người phụ trách, dependency, sản phẩm, test và trạng thái. Khi lập kế hoạch, tất cả đều **chưa bắt đầu**.
- Thứ tự trạng thái đề xuất: chưa bắt đầu → đang triển khai → đang kiểm chứng → hoàn thành. Nếu chờ dependency/quyết định, ghi rõ yếu tố chặn; không tính “đã viết mã” là hoàn thành.
- Mỗi thay đổi thuật toán đi kèm đặc tả/fixture/test liên quan. Thay đổi public API/schema/config hoặc semantics thời gian phải đánh giá ảnh hưởng tới state, golden, bindings và người dùng thư viện trước khi tích hợp.
- Các quyết định lớn lưu bằng ADR ngắn: vấn đề, lựa chọn, lý do, hệ quả và cách kiểm chứng. Không mở rộng thêm framework, dịch vụ hoặc module ngoài phạm vi để giải quyết một vấn đề cục bộ.
- Chỉ ước lượng lịch sau khi P0.1 và đường xử lý đầu tiên cho biết năng lực build/test thực tế. Phân rã đầu việc quá lớn thành phần có thể nghiệm thu độc lập; không gắn một thời hạn cứng thiếu cơ sở cho cả P0–P6.
- Foundation sau G2 và Analysis sau G4 là các mốc tính năng. Chúng vẫn phải có package dùng được, changelog, giới hạn và bộ test của phạm vi đã công bố; có thể dùng phiên bản trước 1.0 theo chính sách version đã chốt.
- V1.0 Stable chỉ sau G6. Pipeline phát hành phải kiểm tra tag/version, commit đã nghiệm thu, artifacts và quyền phát hành; credentials do môi trường phát hành quản lý, không nằm trong Core hoặc fixtures.
- Diễn tập cài bản mới và quay lại artifact trước đó; nếu state/schema không tương thích, hướng dẫn reset/recompute từ dữ liệu do ứng dụng giữ. Không hứa rollback bằng cách dùng lại state không tương thích.

**Thứ tự bắt đầu thực tế:** thực hiện P0.1–P0.4 để chốt môi trường, kiến trúc và contracts; tiếp đó P0.5–P0.7 để có fixture/harness/CI. Chỉ sau G0 mới triển khai primitives và thuật toán của P1. Tác vụ viết kế hoạch này không tự khởi tạo workspace, cài dependency hay triển khai các giai đoạn.

## 11. Cập nhật triển khai P0 — 07/11/2026

Phần hiện trạng ở mục 1 và checklist ban đầu mô tả thời điểm lập kế hoạch.
Đợt triển khai P0 đã tạo workspace `contracts`/`wyc-lib`, typed contracts và
validation nền, ARCHITECTURE/CONTRACTS/ADR, đặc tả API và toàn bộ công thức P1,
fixtures tổng hợp/golden độc lập, test target foundation và workflow CI cơ bản.
Chưa triển khai thuật toán P1 hoặc bindings. `available_len`/validation contract
được thực thi sớm tại P0 để kiểm tra fixtures; hash/ID runtime thuộc P1.1,
state engine thuộc P1.6.

Trạng thái chi tiết P0.1–P0.7, kết quả local và các điều kiện G0 còn thiếu nằm
trong [báo cáo G0](docs/acceptance/G0.md). Git đã khởi tạo, commit nền P0 đã đẩy
lên voxthanhan/wyc-lib; clean clone Windows kiểm chứng thành công. CI đã được
kích hoạt nhưng bị chặn trước khi cấp runner; người dùng yêu cầu chạy CI sau.
Tại thời điểm kết thúc P0, G0 còn mở cho CI Windows/Linux và P1 chưa bắt đầu.
Quyết định tiếp tục sau đó được ghi riêng tại mục 12.

## 12. Ngoại lệ G0 và triển khai P1.1 — 07/11/2026

Theo yêu cầu trực tiếp của người dùng, **tạm hoãn CI và cho phép bắt đầu P1
dựa trên kiểm chứng local**. Ngoại lệ chỉ thay điều kiện bắt đầu P1, không đánh
dấu G0 đầy đủ hoặc CI Windows/Linux đạt. Bằng chứng checkout sạch P0, phạm vi,
giới hạn và điều kiện hoàn tất CI nằm tại [báo cáo G0](docs/acceptance/G0.md).
Các câu “chỉ sau G0” trong kế hoạch được áp dụng cùng ngoại lệ này.

Phạm vi đợt P1.1: tái sử dụng input/as_of validation P0, tạo view
read-only của tiền tố khả dụng, chuẩn hóa thứ tự module, kiểm tra numeric
series/status/time, canonical config hash và stable event ID. Không triển khai
rolling, chỉ báo, structure, state/update/reset hoặc analytical entry points.
P1.1 **hoàn thành với kiểm chứng local Windows**; trạng thái và bằng chứng tại
[báo cáo P1.1](docs/acceptance/P1.1.md). P1.2–P1.7
và G1 chưa được nghiệm thu.

## 13. Triển khai P1.2 — 07/11/2026

Theo xác nhận của người dùng, **chỉ tạm hoãn CI, vẫn kiểm chứng local như
P1.1**. Ngoại lệ G0 còn hiệu lực; không kích hoạt CI hoặc nghiệm thu Linux.

P1.2 thêm crate `indicators` với rolling sum/mean/min/max, population variance
và std. Input là slice FiniteF64 đã được caller chọn tiền tố; output có đủ
warm-up. Tổng bù Neumaier, variance hai lượt và tính lại từng cửa sổ nhằm
kiểm soát mất mát số/drift; arithmetic overflow có mã/path riêng qua
AnalysisError ở contracts. Không đổi công thức/seed/schema/config/hash cũ,
không thêm dependency bên ngoài. Tests/golden/oracle và kiểm biên/causal
replay được bổ sung; chưa có incremental state, benchmark hoặc chỉ báo P1.3.

P1.2 **hoàn thành với kiểm chứng local Windows**: format/Clippy/build và 28
integration tests đạt, oracle độc lập và kiểm dependency/liên kết đạt.
Trạng thái, bằng chứng local và giới hạn nghiệm thu tại
[báo cáo P1.2](docs/acceptance/P1.2.md). P1.3–P1.7 và G1 chưa nghiệm thu;
bản chuẩn O(N×min(n,N)) không được coi là baseline hiệu năng đã đo.

## 14. Triển khai P1.3 — 07/11/2026

Theo yêu cầu tiếp tục P1.3, **mở lại kiểm chứng CI cùng local**. Giữ bằng chứng
lịch sử G0/P1.1/P1.2; không tự đổi các kết luận CI đã hoãn của những đợt trước.

P1.3 thêm SMA/EMA/WMA, TR/ATR, ROC/Momentum, Volume SMA/RVOL; tái sử dụng mean
và tổng bù P1.2, giữ seed/warm-up/algorithm bundle p1-v1. `calculate_basic`
chỉ trả tập chín series có metadata/time; BasicIndicatorState nhận snapshot
tích lũy, kiểm identity/history/availability, update/reset atomic bằng batch
tính lại. Đây là state P1.3, chưa mở rộng sang AnalysisState/orchestrator P1.6.
Không thêm dependency, thay schema/config/hash hoặc tối ưu chưa đo.

Golden sáu case dùng oracle Fraction và smoothing dạng tổng hình học độc lập;
tests kiểm mọi tiền tố, future/revision, period khác mặc định, biên số và lỗi
state. P1.3 đạt local Windows và checkout sạch ở commit `bf97b14` với 42 tests,
format/Clippy/build/oracle/workspace PASS. Sau khi người dùng cho phép rõ
đưa mã lên repo công khai và chạy CI, nhánh đã push; run 42353272952 tại
commit `d86e6ff` đạt cả windows-latest và ubuntu-24.04, hai artifact log đã
upload. Source/tests/config của hai commit không khác nhau. Điều kiện CI
hoãn của G0 đã đóng tại commit hiện tại; P1.3 đạt phạm vi nghiệm thu, không
coi là toàn G1. Phạm vi, metadata artifact và giới hạn tải log tại
[báo cáo P1.3](docs/acceptance/P1.3.md). P1.4–P1.7/G1 còn mở.

## 15. Triển khai P1.4 — 07/11/2026

P1.4 thêm RSI Wilder, MACD line/signal/histogram, DI/DX/ADX và Bollinger
population std trên primitives P1.2/P1.3. `calculate_advanced` trả riêng 11
series có metadata/time và warm-up từng component. Không đổi seed/rule/schema,
config/hash/dependency; BasicIndicatorState vẫn chỉ quản lý P1.3, state và
orchestrator toàn P1 thuộc P1.6.

Tám golden mới được tạo bằng oracle Fraction/Decimal độc lập với Rust;
kiểm mọi sample, prefix với future bị thay đổi, availability/revision và
biên số. Làm rõ giới hạn roundoff của bands gần triệt tiêu ở thang 1e100,
giữ nguyên golden exact-real. Local Windows và checkout sạch tại `5caf81b`
đạt format/Clippy/build, 54 tests, oracle và workspace checks. Sau khi người
dùng cho phép rõ đẩy source công khai và chạy CI, run 42355665535 tại commit
`4521da9` đạt cả windows-latest và ubuntu-24.04; hai artifact log đã upload.
51 source blobs tại CI khớp bytes bản local/checkout sạch. **P1.4 hoàn thành
trong phạm vi đã giao**; bằng chứng và giới hạn tải payload artifact tại
[báo cáo P1.4](docs/acceptance/P1.4.md). P1.5–P1.7 và toàn G1 còn mở.

## 16. Triển khai P1.5 — 07/11/2026

P1.5 thêm crate `market_structure` chỉ phụ thuộc contracts; dùng
`calculate_structure(&MarketData, &AnalysisConfig, as_of)` trước API điều phối
P1.6. Pivot strict với L/R nến đóng/khả dụng; labels cùng loại, trend, BOS/CHoCH
theo close, strength và evidence confirmed có ID/time ổn định. Break xét trước
xác nhận pivot và cập nhật trend; không tiêu thụ reference khi trend unknown,
không chọn tùy ý khi hai hướng cùng kích hoạt. Giữ quy tắc p1-v1/schema/config,
không thêm dependency ngoài, state chung hoặc benchmark ngoài phạm vi.

15 golden từ oracle độc lập liệt kê cửa sổ/tập pivot, Fraction và hashlib;
tests đối chiếu mọi trường, hand-trace, replay/future, delayed confirmation,
revision, dual/equal, xung đột và overflow/underflow. Kết quả local, checkout
sạch, CI và tiêu chí nghiệm thu tại [báo cáo P1.5](docs/acceptance/P1.5.md).
P1.5 đạt local Windows và checkout sạch ở commit `6cbc517`: format/Clippy/build,
70 tests, oracle và workspace checks PASS, 57 source fingerprints khớp. Sau khi
người dùng cho phép rõ push source P1.5 lên repository công khai, run 42393366227
tại commit `c7de17a` đạt cả windows-latest và ubuntu-24.04, hai artifact log đã
upload; 57 Git source blobs khớp local/checkout sạch. **P1.5 hoàn thành trong
phạm vi đã giao**; metadata và giới hạn tải payload tại báo cáo trên.
Toàn G1 còn mở cho tích hợp/state P1.6 và nghiệm thu/baseline P1.7.

## 17. Triển khai P1.6 — 08/11/2026

Theo yêu cầu tiếp tục P1.6, kiểm chứng local và CI Windows/Linux, rồi tự tích
hợp main khi đạt. Thêm calculate_indicators/analyze_structure/analyze_batch và
AnalysisState new/update/reset tại wyc_lib; chỉ chạy module được chọn. Catalog
đủ 25 series qua engine/primitives cũ, không sao chép công thức. State giữ prefix
khả dụng, identity/cursor/metadata và dùng batch tính lại, thay thế atomic.

Chốt trước code [ADR-0003](docs/adr/0003-p1-orchestration-state.md): API đơn
module nhận toàn AnalysisConfig thay config con của draft để giữ hash nhất
quán; thêm DTO ClosedBar/ClosedBarUpdate, không đổi API engine đã triển khai,
seed/rule/schema/config/hash hoặc dependency. Golden cũ giữ nguyên và đối chiếu
qua API mới. Target p1_orchestration kiểm replay, identity/cursor, revision/reset,
atomic errors và module selection. Workspace verifier buộc bảy test targets.

Tiến độ và bằng chứng thực tế tại [báo cáo P1.6](docs/acceptance/P1.6.md).
P1.6 đạt local/checkout sạch Windows tại `4460753`: format/Clippy/build, 85
tests, oracle/workspace PASS, 60 source fingerprints khớp. Sau xác nhận rõ của
người dùng cho phép đẩy source công khai, CI run 42395906710 đúng SHA đạt cả
windows-latest và ubuntu-24.04, hai artifact log đã upload. Metadata CI/đối
chiếu nguồn và giới hạn tải payload trong báo cáo. **P1.6 hoàn thành phạm vi
được giao**; đã tích hợp main tại `5af2383`, run main 42396385077 đạt cả
Windows/Linux, tree khớp bản nghiệm thu. P1.7/toàn G1 còn mở.

## 18. Triển khai P1.7 — 08/11/2026

Theo yêu cầu hoàn tất replay/batch–incremental/revision-reset/golden-regression,
baseline Rust, ví dụ và tài liệu, kiểm chứng local/CI Windows/Linux rồi tự đưa
vào main khi đạt. Người dùng cho phép rõ đẩy mã lên voxthanhan/wyc-lib công khai.
Không đổi thuật toán/Core/config/hash/schema, dependency hoặc golden cũ.

Target p1_replay thêm 1008 prefix traces trên 15 fixtures, ba lựa chọn module
và hai lịch availability; reset revision ở nửa history rồi append phần còn lại,
first-valid lookback 1/2/3/8 và closed-form ramp. Ví dụ Rust chạy input → append
→ revision/reset → JSON. Baseline release 12 workload, năm mẫu sau warm-up,
đo CPU/time/peak RSS process ngoài Core; không tuyên bố O(1) hoặc tối ưu.
Runner local đã đạt 88 tests cùng format/Clippy/build/example/baseline/oracle/
workspace. Trạng thái checkout sạch/CI/main và tiêu chí G1 cập nhật tại
[báo cáo G1](docs/acceptance/G1.md). Local/checkout sạch Windows tại `7c57220`
đạt 88 tests và toàn checks; CI [run 42398202869](https://github.com/voxthanhan/wyc-lib/actions/runs/42398202869)
đạt Windows/Linux đúng SHA, cả hai job kiểm chứng/upload thành công. **G1 đạt**;
65 source fingerprints khớp. CI được xác minh qua HTML công khai sau khi REST
API hết hạn mức; payload artifact chưa tải/kiểm digest. Tích hợp main ghi trong
báo cáo G1; chưa triển khai P2 hoặc phát hành package.

Đã tích hợp main tại `223d742`, tree khớp nhánh nghiệm thu;
[CI main 42399120154](https://github.com/voxthanhan/wyc-lib/actions/runs/42399120154)
đạt Windows/Linux, hai job kiểm chứng/upload success, 65 source blobs khớp.
Báo cáo G1 lưu bằng chứng và giới hạn payload artifact. **P1.7/G1 hoàn thành**;
P2/G2/Foundation release còn ngoài phạm vi triển khai lần này.

## 19. Tối ưu state và kiểm ZIP sau G1 — 08/11/2026

Theo yêu cầu tiếp theo của người dùng, tối ưu cả AnalysisState và
BasicIndicatorState bằng cache/kernel dùng chung, giữ batch làm chuẩn và toàn
semantics/golden, atomic lỗi/reset, identity/revision/as_of. Chốt
[ADR-0004](docs/adr/0004-incremental-state.md); không tuyên bố tổng API O(1)
vì full output clone, evidence insertion và snapshot validation còn tuyến tính.
Tải ZIP CI cũ/mới và kiểm độc lập digest/CRC/summary/checks/traces/baseline;
kiểm local, checkout sạch, CI Windows/Linux rồi tự tích hợp main khi đạt.
[Báo cáo G1 bổ sung](docs/acceptance/G1-incremental.md) ghi bằng chứng thực tế.

Đợt incremental sau G1 đã đạt tại 45752f1 (local/checkout sạch, CI run 42435746649)
và tích hợp main tại 507fdad (CI run 42436169902): Windows/Linux đều PASS,
93 Rust tests + 5 Python controls, 69 source fingerprints khớp, 1008 prefix
hashes giữ nguyên. Tám ZIP bốn run cũ/mới đã tải, kiểm digest/CRC và payload
summary/checks/tests/traces/baseline độc lập. G1 bổ sung đạt; report clone O(N),
snapshot Basic O(M) và noisy batch-only baseline được công bố, không claim O(1).


## P2.2 — Node.js API P1

Node có `analyze_batch(request)`, `calculate_indicators(data, config, as_of)`,
`analyze_structure(data, config, as_of)` và `AnalysisState(request)` nhận object.
State có result/identity/update_closed_bar/reset/close; JSON pilot P2.1 vẫn giữ.
Khai báo TypeScript bao phủ DTO P1, 25 series, cấu trúc/evidence và tagged status.
Input nhận array đặc/Float64Array cho cột số; copy-owned, không ép kiểu hoặc giữ
reference. NaN/Infinity, unsafe integer, sparse/accessor/cycle được chặn trước
JSON serialization; validation nghiệp vụ và mọi phép tính vẫn ở Rust.
Sync Node Runtime, không dùng browser/Edge; Python typed thuộc P2.3, full parity
thuộc P2.4. [ADR-0006](docs/adr/0006-p2-node-typed.md) và
[báo cáo P2.2](docs/acceptance/P2.2.md) ghi contract và mức kiểm chứng thực tế.
