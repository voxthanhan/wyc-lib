# ADR-0008 — P2.5 artifact và consumer server

Ngày 16/11/2026. Quyết định P2.5; nghiệm thu thực chạy tại
[P2.5](../acceptance/P2.5.md). Giữ nguyên Core, API bindings, schema, config/hash,
golden và ba Cargo.lock; không publish npm/PyPI hoặc tạo release/tag.

## Ma trận và đóng gói

Chỉ nhận Windows x64 MSVC và Ubuntu 24.04 x64 GNU; Rust 1.85.1,
Node 22.22.0/Node-API v8, CPython 3.10.9/PyO3 0.23.5/Maturin 1.8.3.
Hai build npm có cùng tên/version nhưng binary khác OS; stage ghi `os` và
`cpu: ["x64"]`, cài `--engine-strict`. Không phải installer đa nền tảng.
Wheel phải đúng `cp310-cp310-win_amd64` hoặc `cp310-cp310-linux_x86_64`;
Linux native tag, không claim manylinux/abi3. Không mở thêm runtime/OS/CPU.

Audit allowlist thành viên archive, metadata/version/target, header PE/ELF x64;
wheel kiểm CRC và mọi digest/size RECORD. Tar không được chứa link, source hoặc
tệp ngoài allowlist. Lưu artifact manifest SHA-256/size/target/commit. Năm controls
kiểm archive hợp lệ và từ chối tệp thừa/path traversal, sai target/binary, RECORD
bị sửa. Kiểm native load thực tế sau cài vẫn bắt buộc, header không đủ chứng minh ABI.

## Consumer và server

Runner tạo thư mục mới ngoài source, cài tarball offline không script và wheel
vào venv mới bằng `--no-index --no-deps`; ghi vị trí module đã nạp trong đúng
node_modules/venv. Ví dụ Python dùng `-I`, không phụ thuộc source/PYTHONPATH.
Ví dụ Node/Python gọi cả batch, hai single-module API và AnalysisState, đối chiếu
direct Rust reference. Request ví dụ sao từ fixture arithmetic/config golden P1;
không có kỳ vọng tự sinh bằng bindings.

Node HTTP server và Next.js production `build --webpack` → `start` nhận cùng
request; so đầy đủ report với Core, kiểm invalid shape/malformed JSON/missing
data/negative volume và recovery. Bind loopback, timeout readiness, dừng process
trong finally. Next.js 16.4.6, React/React DOM 19.3.1 exact và npm lockfile;
native addon external qua `serverExternalPackages`, Route Handler `runtime = 'nodejs'`.
Không import addon ở client/Edge. CI hai OS chạy cùng gates cũ và mới, lưu logs,
manifest, tarball và wheel; native timeout tăng 20 → 30 phút cho production build.

Tham chiếu upstream: [Next.js 16.4.6](https://github.com/vercel/next.js/releases/tag/v16.4.6)
và [serverExternalPackages](https://nextjs.org/docs/app/api-reference/config/next-config-js/serverExternalPackages).
Phiên bản React 19.3.1 đã xác minh qua npm metadata ngày 16/11/2026; lockfile
ghi phiên bản/integrity của toàn cây dependency. Không mặc định hỗ trợ phiên bản khác.

## Giới hạn

Đây là smoke self-hosted production trên runner, không phải triển khai Vercel,
serverless, container portability, standalone output hoặc kiểm tải. Ví dụ dùng
dữ liệu synthetic và không có auth/service thị trường. Lời gọi vẫn đồng bộ,
owned-copy; không đổi GIL/worker/lifetime/state. Baseline FFI/copy/batch thuộc
P2.6; toàn G2/Foundation còn mở sau P2.5.
