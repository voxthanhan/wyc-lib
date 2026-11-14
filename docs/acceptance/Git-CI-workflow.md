# Quy trình Git/CI tự động — 14/11/2026

**Hoàn tất và đã kiểm chứng trên main `46041c6`.**
[PR #3](https://github.com/voxthanhan/wyc-lib/pull/3) đã gộp sau khi Core/native
CI cho đúng HEAD `aaf535b` đạt cả Windows/Linux. CI main sau gộp cũng PASS cả
hai hệ điều hành tại đúng `46041c61c82b1aff57822e0f5461c2ffda4f059c`.

Người dùng yêu cầu các lần sau tự động chạy, không hỏi lại.
[AGENTS.md, mục 12](../../AGENTS.md) lưu nguyên văn ủy quyền thường trực cho
repository công khai `voxthanhan/wyc-lib`. Với tác vụ đã giao, agent được tự
commit/push/PR, theo dõi và sửa CI, gộp main khi đúng HEAD đạt đủ gates, kiểm
CI sau gộp và cập nhật báo cáo. Không tự triển khai phase chưa được giao.

## Thay đổi

| Thành phần | Hành vi hiện tại |
| --- | --- |
| Push | Core/native CI tự chạy trên main và work/**, không cần thêm từng tên nhánh phase |
| Pull request | Core/native CI tự chạy khi PR nhắm main |
| Dispatch | Vẫn cho phép chạy thủ công; bootstrap native locks mặc định false |
| Concurrency | Tách workflow/event/ref, hủy run cũ trên cùng nhánh/PR; main dùng run_id riêng để giữ từng run |
| Lockfiles | Mặc định --locked; thiếu lock phải lỗi, không tự bootstrap |
| Quyền GitHub Actions | contents: read; bước gộp do agent kiểm gates và thực hiện bằng connector |
| Ủy quyền | Commit/push/PR/CI/merge trong phạm vi tác vụ được giao, không xin lại từng bước |

Ủy quyền không tự mở sandbox/ACL hay tắt cơ chế duyệt nền tảng. Publish
npm/PyPI, release/tag, đổi quyền repo và công việc ngoài phạm vi vẫn theo yêu
cầu riêng. Không thêm token ghi vào workflow, không tự merge mọi PR trong repo.
Core, thuật toán, contracts, golden và native Cargo.lock giữ nguyên.

## Kiểm chứng

[Local validation](logs/git-ci-workflow-local.json) PASS: parse YAML/event
filters, quyền read-only, group main theo run_id và default không bootstrap;
353 links hợp lệ tại lần kiểm tra đó, git diff --check đạt.

| Commit/event | Core | Native | Kết quả |
| --- | --- | --- | --- |
| aaf535b / push | [43586654916](https://github.com/voxthanhan/wyc-lib/actions/runs/43586654916) | [43586654913](https://github.com/voxthanhan/wyc-lib/actions/runs/43586654913) | Windows/Linux PASS |
| aaf535b / PR #3 | [43586658315](https://github.com/voxthanhan/wyc-lib/actions/runs/43586658315) | [43586658328](https://github.com/voxthanhan/wyc-lib/actions/runs/43586658328) | Windows/Linux PASS |
| 46041c6 / main | [43587159650](https://github.com/voxthanhan/wyc-lib/actions/runs/43587159650) | [43587159760](https://github.com/voxthanhan/wyc-lib/actions/runs/43587159760) | Windows/Linux PASS |

[Run/jobs/artifacts của PR](logs/git-ci-aaf535b-ci.json),
[native push Windows](logs/git-ci-aaf535b-push-windows.json),
[native push Linux](logs/git-ci-aaf535b-push-linux.json),
[native PR Windows](logs/git-ci-aaf535b-pr-windows.json),
[native PR Linux](logs/git-ci-aaf535b-pr-linux.json) ghi commands/exit codes,
locked/clean flags và commit checkout. PR checkout merge ref eefd995 khác
HEAD aaf535b; run metadata liên kết đúng HEAD và tree được kiểm khi gộp.

[CI main](logs/git-ci-main-46041c6-ci.json),
[native main Windows](logs/git-ci-main-46041c6-windows.json) và
[native main Linux](logs/git-ci-main-46041c6-linux.json) xác nhận:

- 102 Rust tests/OS và Core format/Clippy/build/oracle/replay đều PASS.
- 43 native commands/OS exit 0, --locked, checkout sạch trước/sau, không bootstrap.
- Installed wheel ngoài source: 72 plain và 79 extras Python assertions/OS;
  mypy strict, 67 host conversion assertions/OS, 83 Node typed assertions/OS.
- JSON regression: 40 actions/runtime/OS, 14 direct-Core references, 20 ca lỗi.
- [104 tệp liên quan](logs/git-ci-aaf535b-source-comparison.json) khớp bản PR;
  [merge tree](logs/git-ci-main-46041c6-source-comparison.json) bằng tree PR đã đạt.

Native local vẫn chưa xác minh do dependency/cache/mạng shell. Artifact
metadata/digests lấy từ GitHub và package checksums từ decoded CI logs; chưa
kiểm CRC/SHA-256 ZIP payload độc lập trong workspace. Không claim đo leak/RSS
hoặc binary reproducible. Manual bootstrap=true và stress-test hủy run đồng
thời chưa chạy; push/PR/main/default locked đã được kiểm thực tế.

Tài liệu/bằng chứng sau nghiệm thu chỉ đổi docs/logs. Source engine/bindings,
AGENTS.md và hai YAML workflows giữ nguyên so commit main đã kiểm chứng.
