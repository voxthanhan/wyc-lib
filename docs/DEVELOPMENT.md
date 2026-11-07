# Phát triển và kiểm chứng P0

## Ma trận nền tảng

| Thành phần | Chọn/mục tiêu | Tình trạng |
| --- | --- | --- |
| Rust | 1.85.1 pinned; MSRV 1.85; edition 2021 | Kết quả lần chạy trong báo cáo G0 |
| Windows | x86_64-pc-windows-msvc; Visual Studio Build Tools 2022 + Windows SDK | Nền tảng local ban đầu |
| Linux | x86_64-unknown-linux-gnu; Ubuntu 24.04 CI | Mục tiêu CI, cần log từ commit |
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
Golden P1 là oracle chuẩn bị, chưa phải test algorithm implementation.

## CI và nghiệm thu

Workflow `.github/workflows/ci.yml` chạy format/lint/build/test/oracle trên
Windows và Linux, dùng toolchain pinned và `--locked`; upload log cùng SHA commit.
CI không publish. Không ghi CI đã chạy chỉ vì có YAML hoặc local checks đạt.
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
