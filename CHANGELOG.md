# Changelog

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
- Local/checkout sạch Windows tại 6cbc517 đạt 70 tests; CI chưa chạy vì push
  công khai bị kiểm duyệt tự động từ chối, cần cho phép rõ của người dùng.

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
