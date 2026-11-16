# Foundation 0.1.0 — ứng viên P2.6

Ngày 17/11/2026. **G2 đã đạt** tại `36978aa`, main `4619e4d`; Core/native CI
Windows/Linux PASS và bốn ZIP push/PR/main đã kiểm độc lập. [Báo cáo P2.6](acceptance/P2.6.md)
ghi source SHA, run và SHA-256 từng npm/wheel. Candidate định danh bằng các dấu
vết này, không phải tag. Chưa publish npm/PyPI; Foundation bao gồm P1/P2,
V1.0 và P3–P6 chưa triển khai.

## Cài đặt và sử dụng

Lấy tarball/wheel từ native Actions artifact của đúng source SHA đã nghiệm thu,
đối chiếu artifact-manifest.json, SHA-256 và target trước cài. [Ví dụ](../examples/p2/README.md)
có lệnh cài offline, venv và Node/Next.js production. Consumer không cần Rust.
Không cài tarball/wheel của OS khác. Hướng dẫn và DTO/state/error tại [API](API.md).

Rust cung cấp calculate_indicators, analyze_structure, analyze_batch và
AnalysisState/update_closed_bar/reset. Node nhận plain arrays/Float64Array;
Python nhận list/tuple/NumPy và có converters Pandas/Polars. Cả hai dùng cùng
Core; JSON pilot giữ tương thích. Đầu vào owned-copy, output có mask/status,
timestamps UTC milliseconds safe integer; missing và warm-up khác nhau.
Ứng dụng chuẩn hóa dữ liệu/revision/availability; Core từ chối input lỗi.

Gọi analyze_batch khi cần cả indicators và structure trên cùng snapshot để
giảm số lần qua adapter. Đây không phải batch nhiều symbol. Dùng state cho một
nến đóng kế tiếp, echo identity/cursor; lịch sử sửa hoặc gap cần reset/replay.
Đóng handle bằng close/finally hoặc Python context manager; không chia cùng
state cho các lời gọi đồng thời. Chi phí còn gồm clone toàn output/history.

Node chạy đồng bộ trên Node Runtime; Python giữ GIL. Ứng dụng chịu trách nhiệm
điều phối công việc dài và giới hạn tài nguyên. Next Route Handler đặt runtime
nodejs và serverExternalPackages=['wyc-lib']; không import trong browser/Edge.

## Compatibility

| Thành phần | Target đã nghiệm thu tại main 4619e4d |
| --- | --- |
| Windows | x64 MSVC, windows-latest |
| Linux | Ubuntu 24.04 x64 GNU, wheel linux_x86_64 |
| Rust/build | 1.85.1, release native, ba Cargo.lock, --locked |
| Node | 22.22.0, NAPI-RS 2.16.17, Node-API v8, tarball riêng OS |
| Python | CPython 3.10.9, PyO3 0.23.5, Maturin 1.8.3, cp310 wheel |
| Extras | NumPy 2.2.6, Pandas 2.2.3, Polars 1.30.0 |
| Server | Node HTTP; Next 16.4.6/React 19.3.1 build --webpack + start |
| Versions | package/schema 0.1.0; config p1-v1; binding p2-ffi-v1 |

macOS/ARM/musl, manylinux/abi3, runtime khác, browser/WASM, Vercel/serverless/
standalone chưa nghiệm thu. Không suy kết quả smoke thành load/concurrency hoặc
leak-free. Không có cam kết binary bytes tái lập tuyệt đối giữa các builds.

## Baseline và rollback

CPU/RSS Python lấy từ worker thực trước final JSON encoding; wall từ parent
gồm output, và parent_process_observation được giữ riêng để phân biệt Windows
venv launcher. Rust/Node đo process tới khi kết thúc. Không suy các chỉ số này
thành heap/state size hoặc bằng chứng leak-free.
[ADR-0009](adr/0009-p2-ffi-baseline.md) định nghĩa scope từng tầng và phép đo.
Native CI lưu baseline-input.json, baseline-report.json, raw outputs và năm
samples mỗi operation, CPU/wall/RSS; không dùng tốc độ làm pass/fail. Không
cộng/trừ các tầng để suy chi phí FFI thuần hoặc dùng RSS để kết luận leak.
[P2.6](acceptance/P2.6.md) và [G2](acceptance/G2.md) ghi kết quả cuối cùng.

Giữ artifact/checksum cũ để cài lại đúng OS/runtime. State chỉ ở RAM, không có
định dạng serialized cache tương thích để chuyển giữa engine/config/revision.
Sau đổi artifact hoặc dữ liệu, tạo state mới và replay snapshot đã chuẩn hóa
do ứng dụng lưu; không tái dùng handle cũ hoặc tự sửa golden để chấp nhận kết quả.
