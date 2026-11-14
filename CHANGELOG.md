# Changelog

## Quy trình Git/CI — 14/11/2026

- Lưu ủy quyền thường trực của người dùng trong AGENTS.md cho commit/push/PR,
  CI và tự gộp main sau khi đúng HEAD đạt kiểm tra, trong phạm vi tác vụ đã giao.
- Core/native CI nhận `work/**` và PR vào main; concurrency hủy run cũ trên
  nhánh/PR, giữ run main. Không cần liệt kê riêng từng nhánh phase.
- Native CI không tự bootstrap khi thiếu lock; quyền workflow vẫn contents: read.

## 0.1.0 — P2.3 Python P1, chưa phát hành

- Thêm ba dict APIs và AnalysisState/context manager gọi Rust qua PyO3; giữ JSON pilot.
- Copy-owned list/tuple/NumPy và converters Pandas/Polars; không triển khai lại thuật toán.
- Schema PEP 561/TypedDict cho P1, tagged missing/warmup và class/code/path errors.
- CI kiểm wheel ngoài source, plain Python không extras, NumPy/Pandas/Polars,
  mypy strict, ownership/append/reset/close và native JSON regression Windows/Linux.
- Core/golden/schema/config/hash/native lockfiles giữ nguyên; G2 còn mở.
- P2.3 đạt tại e4424af: CI Core/native Windows/Linux PASS, 72 plain và 79 extras
  Python assertions/OS, 67 host assertions/OS, mypy strict và Node/JSON regression.
  97 source blobs khớp local; --locked/checkout sạch. Chưa tích hợp main; bàn giao PR.
- Kết quả thực tế tại [báo cáo P2.3](docs/acceptance/P2.3.md).

## 0.1.0 — P2.2 Node.js P1, chưa phát hành

- Thêm ba object APIs, AnalysisState và khai báo TypeScript đầy đủ P1; giữ JSON pilot.
- Copy-owned array/Float64Array, chặn input host gây mất dữ liệu trước JSON; Rust giữ mọi phép tính.
- Local 102 Rust tests/5 controls/24 host assertions; CI Windows/Linux typed Node
  83 assertions, TypeScript strict, native package và JSON smoke PASS tại 5946286.
- Đã tích hợp main; CI main Core/native đều PASS Windows/Linux. Native local chưa xác minh, G2 còn mở.
- Bằng chứng và giới hạn tại [báo cáo P2.2](docs/acceptance/P2.2.md).

## 0.1.0 - P2.1 native JSON pilot, chưa phát hành

- Chốt FFI v1: f64/safe integer time, tagged numeric status, config/result,
  code/path exception, owned handle và close/GC tại ADR-0005.
- Thêm shared Rust JSON transport và state wrapper chỉ gọi public Core API;
  giữ schema/config/hash/algorithm và golden P1. Native Node/Python nằm trong
  workspace riêng, không thêm runtime vào dependency graph Core.
- Thêm adapter pilot NAPI-RS/PyO3, npm/wheel manifests, 40-action smoke và
  workflow hai OS. NAPI-RS/PyO3 build, npm/wheel cài ngoài source tree và smoke
  đã chạy thành công trên Windows/Linux; native lockfiles khớp giữa hai OS.
  CI nghiệm thu bắt buộc --locked và checkout sạch, ghi checksum/summary.
  Không đổi mã thuật toán Core hoặc golden; G2/Foundation chưa phát hành.
- Kết quả local, log và các điều kiện còn thiếu tại
  [báo cáo P2.1](docs/acceptance/P2.1.md).
- P2.1 đạt tại cf6c464 và đã tích hợp main: local 101 Rust tests + 5 controls,
  1008 replay traces giữ nguyên; 86 source blobs local/CI khớp. CI main Core
  43578612827 và native 43578612862 đạt Windows/Linux, --locked/clean checkout,
  40 actions/runtime/OS, 14 Rust references, 20 error cases. Native local và
  kiểm ZIP độc lập chưa xác minh; typed adapters/G2 còn mở.

## 0.1.0 — Incremental sau G1, chưa phát hành

- AnalysisState/BasicIndicatorState append tính delta từ cache, shared numerical
  kernels và structure step; không tính lại history trên đường thành công.
- Shared row validation, atomic cache/delta và batch error-priority fallback
  trên đường lỗi. Giữ API/schema/config/hash/p1-v1/golden và dependency.
- Kiểm ZIP CI độc lập bằng digest/CRC, SHA/clean/checks/tests/traces/baseline;
  thêm long/seed/rollback/revision tests và verifier negative controls.
- Local/checkout sạch và CI Windows/Linux tại 45752f1 đạt 93 Rust tests +
  5 Python controls; ZIP hai OS kiểm digest và payload độc lập, 1008 trace hashes
  giữ nguyên. Baseline local N=1024 append khoảng 10×, replay khoảng 48× trong
  cặp đo tham chiếu; full output vẫn O(N), batch-only không được coi cải thiện.
  Đã tích hợp main tại 507fdad; CI main 42436169902 đạt hai OS, cả tám ZIP
  G1 cũ/mới đã kiểm độc lập, source/output hashes giữ nguyên.
  Số đo/giới hạn/tích hợp tại [G1 bổ sung](docs/acceptance/G1-incremental.md).

## 0.1.0 — P1.7, chưa phát hành

- Thêm regression replay delayed/grouped availability trên toàn fixtures
  structure qua ba lựa chọn module; reset revision tại nửa history rồi append,
  first-valid lookback 1/2/3/8 và closed-form ramp độc lập.
- Thêm ví dụ Rust p1_walkthrough và baseline release p1_baseline; runner
  local/CI xuất prefix traces, năm mẫu timing/workload, CPU và peak RSS process.
- Không đổi Core, công thức/seed/schema/config/hash, dependency hoặc golden.
  State vẫn tính lại batch; không công bố tối ưu hay O(1).
- G1 đạt local/checkout sạch Windows và CI Windows/Linux tại 7c57220, 88 tests,
  run 42398202869. Có 1008 prefix traces và 65 source fingerprints khớp; bằng
  chứng CI HTML/giới hạn artifact tại [báo cáo G1](docs/acceptance/G1.md).
- Đã tích hợp main tại 223d742; CI main run 42399120154 đạt Windows/Linux,
  tree khớp bản nghiệm thu và 65 source fingerprints không đổi.

## 0.1.0 — P1.6, chưa phát hành

- Thêm calculate_indicators (25 series), analyze_structure, analyze_batch và
  AnalysisState với new/result/identity/history_len/update_closed_bar/reset.
- Thêm DTO ClosedBar/ClosedBarUpdate và protocol append identity/cursor,
  metadata/revision/availability; update/reset atomic, chỉ giữ prefix khả dụng.
- API đơn module dùng toàn AnalysisConfig thay chữ ký draft để giữ hash nhất
  quán; giữ API engine cũ, công thức/seed/schema/config/hash và dependency.
- Golden cũ được kiểm qua API mới; kiểm module selection bằng lỗi riêng từng
  engine, replay mọi prefix, stale cursor, revision/reset và input bất biến.
- Local/checkout sạch và CI Windows/Linux tại 4460753 đều đạt (85 tests),
  run 42395906710, hai artifact upload, 60 source fingerprints khớp.
  Bằng chứng tại [báo cáo P1.6](docs/acceptance/P1.6.md).
- Đã tích hợp main tại 5af2383; CI main run 42396385077 đạt Windows/Linux,
  tree khớp bản nghiệm thu và source bytes không đổi.
  State tính lại batch, baseline và nghiệm thu toàn G1 còn chờ P1.7.

## 0.1.0 — P1.5, chưa phát hành

- Thêm crate market_structure và calculate_structure: pivot strict xác nhận,
  HH/HL/LH/LL/EH/EL, trend, close-confirmed BOS/CHoCH, strength và evidence.
- Quy định ID/rule/ordinal, missing strength, precedence và lỗi xung đột;
  giữ schema/config/hash p1-v1, không sửa contracts hoặc công thức đã nghiệm thu.
  Cargo.lock chỉ thêm package/cạnh dependency nội bộ, không thêm thư viện ngoài.
- Thêm 15 golden độc lập, oracle cửa sổ/tập pivot, Fraction và hashlib; kiểm
  đầy đủ output, replay mọi prefix/future/revision, ca biên và input bất biến.
- CI verifier bắt buộc bốn packages/sáu targets. Kết quả local/CI và giới hạn
  tại [báo cáo P1.5](docs/acceptance/P1.5.md). P1.6/P1.7 và G1 còn mở.
- Local/checkout sạch Windows tại 6cbc517 đạt 70 tests; sau khi người dùng cho
  phép rõ push source P1.5 công khai, CI Windows/Linux tại c7de17a đều success,
  run 42393366227, hai artifact đã upload. P1.5 hoàn thành phạm vi đã giao.

## 0.1.0 — P1.4, chưa phát hành

- Thêm RSI Wilder, MACD line/signal/histogram, DI/DX/ADX và Bollinger population;
  API `calculate_advanced` xuất đúng 11 series với metadata và warm-up riêng.
- Tái sử dụng smoothing/TR/mean/std P1.2/P1.3; không đổi seed/rule/schema/config/hash
  hoặc thêm dependency. BasicIndicatorState vẫn chỉ gồm chín chỉ báo nền.
- Thêm tám golden độc lập Fraction/Decimal, closed-form smoothing và variance
  từ khoảng cách cặp; kiểm mỗi sample, prefix/revision/availability và biên số.
- Làm rõ giới hạn làm tròn bands gần triệt tiêu ở thang 1e100; giữ nguyên golden,
  không clip giá trị. Ngân sách số và giới hạn trong catalog P1.
- Kiểm chứng local/CI và phạm vi nghiệm thu tại [báo cáo P1.4](docs/acceptance/P1.4.md).
  Local/checkout sạch đạt 54 tests; CI Windows/Linux đều success tại `4521da9`,
  run 42355665535, hai artifact đã upload. P1.4 hoàn thành phạm vi được giao.
  Chưa nghiệm thu P1.5–P1.7/G1; không publish/tag hoặc đổi quyền repository.

## 0.1.0 — P1.3, chưa phát hành

- Thêm chín chỉ báo nền, seed SMA/smoothing đã chốt, TR đầu tiên H-L,
  missing cho ROC/RVOL mẫu số 0, cùng API Rust `calculate_basic` có metadata/time.
- Thêm BasicIndicatorState nhận snapshot tích lũy, kiểm identity/history,
  update/reset atomic và tính lại batch. AnalysisState toàn P1 vẫn thuộc P1.6.
- Sáu golden độc lập dùng Fraction và smoothing dạng tổng hình học; kiểm
  mọi tiền tố batch–state, tương lai thay đổi, warm-up, biên số và revision.
- Không thêm dependency hoặc đổi schema/config/hash/seed; rolling chỉ mở
  helpers nội bộ để tái sử dụng mean/tổng bù. Không publish/tag hoặc đổi quyền repo.
- Mở lại kiểm chứng CI theo yêu cầu; bằng chứng tại [báo cáo P1.3](docs/acceptance/P1.3.md).
- Local/checkout sạch đạt 42 tests; CI windows-latest/ubuntu-24.04 đều PASS
  tại d86e6ff (run 42353272952), hai artifact log đã upload. Đóng điều kiện
  CI hoãn của G0; G1 còn mở cho P1.4–P1.7.

## 0.1.0 — P1.2, chưa phát hành

- Thêm crate indicators và sáu rolling primitives sum/mean/min/max/variance/std;
  re-export qua wyc_lib, không có analytical entry points/state.
- Tổng bù Neumaier, variance dân số hai lượt, cửa sổ inclusive tính lại độc
  lập; warm-up rõ ràng và input bất biến. Công thức/seed/config/hash không đổi.
- Thêm AnalysisError và mã numeric_overflow, phân biệt lỗi số trung gian
  với non_finite của input; giữ nguyên API ContractError đã có.
- Thêm golden tính tay cho tám chuỗi, oracle Fraction/Decimal, kiểm biên số,
  cửa sổ, causal replay/revision và cập nhật dependency verifier ba crate.
- Chỉ hoãn CI theo yêu cầu; kiểm chứng local và phạm vi nghiệm thu tại
  [báo cáo P1.2](docs/acceptance/P1.2.md). Chưa nghiệm thu G1 hoặc P1.3–P1.7.

## 0.1.0 — P1.1, chưa phát hành

- Ghi ngoại lệ G0 ngày 07/11/2026: tạm hoãn CI, cho phép bắt đầu P1 dựa trên
  kiểm chứng local; chưa đóng đầy đủ G0 hoặc nghiệm thu Linux.
- Thêm validated AvailableData, numeric series/status/time checks và
  prepare_request với module order/metadata xác định; không sửa input.
- Thực thi canonical config-v1 hash và event-v1 ID theo đặc tả P0 bằng sha2
  hiện có; thêm vectors SHA-256 kiểm chứng độc lập bằng Python standard library.
- Wire schema/config/rule không đổi; chưa có thuật toán, state hoặc bindings.

## 0.1.0 — P0, chưa phát hành

- Khởi tạo workspace contracts/wyc-lib, toolchain pinned và CI kiểm tra nền.
- Contract 0.1.0 bổ sung available_at, revision_available_at, snapshot_kind,
  đơn vị dữ liệu, gaps và FiniteF64/status có thể chuyển JSON rõ ràng.
- Đặc tả API/state, công thức P1, temporal fixtures và golden tính độc lập.
- Chưa có analytical engine hoặc bindings. Tình trạng nghiệm thu tại báo cáo G0.


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
