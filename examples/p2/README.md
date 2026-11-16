# Ví dụ consumer P2.5

Copy thư mục này ra ngoài repository, cài artifact đúng OS/CPU/runtime trong
[ma trận](../../docs/adr/0008-p2-package-consumers.md). Không chạy trực tiếp từ
source tree khi nghiệm thu. Không cần Rust hoặc mã nguồn để nạp package đã cài.

`request.json` dùng fixture arithmetic P1 và cấu hình golden đã có, sáu nến đóng
synthetic. Node/Python examples in batch, indicators, structure và state result;
không kết nối dữ liệu thị trường. `node.cjs` đóng state trong finally;
`python.py` dùng context manager.

## Node 22.22.0

Trong bản copy, tạo package.json có `private: true`, rồi chạy:

```text
npm install --engine-strict --ignore-scripts /absolute/path/wyc-lib-0.1.0.tgz
node node.cjs
node node-server.cjs
```

Server bind 127.0.0.1:3000 (đổi bằng PORT), nhận POST `/analyze` với JSON request.
Đây là smoke app nhỏ, cần bổ sung auth/giới hạn tài nguyên theo ứng dụng thực tế.

## CPython 3.10.9

Tạo venv mới, dùng interpreter của venv cho cả pip và ví dụ:

```text
python -m venv consumer-venv
<venv-python> -m pip install --no-index --no-deps /absolute/path/wyc_lib-0.1.0-cp310-cp310-<platform>.whl
<venv-python> -I python.py
```

Platform là win_amd64 hoặc linux_x86_64. Plain API không cần NumPy/Pandas/Polars.
Các adapter extras và type/parity regression vẫn được runner kiểm riêng.

## Next.js server production

Trong `next-server` của bản copy:

```text
npm ci --ignore-scripts
npm install --no-save --engine-strict --ignore-scripts /absolute/path/wyc-lib-0.1.0.tgz
npm run build
npm start -- --hostname 127.0.0.1
```

POST `/api/analyze` nhận cùng JSON. Route Handler dùng Node runtime và external
package; native binary phải được cài trên đúng target OS. Không import addon
trong Client Component/browser/Edge. Next 16.4.6 + React/React DOM 19.3.1 được
khóa trong package-lock. Smoke dùng `next build --webpack` và `next start`,
không chứng minh Vercel/serverless/standalone hoặc nền tảng khác.

`python scripts/check_p2.py` tự copy examples, cài artifact, chạy hai ví dụ và
hai HTTP server, đối chiếu report/error/recovery với direct Rust reference;
log tại ci-evidence/p2. Chưa publish package; G2/Foundation còn chờ P2.6.
