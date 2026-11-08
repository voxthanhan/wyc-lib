# Phát triển và kiểm chứng P0–P1.7

## Ma trận nền tảng

| Thành phần | Chọn/mục tiêu | Tình trạng |
| --- | --- | --- |
| Rust | 1.85.1 pinned; MSRV 1.85; edition 2021 | Kết quả lần chạy trong báo cáo G0 |
| Windows | x86_64-pc-windows-msvc; Visual Studio Build Tools 2022 + Windows SDK cho local | Local/checkout sạch incremental, 93 Rust + 5 Python controls tại 45752f1; CI windows-latest PASS, ZIP payload verified |
| Linux | x86_64-unknown-linux-gnu; Ubuntu 24.04 CI | CI ubuntu-24.04 incremental PASS tại 45752f1, ZIP payload verified |
| macOS/ARM64 | Chưa chọn phiên bản/runner | Chưa công bố hỗ trợ |
| Node | 22.x, NAPI-RS ở P2 | Mục tiêu, chưa chọn bản patch/ABI hoặc kiểm chứng |
| Python | CPython 3.10–3.13, PyO3/Maturin ở P2 | Mục tiêu, chưa chọn bản patch/ABI hoặc kiểm chứng |
| Server | Linux x64 Node Runtime, gồm khảo sát Vercel P2 | Chưa kiểm chứng; browser/Edge/WASM ngoài phạm vi |

Dependencies trực tiếp khóa exact trong Cargo.toml: serde 1.0.219,
serde_json 1.0.140, sha2 0.10.9. `Cargo.lock` khóa bắc cầu. Không thêm vnstock
hoặc thư viện nguồn dữ liệu vào Core. Tests không cần keys, network hay market feed
sau khi dependency đã được tải. Chưa có Node/Python manifest hoặc build command.

## Chạy từ checkout có Rust

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python scripts/verify_oracles.py
```

Windows phải có linker/SDK MSVC. Rustup chọn toolchain từ rust-toolchain.toml.
Không cần cấu hình hệ thống nếu đã có Rust/Build Tools tương thích.
Lần triển khai P0 này dùng toolchain riêng dưới `.tools/` (đã gitignore); trong
PowerShell có thể đặt **cho tiến trình hiện tại**:

```powershell
$env:RUSTUP_HOME = "$PWD\.tools\rustup"
$env:CARGO_HOME = "$PWD\.tools\cargo"
$env:PATH = "$PWD\.tools\cargo\bin;$env:PATH"
cargo test --workspace --locked
```

`verify_oracles.py` dùng Python standard library, Fraction/Decimal; không import
wyc_lib hoặc tự ghi golden. Rust foundation harness kiểm checksum trước đọc,
round-trip/types/errors, replay availability, số/mask/schema, negative drift.
Golden cho các thuật toán chưa triển khai vẫn chỉ là oracle chuẩn bị;
không coi chúng là bằng chứng thuật toán đã chạy.
Từ P1.1, target `p1_foundation` chạy kiểm tra preparation/availability, result
mask/time và hash/ID runtime. Bộ `p1-identity-v1` được kiểm bằng `hashlib`/`struct`
độc lập trong cùng script oracle; không tạo hoặc tự ghi đè golden khi kiểm tra.

Từ P1.2, target `p1_rolling` của crate `indicators` kiểm sáu phép rolling,
golden từng cửa sổ, warm-up/period, overflow/underflow và replay causal qua
AvailableData. `p1-rolling-v1` có tám case tính tay, được script oracle kiểm
bằng Fraction/Decimal độc lập. Golden P0 cho rolling cũng được đối chiếu với
implementation; thời điểm nghiệm thu P1.2, các chỉ báo P1.3–P1.5 chưa được kiểm
bằng implementation. P1.3 được bổ sung ở đợt tiếp theo dưới đây.
Từ P1.3, target `p1_basic` kiểm chín chỉ báo nền và BasicIndicatorState:
golden mọi sample/status, batch–state mọi tiền tố, availability/future/revision,
identity và atomic failure/reset. `p1-basic-v1` có sáu case độc lập dùng Fraction
và smoothing dạng tổng hình học tại `scripts/basic_oracle.py`; script kiểm chỉ
đọc expected có sẵn, không lấy output Rust hoặc tự cập nhật golden.
Workspace verifier từ P1.5 yêu cầu đúng bốn package
contracts/indicators/market_structure/wyc-lib, dependency không vòng và đủ sáu
integration test targets, gồm p1_structure.

Từ P1.4, target `p1_advanced` chạy 12 ca kiểm 11 components RSI/MACD/ADX/Bollinger,
mọi sample/status/time của tám fixture, prefix có/không future, revision,
warm-up riêng, giá âm/zero/tiny/large, cấu hình lỗi và overflow/underflow.
Oracle Fraction/Decimal tại `scripts/advanced_oracle.py` dùng closed-form
smoothing và pairwise variance, khác đường tính Rust. Golden không tự cập nhật.
P1.4 chưa có mutable state; batch–incremental toàn danh mục thuộc P1.6/P1.7.
Kết quả local/CI mới nhất tại [báo cáo P1.4](acceptance/P1.4.md).
P1.4 đạt local và checkout sạch Windows với 54 tests; CI Windows/Linux tại
`4521da9`, run 42355665535 đều success và upload hai artifact. Source/test/config
CI đã đối chiếu với 51 SHA-256 bản local. Payload zip cần đăng nhập GitHub
để tải (thử API không đăng nhập trả 401); report phân biệt metadata đã xác
minh với payload chưa đọc. Không lấy local PASS thay cho bằng chứng Linux.

Từ P1.5, target `p1_structure` kiểm 15 fixture/golden đầy đủ pivot, labels,
trend, breaks, strength và evidence, cùng replay từng prefix và future thay đổi.
Oracle `scripts/structure_oracle.py` liệt kê cửa sổ trước rồi suy trend mỗi mốc
từ tập pivot khả dụng, strength Fraction và ID hashlib; không gọi Rust hoặc ghi
golden khi kiểm. Tests bổ sung hand-trace, delayed right/availability, revision,
dual/equal/plateau, reference one-shot/unknown/conflict, overflow/underflow,
input bất biến và toàn config/hash. Chi tiết local/CI tại
[báo cáo P1.5](acceptance/P1.5.md). State chung và benchmark ở P1.6/P1.7.
P1.5 đã đạt local/checkout sạch Windows tại 6cbc517 với 70 tests và 57
source fingerprints khớp. Sau khi người dùng cho phép rõ đưa source P1.5
lên repository công khai, CI run 42393366227 tại c7de17a đạt cả Windows/Linux,
hai artifact đã upload, 57 Git source blobs khớp bản local. Run/job/step và
artifact metadata đã lưu trong repo; thử tải hai ZIP không đăng nhập trả 401,
chưa đọc payload hoặc tự kiểm digest. P1.5 hoàn thành phạm vi nghiệm thu.

## Luồng bàn giao Git

Theo lựa chọn của người dùng, mặc định agent hoàn tất bàn giao trên `main`.
Nếu dùng nhánh riêng để triển khai/kiểm chứng, agent tự tích hợp phần đã đạt
vào `main` và đẩy lên GitHub; không để người dùng phải tự merge mỗi bước.
Trước khi tích hợp, cập nhật remote, bảo toàn lịch sử/thay đổi hiện có và kiểm
tra nội dung sau merge. Theo dõi CI của commit trên main đến kết quả thực tế.

## CI và nghiệm thu

Workflow `.github/workflows/ci.yml` chạy format/lint/build/test/oracle trên
Windows và Linux, dùng toolchain pinned và `--locked`; upload log cùng SHA commit.
CI không publish. Không ghi CI đã chạy chỉ vì có YAML hoặc local checks đạt.
Theo [ngoại lệ G0 ngày 07/11/2026](acceptance/G0.md), P1.1/P1.2 bắt đầu với
kiểm chứng local và hoãn CI. Yêu cầu P1.3 mở lại kiểm chứng CI; workflow đổi
tên hiển thị thành Rust foundation, giữ matrix và runner kiểm chung toàn
workspace. Kết quả đợt đó tại [báo cáo P1.3](acceptance/P1.3.md).
P1.3 đạt local Windows và checkout sạch (42 tests); sau khi người dùng cho phép
rõ đẩy mã lên repo công khai, CI run 42353272952 tại d86e6ff đạt cả Windows và
Ubuntu, hai artifact đã upload. Run/job/step và artifact metadata đã lưu trong
repo; payload artifact cần đăng nhập GitHub để tải, chưa được kiểm độc lập
digest local. Điều kiện CI hoãn của G0 đã đóng tại commit này. Lần run P0 cũ
giữ nguyên như bằng chứng lịch sử, không dùng để thay kết quả CI mới.
Local PASS chỉ chứng minh target Windows đã chạy, không thay bằng chứng Linux.
Báo cáo mỗi cổng theo `docs/acceptance/TEMPLATE.md`; commit chưa tồn tại phải
ghi chưa có, không bịa SHA. Log local P0 ở `docs/acceptance/logs/`.

Sau khi có commit, dùng cùng runner như CI trên checkout sạch:

```powershell
./scripts/check-foundation.ps1 -RequireClean
```

Thêm `-Offline` khi cache dependencies đã đầy đủ; `-ExpectedCommit <SHA>` ràng
buộc phép kiểm vào commit cụ thể. Runner cần PowerShell 7, Git, Rust và Python
trên PATH; đặt `CARGO_INCREMENTAL=0` chỉ trong tiến trình để tránh cache hard-link
trên filesystem không hỗ trợ. `ci-evidence/` được ignore và chứa summary.json,
log từng lệnh và metadata; summary ghi commit/tree, clean_before/clean_after,
exit code từng check và URL run GitHub nếu có. Kiểm thêm dependency boundary,
target foundation và liên kết Markdown bằng `scripts/verify_workspace.py`.

## P1.6 — API điều phối và state

Target `p1_orchestration` được buộc bởi workspace verifier cùng sáu target cũ.
Nó đối chiếu golden độc lập có sẵn qua API mới, replay indicators/structure/cả
hai theo nến, validation/availability/revision, toàn identity/metadata/cursor,
module selection và atomic errors/reset. Fixtures/golden cũ giữ nguyên bytes.
API và protocol tại [ADR-0003](adr/0003-p1-orchestration-state.md),
[báo cáo P1.6](acceptance/P1.6.md). Dùng runner chung phía trên cho local/CI;
không cần Node/Python bindings hoặc dữ liệu online. State tính lại batch,
không là baseline hiệu năng hoặc nghiệm thu G1; P1.7 vẫn còn mở.

P1.6 đạt local/checkout sạch và CI Windows/Linux tại `4460753`, run 42395906710,
85 tests và 60 source fingerprints khớp. Hai artifact upload thành công; tải
ZIP không đăng nhập trả 401. Giữ tách biệt metadata đã kiểm với payload chưa
đọc; chi tiết/bàn giao main trong báo cáo P1.6 ở trên.

P1.6 đã tích hợp main tại `5af2383`, tree khớp bản nghiệm thu; CI main run
42396385077 đạt cả Windows/Linux. Commit bổ sung báo cáo sau run chỉ thay
tài liệu/log, không thay code đã kiểm chứng.

## P1.7 — Replay, baseline và G1

Runner chung hiện buộc tám integration targets, thêm p1_replay và hai examples.
Sau toàn bộ tests, runner chạy p1_walkthrough (input → append → revision/reset →
JSON), build release p1_baseline rồi dùng measure_baseline.py đo riêng binary.
Trace mỗi prefix tại `ci-evidence/replay-traces.json`; raw/tổng hợp baseline,
CPU/time/RSS và scope phép đo ở `ci-evidence/baseline.json`. CI hai OS upload
cùng artifact log đã có, không dùng giới hạn timing làm điều kiện pass/fail.

Chạy từ checkout sạch bằng lệnh runner phía trên. Có thể chỉ chạy:

```text
cargo test --locked --package wyc-lib --test p1_replay
cargo run --locked --package wyc-lib --example p1_walkthrough
cargo build --release --locked --package wyc-lib --example p1_baseline
```

[Protocol baseline](../benchmarks/README.md) ghi chính xác workload/setup/timer
và giới hạn RAM. Không so tốc độ Windows/Linux trên runner khác nhau như một
thử nghiệm cùng môi trường. Không có state O(1), FFI hoặc bindings ở P1.7.
[Báo cáo G1](acceptance/G1.md) ghi kết quả và commit thực tế; báo cáo cũ là
bằng chứng lịch sử tại từng phase.

P1.7/G1 đã đạt local, checkout sạch Windows và CI Windows/Linux tại `7c57220`,
run 42398202869; cả hai job kiểm chứng/upload success. REST API hết hạn mức đọc
ẩn danh, nên bằng chứng CI dùng trang run/job công khai với từng step, đúng SHA
và artifact display metadata. Chưa tải payload ZIP; không coi digest hiển thị
là digest đã tự kiểm. Chi tiết và tích hợp main tại báo cáo G1.

Main đã tích hợp P1.7/G1 tại `223d742`; CI main run 42399120154 đạt Windows/Linux
với cùng runner, mỗi job chín steps success. Tree khớp nhánh nghiệm thu và 65
source blobs không đổi; bổ sung báo cáo sau run chỉ thay tài liệu/log.

## Incremental và xác minh ZIP sau G1

Runner hiện buộc thêm target p1_incremental (chín integration targets tổng cộng),
chạy năm negative controls của artifact verifier. Công cụ verify_ci_artifacts.py
nhận CI evidence JSON có head_sha/run/artifact names/digest và output directory;
chỉ tải public ZIP, kiểm digest trước unzip và không đọc credentials. Ví dụ:

```text
python scripts/verify_ci_artifacts.py docs/acceptance/logs/ci-223d742-run-42399120154.json .tools/g1-artifacts/42399120154 --local-trace docs/acceptance/logs/p1.7-windows-clean/replay-traces.json
```

Đường nightly.link chỉ vận chuyển bytes của repo công khai, không cài app;
GitHub published digest cùng summary SHA/run là bằng chứng. Không ghi signed
redirect URL. Test verifier dùng synthetic local ZIP, không phụ thuộc network.
G1 mới/benchmark trước-sau và các ZIP thực tế tại [báo cáo bổ sung](acceptance/G1-incremental.md).
