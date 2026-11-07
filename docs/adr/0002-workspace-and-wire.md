# ADR-0002 — Workspace tối thiểu và wire số hữu hạn

Trạng thái: chấp nhận cho P0, 07/11/2026.

Quyết định: hai crate contracts/wyc-lib; chưa sinh stub thuật toán/binding.
Rust 1.85.1 pinned, MSRV khai báo 1.85, edition 2021. Windows x64 MSVC là nền
tảng phát triển ban đầu; Linux x64 GNU là runner mục tiêu thứ hai. Ma trận mục
tiêu không tự đồng nghĩa đã hỗ trợ. Phiên bản toolchain mới chỉ nâng sau checks.

Serde phục vụ mapping wire có kiểu, FiniteF64 chặn non-finite, NumericValue
phân biệt ready/warmup/missing. Vec<Option<FiniteF64>> nhận diện null đầu vào
để trả missing_data rõ ràng. Không chấp nhận null thành 0. SHA-256 chỉ dùng ở
test P0 cho checksum; canonical config/event hash là spec để P1 triển khai.

Giữ private source, publish=false, chưa tự chọn giấy phép phân phối thư viện.
Không thêm framework, market-data provider, Node/Python runtime vào Core.
Test target khai báo tường minh trong package để Cargo chạy tài nguyên ở gốc.

Hệ quả: P0 có thể nghiệm thu hợp đồng và oracle, không giả có public analytical
API. CI chạy cùng commands như local. G0 còn cần log CI gắn commit thực tế;
thư mục chưa có Git không cho phép giả lập bằng chứng đó.

Cập nhật 07/11/2026: người dùng đã chỉ định kho voxthanhan/wyc-lib public hiện
tại và dự kiến chuyển private sau; đây là ngoại lệ có ủy quyền đối với định
hướng private source. Git/commit và kiểm chứng clean clone đã hoàn tất. CI đã
kích hoạt nhưng chưa được cấp runner; người dùng yêu cầu để CI chạy sau.
Không thay điều kiện G0 hoặc coi local checks là CI pass.
