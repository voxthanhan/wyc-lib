# Quy trình Git/CI tự động — 14/11/2026

Người dùng đồng ý gộp PR #3 và yêu cầu các lần sau chạy tự động, không hỏi lại.
[AGENTS.md, mục 12](../../AGENTS.md) lưu nguyên văn ủy quyền, phạm vi repository
`voxthanhan/wyc-lib` và điều kiện gộp. Agent được tự thực hiện commit/push/PR,
kiểm CI, sửa lỗi trong phạm vi và merge main khi đúng HEAD đạt mọi checks.

## Thay đổi

- Hai workflow Core/native nhận push `main`, `work/**`, PR vào `main` và dispatch.
- Concurrency tách workflow/event/ref; hủy run cũ ở nhánh/PR; main dùng run_id
  riêng để không làm mất run/bằng chứng của commit khác.
- CI mặc định dùng --locked, missing native lock phải lỗi. Bootstrap chỉ có
  khi người vận hành chọn input thủ công; không được dùng để nghiệm thu.
- CI vẫn chỉ có contents: read; bước gộp do agent thực hiện theo ủy quyền,
  không cấp token ghi cho workflow hoặc tự merge mọi PR.
- Không đổi Core, thuật toán, contract, golden hoặc native lockfiles.

## Kiểm chứng

[Local validation](logs/git-ci-workflow-local.json) PASS: parse YAML, event
filters, quyền read-only, group main theo run_id và default không bootstrap;
353 local links hợp lệ, git diff --check đạt. CI trên PR #3 và main sẽ được
bổ sung theo SHA/run sau khi thực chạy.
