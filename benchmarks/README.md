# Baseline Rust P1.7

`p1-baseline-v1` đo bản tham chiếu hiện tại, không đổi thuật toán và không đặt
ngưỡng thời gian làm điều kiện test. Dữ liệu `synthetic-triangle-v1` được tạo
xác định trong [p1_baseline.rs](p1_baseline.rs): 256/1024 nến, chu kỳ tam giác
32 nến, close=100+min(i%32,32-i%32), high/low=close±1, open=close,
volume=(i%17)*100; timestamp=(i+1)*1000, availability=timestamp+200, revision
baseline-r1. Có pivot strict, equal labels và volume 0; đây không phải dữ liệu
thị trường đại diện mọi phân bố hoặc benchmark nến có nhiều BOS/CHoCH.

Cấu hình explicit `AnalysisConfig::default()` (period=14, MACD=12/26/9,
multiplier=2, L/R=2); JSON ghi đầy đủ config/hash/version. Một luồng Rust,
profile release mặc định, toolchain pinned 1.85.1, không dependency benchmark mới.
Mỗi workload có một warm-up rồi năm mẫu, báo raw/min/median/max và throughput:

- `batch_indicators`, `batch_structure`, `batch_both`: input chuẩn bị ngoài timer;
  gồm validation, tính toán, output allocation, tiêu thụ và drop output.
- `append_one_both`: state N-1 chuẩn bị/clone ngoài timer, đo một update trả
  owned result; gồm validation/copy/tính lại toàn batch và thay/drop state cũ,
  chưa gồm tiêu thụ/drop bản result trả về. N là tổng history sau append.
- `replay_both`: từ rỗng đến N, gồm new, tạo updates, toàn bộ tính lại batch,
  output/drop và clone result cuối; không được so với một lần batch như cùng
  lượng công việc. Replay này đánh giá N prefix, batch chỉ đánh giá prefix cuối.
- `watchlist_8_sequential`: tám symbol độc lập cùng dữ liệu, chạy tuần tự;
  input/đổi tên symbol đã chuẩn bị ngoài timer, gồm tiêu thụ/drop output.

```text
cargo build --release --locked --package wyc-lib --example p1_baseline
python scripts/measure_baseline.py target/release/examples/p1_baseline.exe ci-evidence/baseline.json
```

Linux bỏ `.exe`. Runner kiểm chứng tự build/chạy cùng phép đo trên local và CI.
Binary Rust chỉ xuất JSON, wrapper Python đo CPU và peak RSS của riêng process
binary, loại thời gian/RAM compiler. CPU process gồm setup/warmup/serialization,
không phải CPU từng operation. Windows lấy PeakWorkingSetSize mỗi 10 ms (có thể
bỏ lỡ phần cuối); Linux dùng getrusage ru_maxrss của một child. RSS gồm runtime,
allocator, input, history và output, không phải kích thước heap/state riêng.
Chưa đo allocation count, RAM từng operation, FFI (P2), watchlist thật, SIMD hoặc
song song. Runner CI chia sẻ máy; không suy chênh tốc độ giữa OS từ các mẫu này.
Thời gian hệ thống chỉ dùng trong công cụ đo bên ngoài Core.

Kết quả gắn commit và môi trường ở [báo cáo G1](../docs/acceptance/G1.md).

## So sánh incremental sau G1

Giữ nguyên generator/workloads/config/profile/warmup/năm samples của baseline.
Chạy binary bản 64fa87f trước sửa và binary bản mới trên cùng máy/toolchain;
lưu cả raw samples, CPU/RSS và phạm vi đo, không đổi workload để lấy speedup.
Append vẫn trả full owned report, nên số đo gồm chi phí copy O(N); seed/reset
và batch-only workloads được báo riêng, không suy hiệu năng mọi input từ tam giác.
BasicIndicatorState không có workload riêng trong baseline này; không công bố
speedup API đó từ số đo AnalysisState. Kết quả tại
[G1 bổ sung](../docs/acceptance/G1-incremental.md).
