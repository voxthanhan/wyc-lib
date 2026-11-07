\# wyc-lib V1.0

\## Thiết kế kiến trúc thư viện lõi phân tích định lượng đa nền tảng

## Quy ước tên thư viện và package

Tên chính thức của thư viện là **wyc-lib**. Khi khởi tạo workspace và bindings, áp dụng thống nhất:

| Thành phần | Tên/đường dẫn |
| --- | --- |
| Thư viện, repository và thư mục gốc | `wyc-lib` |
| Cargo package cung cấp public API thống nhất | `wyc-lib` |
| Crate thư viện và định danh dùng trong mã Rust | `wyc_lib` |
| Thư mục crate điều phối | `crates/wyc_lib/` |
| npm package | `wyc-lib` |
| Python distribution | `wyc-lib` |
| Python import module | `wyc_lib` |

`wyc_lib` là định danh dùng trong mã và đường dẫn crate của cùng thư viện `wyc-lib`. Tên các module nghiệp vụ giữ theo trách nhiệm trong kiến trúc.



\*\*Định hướng:\*\* One Core – Multiple Applications  

\*\*Ngôn ngữ lõi:\*\* Rust  

\*\*Nền tảng:\*\* Node.js / Next.js, Python, Web Services, Desktop  

\*\*Kiến trúc:\*\* Modular · Evidence-Driven · Deterministic · High Performance



\---



\# I. TẦM NHÌN VÀ MỤC TIÊU



\## 1. Tầm nhìn



Xây dựng \*\*wyc-lib\*\* – thư viện lõi phân tích định lượng hiệu năng cao, độc lập với ứng dụng, cung cấp một hệ thống thuật toán thống nhất cho nhiều dự án phân tích thị trường tài chính.



Thư viện tập trung vào việc nhận diện, đo lường và diễn giải hành vi thực tế của thị trường dựa trên:



\- Price Action và Market Structure.

\- Volume Spread Analysis (VSA).

\- Wyckoff Method.

\- Relative Strength và Momentum.

\- Price Zones và Volume Profile.

\- Evidence-Driven Analysis.

\- Các đặc trưng định lượng phục vụ Machine Learning và Backtesting.



\*\*Triết lý cốt lõi: What I See – Evidence Before Prediction.\*\*



Mọi nhận định phải xuất phát từ dữ liệu quan sát được, có bằng chứng, có điều kiện xác nhận và có khả năng kiểm chứng.



\## 2. Mục tiêu



1\. \*\*Single Analytical Source of Truth:\*\* Một bộ thuật toán và quy tắc phân tích dùng chung, tránh sai lệch logic giữa các dự án.

2\. \*\*Write Once, Run Across Platforms:\*\* Viết thuật toán bằng Rust, cung cấp bindings cho Node.js và Python.

3\. \*\*High Performance:\*\* Tối ưu tính toán trên mảng, xử lý batch, incremental và vectorization khi phù hợp.

4\. \*\*Deterministic:\*\* Cùng dữ liệu, cấu hình, phiên bản thuật toán và thời điểm phân tích phải tạo kết quả tương đương.

5\. \*\*Evidence-Driven:\*\* Mọi nhận định có thể truy vết tới dữ liệu và điều kiện kích hoạt.

6\. \*\*Non-Repainting:\*\* Không sử dụng thông tin tương lai để tạo tín hiệu lịch sử.

7\. \*\*Modular \& Extensible:\*\* Các module độc lập, có thể sử dụng riêng hoặc kết hợp.

8\. \*\*Reusable:\*\* Dùng chung cho phân tích realtime, nghiên cứu, ML và backtest.

9\. \*\*Versioned \& Testable:\*\* Thuật toán, cấu hình, schema và kết quả được quản lý phiên bản.



\## 3. Nguyên tắc thiết kế



\- \*\*Core tính toán, Application điều phối.\*\*

\- Không phụ thuộc framework, giao diện, database hoặc nhà cung cấp dữ liệu.

\- Không nhúng API key, nghiệp vụ giao dịch hoặc quy tắc quản lý danh mục vào lõi.

\- Không tự động suy diễn dữ liệu thiếu hoặc thay đổi dữ liệu nguồn.

\- Phân biệt rõ quan sát thực tế, giả thuyết và kết luận đã xác nhận.

\- Không mặc định đồng nhất khối lượng cao với hành vi mua của tổ chức.

\- Ưu tiên độ chính xác, tính nhất quán và kiểm chứng trước tối ưu tốc độ.

\- Chỉ tối ưu đa luồng, SIMD hoặc zero-copy khi benchmark chứng minh có lợi ích.



\---



\# II. KIẾN TRÚC TỔNG THỂ



\## 1. Mô hình Core – Bindings – Applications



\### Application Layer



Các hệ thống sử dụng thư viện:



\- SSI Technical \& Market Intelligence.

\- Wyckoff Intelligence System (WIS).

\- Evidence-Driven Trading System.

\- Python Machine Learning và Backtesting.

\- Các ứng dụng phân tích thị trường tương lai.



Mỗi ứng dụng tự quản lý giao diện, dữ liệu đầu vào, cache, database và nghiệp vụ đặc thù.



\### Binding Layer



\*\*Node.js Binding\*\*

\- Công nghệ: NAPI-RS.

\- Đầu ra: Native Addon `.node`, đóng gói thành npm package.

\- Phục vụ Next.js, Node.js backend và các dịch vụ JavaScript.



\*\*Python Binding\*\*

\- Công nghệ: PyO3 + Maturin.

\- Đầu ra: Python wheel `.whl`.

\- Tương thích môi trường nghiên cứu với NumPy, Pandas và Polars thông qua adapter phù hợp.



\*\*WASM Binding – Tùy chọn mở rộng\*\*

\- Dành cho những trường hợp cần tính toán trực tiếp trong trình duyệt.

\- Không thuộc phạm vi triển khai ban đầu.



\### Rust Core Layer



Bao gồm các module thuật toán, hợp đồng dữ liệu, bộ máy đánh giá bằng chứng và API phân tích.



\### Data Layer – Do ứng dụng quản lý



\- SSI API.

\- vnstock hoặc các nguồn OHLCV khác.

\- Realtime Data Provider.

\- Cache và Database.



Rust Core chỉ nhận dữ liệu đã được chuẩn bị theo hợp đồng đầu vào.



\## 2. Cấu trúc module



| Module | Chức năng |

|---|---|

| `contracts` | Định nghĩa các kiểu dữ liệu, schema và cấu hình |

| `indicators` | Tính toán chỉ báo định lượng |

| `market\_structure` | Nhận diện cấu trúc và xu hướng |

| `price\_zones` | Phát hiện, quản lý các vùng giá |

| `volume\_engine` | Phân tích khối lượng, VSA và Volume Profile |

| `relative\_strength` | Phân tích sức mạnh tương đối |

| `wyckoff\_engine` | Nhận diện sự kiện, giai đoạn và bối cảnh Wyckoff |

| `evidence\_engine` | Tổng hợp bằng chứng và tạo báo cáo |

| `feature\_engine` | Xuất đặc trưng cho ML và backtest |



Các module dùng chung `contracts` nhưng không được tạo phụ thuộc vòng.



\---



\# III. THIẾT KẾ HỆ THỐNG THUẬT TOÁN



\## 1. Indicators Engine



\*\*Mục tiêu:\*\* Cung cấp các phép tính nền tảng thống nhất cho toàn bộ thư viện.



Chức năng:



\- SMA, EMA, WMA.

\- ATR, True Range.

\- RSI, MACD, ADX.

\- ROC, Momentum.

\- Bollinger Bands.

\- Rolling Min/Max.

\- Volume SMA, RVOL.

\- Standard Deviation.

\- Các phép toán rolling và thống kê định lượng.



Yêu cầu:



\- Hỗ trợ dữ liệu mảng.

\- Xác định rõ warm-up period.

\- Không tự ý điền dữ liệu thiếu.

\- Hỗ trợ tính toán batch và incremental đối với thuật toán phù hợp.

\- Tái sử dụng kết quả trung gian để tránh tính toán trùng lặp.



\## 2. Market Structure Engine



\*\*Mục tiêu:\*\* Nhận diện cấu trúc thị trường và các thay đổi thực tế trong hành vi giá.



Chức năng:



\- Swing High / Swing Low.

\- Higher High / Higher Low.

\- Lower High / Lower Low.

\- Uptrend / Downtrend / Sideways.

\- Break of Structure (BOS).

\- Change of Character (CHoCH).

\- Internal và External Structure khi có định nghĩa rõ.

\- Structure Strength và trạng thái xác nhận.



\*\*Quy tắc bắt buộc:\*\*



Một pivot chỉ được coi là xác nhận khi đủ dữ liệu theo quy tắc phát hiện.



Mỗi sự kiện lưu thời điểm thực tế xảy ra và thời điểm hệ thống có đủ thông tin để xác nhận.



Không được gán ngược tín hiệu đã xác nhận vào thời điểm mà thông tin chưa tồn tại.



\## 3. Price Zones Engine



\*\*Mục tiêu:\*\* Xác định các vùng giá có ý nghĩa dựa trên cấu trúc, phản ứng giá và biến động.



Chức năng:



\- Support / Resistance Zones.

\- Swing Clustering.

\- ATR-Adaptive Zones.

\- Zone Strength.

\- Số lần kiểm định vùng.

\- Breakout / Retest.

\- Zone Invalidation.

\- Zone Lifecycle.



Mỗi vùng giá có các trạng thái:



`candidate → active → tested → broken / invalidated`



Độ rộng vùng được xác định bằng quy tắc cấu hình, có thể kết hợp ATR, swing clustering và biên dao động giá.



Không mặc định mọi vùng hỗ trợ/kháng cự đều đại diện cho hoạt động của dòng tiền lớn.



\## 4. Volume \& VSA Engine



\*\*Mục tiêu:\*\* Phân tích tương quan giữa nỗ lực và kết quả của hành vi giá.



\### Thành phần phân tích



\- Volume trung bình và RVOL.

\- Candle Spread.

\- Close Location Value.

\- Price Progress.

\- Effort vs Result.

\- Volume Expansion / Contraction.

\- Volume Dry-up.

\- Absorption Candidates.

\- No Supply / No Demand Candidates.

\- Stopping Volume Candidates.

\- Climax Candidates.



\### Volume Profile



\- Point of Control (POC).

\- Value Area High (VAH).

\- Value Area Low (VAL).

\- Low Volume Nodes (LVN).

\- Phân phối khối lượng theo vùng giá.



Volume Profile xây dựng từ OHLCV chỉ mang tính xấp xỉ, không đồng nhất với dữ liệu khối lượng giao dịch thực tế ở từng mức giá.



\### Nguyên tắc VSA



Kết quả phân tích phải phân biệt:



\- \*\*Observation:\*\* Hành vi thực tế quan sát được.

\- \*\*Interpretation:\*\* Giả thuyết về cung/cầu.

\- \*\*Confirmation:\*\* Bằng chứng bổ sung xác nhận giả thuyết.



Không khẳng định Smart Money đang mua hoặc bán chỉ dựa trên OHLCV.



\## 5. Relative Strength Engine



\*\*Mục tiêu:\*\* Đánh giá sức mạnh của cổ phiếu so với chỉ số hoặc tập hợp tham chiếu.



Chức năng:



\- Mansfield Relative Strength.

\- RS Momentum.

\- RS Slope.

\- Price Relative.

\- Relative Performance đa khung thời gian.

\- RS Ranking theo tập cổ phiếu được cung cấp.



Các phép tính xếp hạng theo tập cổ phiếu phải sử dụng dữ liệu tại cùng thời điểm đánh giá, không dùng thành phần hoặc dữ liệu tương lai.



Các phương pháp xếp hạng như RS IBD cần có công thức và cấu hình phiên bản riêng, không mặc định đồng nhất với thuật toán độc quyền.



\## 6. Wyckoff Engine



\*\*Mục tiêu:\*\* Xây dựng hệ thống phân tích Wyckoff có chiều sâu dựa trên bối cảnh, cấu trúc giá và khối lượng.



\### Thành phần



\*\*Trading Range Detection\*\*

\- Nhận diện vùng tích lũy/phân phối tiềm năng.

\- Xác định biên trên, biên dưới và trạng thái vùng.

\- Đánh giá độ ổn định và chất lượng Trading Range.



\*\*Wyckoff Events\*\*

\- Preliminary Support / Supply.

\- Selling Climax / Buying Climax.

\- Automatic Rally / Reaction.

\- Secondary Test.

\- Spring / Upthrust.

\- Sign of Strength / Sign of Weakness.

\- Last Point of Support / Supply.

\- Back-up và Test.



\*\*Wyckoff Phase Engine\*\*

\- Phase A.

\- Phase B.

\- Phase C.

\- Phase D.

\- Phase E.



\### State Machine



Thay vì xác định một sự kiện độc lập rồi kết luận ngay, hệ thống quản lý diễn biến Wyckoff theo thời gian.



\- Mỗi giai đoạn có điều kiện hình thành.

\- Mỗi sự kiện có bằng chứng hỗ trợ và phản bác.

\- Có thể tồn tại nhiều giả thuyết về Phase.

\- Trạng thái chỉ chuyển khi đáp ứng điều kiện định nghĩa.

\- Giả thuyết có thể bị vô hiệu khi thị trường phát triển trái với điều kiện kỳ vọng.



\### Multi-Timeframe



Cho phép kết hợp:



\- Khung lớn: Xác định bối cảnh.

\- Khung trung gian: Nhận diện cấu trúc và sự kiện.

\- Khung nhỏ: Quan sát chi tiết hành vi giá.



Chỉ sử dụng nến đã đóng tại thời điểm phân tích, không được sử dụng kết quả cuối kỳ của một nến lớn chưa hoàn thành.



Wyckoff Engine phải trả về bằng chứng và các giả thuyết có điều kiện, không tự động biến mọi Trading Range thành Accumulation hoặc Distribution.



\## 7. Evidence \& Decision Engine



\*\*Mục tiêu:\*\* Tổng hợp kết quả từ các module để tạo nhận định nhất quán, có thể giải thích và kiểm chứng.



\### Cấu trúc xử lý



\*\*Facts → Context → Hypothesis → Confirmation → DecisionReport\*\*



\- `Facts`: Những quan sát định lượng trực tiếp.

\- `Context`: Bối cảnh cấu trúc và vị trí thị trường.

\- `Hypothesis`: Giả thuyết phân tích.

\- `Confirmation`: Điều kiện xác nhận theo quy tắc.

\- `DecisionReport`: Kết quả, bằng chứng và trạng thái.



\### Trạng thái



\- `watching`: Đang theo dõi.

\- `candidate`: Có giả thuyết đủ điều kiện sơ bộ.

\- `confirmed`: Đáp ứng bộ điều kiện xác nhận.

\- `invalidated`: Giả thuyết hoặc thiết lập không còn hiệu lực.



`confirmed` chỉ có nghĩa đã đáp ứng quy tắc xác nhận của thuật toán, không bảo đảm giá sẽ vận động theo một hướng nhất định.



\### Đầu ra



DecisionReport cung cấp:



\- Mã và loại sự kiện.

\- Trạng thái đánh giá.

\- Bằng chứng ủng hộ.

\- Bằng chứng phản bác.

\- Các điều kiện chưa đạt.

\- Điều kiện xác nhận.

\- Điều kiện vô hiệu.

\- Thời điểm sự kiện.

\- Thời điểm xác nhận.

\- Phiên bản thuật toán và cấu hình.



\*\*Không bắt buộc tạo điểm số xác suất.\*\* Nếu sử dụng confidence hoặc xác suất thành công, phải phân biệt điểm đánh giá theo quy tắc với xác suất đã được hiệu chuẩn bằng dữ liệu thực nghiệm.



\## 8. Feature Engine



\*\*Mục tiêu:\*\* Cung cấp đầu ra định lượng phục vụ Machine Learning và Backtesting.



Chức năng:



\- Xuất feature theo mốc thời gian.

\- Kết hợp đặc trưng từ các module.

\- Cung cấp feature batch.

\- Cung cấp feature incremental.

\- Quản lý tên, kiểu, phiên bản và thời điểm khả dụng của feature.



Feature Engine không thực hiện:



\- Huấn luyện mô hình.

\- Tối ưu siêu tham số.

\- Dự báo trực tiếp.

\- Mô phỏng khớp lệnh.

\- Quản lý danh mục.



Các chức năng trên thuộc ứng dụng ML hoặc Backtest sử dụng thư viện.



\---



\# IV. DATA CONTRACTS \& API



\## 1. MarketData Contract



Đầu vào tối thiểu:



\- `symbol`

\- `timeframe`

\- `timestamp`

\- `open`

\- `high`

\- `low`

\- `close`

\- `volume`



Metadata:



\- `source`

\- `timezone`

\- `adjustment\_mode`

\- `is\_closed`



Quy định:



\- Timestamp sử dụng chuẩn UTC thống nhất, ví dụ Unix milliseconds.

\- Dữ liệu sắp xếp theo thời gian tăng dần.

\- Giá sử dụng `float64`, có quy định về độ chính xác.

\- Phải khai báo cách hiểu timestamp đầu/cuối nến.

\- Không tự ý nội suy nến thiếu.

\- Kiểm tra tối thiểu tính hợp lệ đầu vào.

\- Việc điều chỉnh giá, đồng bộ dữ liệu và quản lý lịch giao dịch thuộc Data Provider.



\## 2. Analysis Contract



Mỗi yêu cầu phân tích gồm:



\- Dữ liệu OHLCV.

\- Danh sách module cần chạy.

\- Cấu hình thuật toán.

\- Thời điểm phân tích `as\_of`.

\- Phiên bản cấu hình.



Lõi chỉ được sử dụng thông tin thực sự khả dụng tại `as\_of`.



\## 3. Evidence Contract



Mỗi bằng chứng lưu:



\- `evidence\_id`

\- `event\_type`

\- `observed\_at`

\- `known\_at`

\- `source\_module`

\- `measured\_values`

\- `rule\_id`

\- `status`



Trong đó `known\_at` đặc biệt quan trọng để ngăn Look-ahead Bias.



\## 4. DecisionReport Contract



Schema logic:



\- `schema\_version`

\- `symbol`

\- `timeframe`

\- `as\_of`

\- `engine\_version`

\- `config\_hash`

\- `hypothesis`

\- `status`

\- `evidence`

\- `counter\_evidence`

\- `pending\_conditions`

\- `confirmation`

\- `invalidation`



Đầu ra phải có cấu trúc ổn định, dễ chuyển đổi sang JSON để dùng trong Web, Python và hệ thống báo cáo.



\## 5. Public API



Cung cấp các nhóm API:



\- `calculate\_indicators()`

\- `analyze\_structure()`

\- `detect\_price\_zones()`

\- `analyze\_volume()`

\- `analyze\_relative\_strength()`

\- `analyze\_wyckoff()`

\- `evaluate\_evidence()`

\- `extract\_features()`

\- `analyze\_batch()`

\- `update\_closed\_bar()`



Các hàm trên là hợp đồng API mục tiêu. Chữ ký, kiểu trả về và cơ chế quản lý trạng thái phải được đặc tả trước khi triển khai binding.



\---



\# V. WORKFLOW \& PROCESSING PIPELINE



\## 1. Batch Processing



1\. Application tải dữ liệu.

2\. Chuẩn hóa dữ liệu theo MarketData Contract.

3\. Chuyển mảng cho Rust Core.

4\. Tính toán Indicators.

5\. Phân tích các module cấu trúc, vùng giá, khối lượng và sức mạnh tương đối theo dependency graph.

6\. Wyckoff Engine sử dụng các feature cần thiết.

7\. Evidence Engine tổng hợp bằng chứng.

8\. Trả về AnalysisResult / DecisionReport.

9\. Application xử lý giao diện, lưu trữ, chiến lược hoặc ML.



\## 2. Incremental Processing



Sau lần tính toán đầu tiên:



1\. Nhận nến mới đã đóng.

2\. Kiểm tra tính liên tục và trạng thái dữ liệu.

3\. Cập nhật các chỉ báo hỗ trợ incremental.

4\. Cập nhật các cấu trúc và vùng chịu ảnh hưởng.

5\. Đánh giá các sự kiện mới hoặc thay đổi trạng thái.

6\. Tạo kết quả mới.



Nếu dữ liệu lịch sử bị sửa đổi, hệ thống phải hỗ trợ tính lại từ điểm ảnh hưởng, không mặc định incremental vẫn hợp lệ.



\## 3. Batch – Incremental Consistency



Với cùng dữ liệu, cấu hình và `as\_of`, kết quả cuối cùng của batch và incremental phải tương đương theo tiêu chí kiểm thử đã quy định.



Đây là tiêu chí nghiệm thu bắt buộc trước khi dùng thư viện cho realtime và backtest.



\---



\# VI. HIỆU NĂNG VÀ TỐI ƯU



\## 1. Nguyên tắc



\- Ưu tiên thuật toán có độ phức tạp hợp lý.

\- Sử dụng cấu trúc mảng contiguous khi phù hợp.

\- Hạn chế cấp phát bộ nhớ không cần thiết.

\- Tái sử dụng chỉ báo và kết quả trung gian.

\- Tránh chuyển đổi dữ liệu lặp lại qua FFI.

\- Tối ưu rolling và incremental computation.

\- Kiểm soát dung lượng bộ nhớ của state và cache.



\## 2. Parallel Processing



\- Một mã cổ phiếu: ưu tiên thực thi đơn luồng nếu workload nhỏ.

\- Nhiều mã độc lập: có thể phân phối tính toán song song.

\- Module phụ thuộc kết quả nhau: thực thi theo dependency graph.

\- Số worker và chiến lược song song phải cấu hình được.



Không mặc định bật đa luồng nếu chi phí điều phối lớn hơn lợi ích.



\## 3. Benchmark



Benchmark tối thiểu:



\- Thời gian xử lý một mã.

\- Thời gian xử lý toàn bộ watchlist.

\- Batch so với incremental.

\- Mức sử dụng RAM.

\- Chi phí chuyển dữ liệu Node.js/Python ↔ Rust.

\- Hiệu quả xử lý tuần tự so với song song.



Sử dụng cùng bộ dữ liệu và môi trường kiểm thử, đo nhiều lần để xác định kết quả ổn định.



Không đặt mục tiêu tốc độ tuyệt đối trước khi có benchmark cơ sở.



\---



\# VII. KIẾN TRÚC REPOSITORY



Sử dụng Rust Cargo Workspace, phát triển trong repository độc lập.



Cấu trúc đề xuất:



```text

wyc-lib/

│

├── crates/

│   ├── contracts/

│   ├── indicators/

│   ├── market\_structure/

│   ├── price\_zones/

│   ├── volume\_engine/

│   ├── relative\_strength/

│   ├── wyckoff\_engine/

│   ├── evidence\_engine/

│   ├── feature\_engine/

│   └── wyc_lib/

│

├── bindings/

│   ├── node/

│   └── python/

│

├── tests/

│   ├── fixtures/

│   ├── golden/

│   ├── regression/

│   └── parity/

│

├── benchmarks/

├── examples/

├── docs/

├── .github/workflows/

│

├── Cargo.toml

├── README.md

├── ARCHITECTURE.md

├── CONTRACTS.md

├── IMPLEMENTATION\_PLAN.md

└── CHANGELOG.md

```



`wyc_lib` là crate điều phối và cung cấp public API thống nhất; các crate nghiệp vụ độc lập không phụ thuộc ngược vào nó.



Bindings sử dụng Rust Core thông qua API nội bộ và không tự triển khai lại logic phân tích.



\---



\# VIII. KIỂM THỬ VÀ ĐẢM BẢO CHẤT LƯỢNG



\## 1. Unit Tests



Kiểm thử riêng từng thuật toán, bao gồm các trường hợp biên.



\## 2. Golden Tests



Sử dụng bộ dữ liệu và kết quả chuẩn đã được xác minh để phát hiện sai lệch sau mỗi lần cập nhật.



\## 3. Cross-Language Parity Tests



Cùng một input phải cho kết quả tương đương giữa:



\- Rust.

\- Node.js.

\- Python.



Các sai khác số thực phải nằm trong dung sai được định nghĩa.



\## 4. Non-Repainting Tests



Chạy lại thuật toán theo từng nến đóng.



Kiểm tra:



\- Không có tín hiệu trước khi đủ bằng chứng.

\- Không gán ngược xác nhận.

\- Không sử dụng nến tương lai.

\- Không đọc dữ liệu chưa đóng của timeframe lớn.



\## 5. Regression Tests



Mọi thay đổi thuật toán cần kiểm thử ảnh hưởng tới kết quả cũ. Thay đổi có chủ đích phải được ghi nhận trong phiên bản và changelog.



\## 6. Property \& Invariant Tests



Kiểm thử các bất biến như:



\- Không trả về thời điểm xác nhận trước thời điểm dữ liệu khả dụng.

\- Không tạo giá trị chỉ báo hợp lệ trong giai đoạn warm-up chưa đủ dữ liệu.

\- Không thay đổi dữ liệu đầu vào.

\- Không phát sinh trạng thái không hợp lệ trong Wyckoff State Machine.



\---



\# IX. PHÂN PHỐI VÀ TRIỂN KHAI



\## 1. Repository



\- Private GitHub Repository.

\- Cargo Workspace.

\- Quản lý phiên bản theo Semantic Versioning.

\- CI chạy test trước khi phát hành.

\- Version tag gắn với bản build và changelog.



\## 2. Node.js Distribution



\- Đóng gói native addon bằng NAPI-RS.

\- Phát hành npm package vào registry riêng.

\- Tạo binary phù hợp với nền tảng mục tiêu.

\- Kiểm tra tương thích Node.js Runtime trên Vercel.



Native addon chỉ sử dụng trong môi trường Node.js tương thích; không mặc định chạy trên Edge Runtime hoặc browser.



\## 3. Python Distribution



\- Build wheel bằng Maturin.

\- Phát hành qua private Python package index hoặc GitHub Releases.

\- Build theo hệ điều hành, kiến trúc CPU và các cấu hình Python được hỗ trợ.



\## 4. CI/CD



GitHub Actions thực hiện:



1\. Rust lint và format check.

2\. Unit tests.

3\. Integration tests.

4\. Golden và parity tests.

5\. Build Node.js package.

6\. Build Python wheel.

7\. Kiểm tra artifact.

8\. Phát hành khi tạo release tag hợp lệ.



\## 5. Bảo vệ thuật toán



\- Không công khai repository chứa source.

\- Phân phối compiled binaries nếu cần chia sẻ.

\- Không nhúng secrets trong thư viện.

\- Có thể kiểm tra checksum và provenance của artifact.



Compiled binary giúp hạn chế lộ mã nguồn trực tiếp nhưng không bảo đảm chống reverse engineering tuyệt đối.



\---



\# X. PHẠM VI VÀ RANH GIỚI HỆ THỐNG



\## 1. wyc-lib chịu trách nhiệm



\- Tính toán chỉ báo.

\- Nhận diện cấu trúc.

\- Phân tích giá và khối lượng.

\- Phân tích Wyckoff.

\- Tổng hợp evidence.

\- Xuất feature.

\- Quản lý trạng thái thuật toán.

\- Cung cấp API và binding ổn định.



\## 2. Application chịu trách nhiệm



\- Kết nối SSI, vnstock và nguồn dữ liệu khác.

\- Làm sạch, đồng bộ và lưu dữ liệu.

\- Cache và database.

\- Giao diện biểu đồ.

\- Quản lý người dùng.

\- Chiến lược giao dịch cụ thể.

\- Quản trị rủi ro và danh mục.

\- Huấn luyện ML.

\- Backtesting và mô phỏng khớp lệnh.

\- Báo cáo, PDF và cảnh báo.



\*\*Nguyên tắc:\*\* Không đưa logic đặc thù của từng dự án vào thư viện lõi nếu logic đó không có giá trị tái sử dụng chung.



\---



\# XI. IMPLEMENTATION ROADMAP



| Phase | Phạm vi | Sản phẩm |

|---|---|---|

| P0 | Contracts, API, architecture, golden fixtures | Bộ đặc tả và nền tảng kiểm thử |

| P1 | Indicators, Market Structure | Rust Core nền tảng |

| P2 | Node.js và Python Bindings | Bản thư viện đa nền tảng đầu tiên |

| P3 | Price Zones, Volume/VSA, Relative Strength | Bộ phân tích giá và khối lượng |

| P4 | Wyckoff State Machine, Multi-Timeframe | Wyckoff Engine |

| P5 | Evidence Engine, DecisionReport | Hệ thống lập luận định lượng |

| P6 | Feature Engine, benchmark, optimization, CI/CD | wyc-lib V1.0 |



\## Nguyên tắc triển khai



\- Xây dựng và nghiệm thu tuần tự theo dependency.

\- Mỗi module phải có test trước khi tích hợp.

\- Bindings không được tự tính toán lại thuật toán.

\- Không mở rộng chức năng khi contracts chưa ổn định.

\- Không tối ưu hiệu năng nếu chưa có benchmark.

\- Không đưa một module vào production khi chưa đạt các tiêu chí kiểm thử liên quan.



\## Mốc phát hành



\*\*Foundation Release:\*\* Sau P2, có khả năng tái sử dụng Indicators và Market Structure trên Node.js/Python.



\*\*Analysis Release:\*\* Sau P4, có hệ thống phân tích giá, khối lượng và Wyckoff.



\*\*V1.0 Stable:\*\* Sau P6, hoàn tất các tiêu chí kiểm thử, tương thích và phân phối.



Các mốc trên là phạm vi phát hành, không mặc định thời gian thực hiện cố định.



\---



\# XII. TIÊU CHÍ NGHIỆM THU V1.0



wyc-lib V1.0 được nghiệm thu khi đáp ứng:



\- \[ ] Pure Rust Core chạy độc lập với framework.

\- \[ ] Các module có trách nhiệm và dependency rõ ràng.

\- \[ ] Có public API và versioned contracts.

\- \[ ] Node.js và Python sử dụng chung thuật toán.

\- \[ ] Kết quả cross-language đạt parity tests.

\- \[ ] Market Structure không sử dụng thông tin tương lai.

\- \[ ] Price Zones và Wyckoff Events có vòng đời xác định.

\- \[ ] Phân biệt observation, hypothesis và confirmation.

\- \[ ] DecisionReport truy vết được bằng chứng.

\- \[ ] Batch và incremental có kết quả tương đương.

\- \[ ] Feature API không gây data leakage.

\- \[ ] Có golden tests và regression tests.

\- \[ ] Có benchmark CPU, RAM, throughput.

\- \[ ] Build và phân phối được trên các nền tảng mục tiêu.

\- \[ ] Có tài liệu API, cấu hình và ví dụ tích hợp.

\- \[ ] Các kết quả có thể tái lập theo dữ liệu, cấu hình và phiên bản.



\---



\# XIII. ĐỊNH HƯỚNG PHÁT TRIỂN SAU V1.0



Các hướng mở rộng có thể bao gồm:



\*\*1. Advanced Market Structure\*\*

\- Phân tích cấu trúc đa cấp.

\- Market Regime Detection.

\- Structure Transition Models.



\*\*2. Advanced Wyckoff Intelligence\*\*

\- So sánh các giả thuyết Wyckoff.

\- Evidence Scoring đã hiệu chuẩn.

\- Phân tích mâu thuẫn giữa nhiều khung thời gian.



\*\*3. Advanced Feature Library\*\*

\- Đặc trưng cấu trúc và hành vi giá.

\- Feature Selection Support.

\- Hỗ trợ xuất dữ liệu định dạng cột.



\*\*4. WASM Distribution\*\*

\- Cho phép sử dụng một phần thuật toán trong browser.

\- Ưu tiên các module phù hợp về dung lượng và hiệu năng.



\*\*5. Extended Evidence Framework\*\*

\- Theo dõi diễn biến của từng giả thuyết.

\- Phân tích các tình huống bị vô hiệu.

\- Thống kê độ tin cậy của từng điều kiện dựa trên kết quả kiểm nghiệm.



Các nội dung này không thuộc phạm vi bắt buộc của V1.0.



\---



\# XIV. KẾT LUẬN



\*\*wyc-lib là thư viện thuật toán phân tích định lượng dùng chung, không phải một ứng dụng chứng khoán hoàn chỉnh.\*\*



Giá trị cốt lõi của hệ thống gồm:



1\. \*\*Consistency:\*\* Mọi ứng dụng sử dụng cùng một logic phân tích.

2\. \*\*Evidence:\*\* Kết quả dựa trên bằng chứng có thể kiểm tra.

3\. \*\*Reliability:\*\* Không repaint và không nhìn trước dữ liệu.

4\. \*\*Performance:\*\* Rust xử lý hiệu quả các khối lượng tính toán lớn.

5\. \*\*Reusability:\*\* Một thư viện phục vụ nhiều ứng dụng và nền tảng.

6\. \*\*Extensibility:\*\* Phát triển thêm thuật toán mà không phải thay đổi kiến trúc tổng thể.



\*\*Định hướng cuối cùng:\*\*



Xây dựng một nền tảng phân tích thị trường dựa trên những gì thực sự diễn ra trong giá, khối lượng và cấu trúc thị trường; cung cấp thông tin có căn cứ, giải thích được và kiểm chứng được, thay vì tạo ra những dự báo thiếu cơ sở.



\*\*wyc-lib V1.0 — One Analytical Core, Multiple Applications, Evidence-Driven by Design.\*\*

