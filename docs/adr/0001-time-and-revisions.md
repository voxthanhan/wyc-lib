# ADR-0001 — Close timestamp, prefix availability và revision

Trạng thái: chấp nhận cho contract 0.1.0, 07/11/2026.

Vấn đề: `is_closed` của snapshot hôm nay không chứng minh dữ liệu đã biết trong
quá khứ; OHLC sửa hồi tố có thể làm mất khả năng tái lập.

Quyết định: dùng timestamp biên đóng UTC milliseconds, available_at từng nến,
data_revision và revision_available_at; từ chối phiên bản lịch sử chưa biết ở
as_of. Availability không giảm và nến đóng tạo thành tiền tố. Đến muộn chen giữa,
gap đã khai báo hay revision thay đổi cần snapshot liên tục mới và reset/replay.
Không suy lịch thị trường từ khoảng cách timestamp. Giữ snapshot_kind để phân
biệt dữ liệu caller chứng minh point-in-time với revised_history.

Hệ quả: contract đơn giản, chưa xử lý stream đảo thứ tự hoặc kho revision trong
Core. Caller chịu trách nhiệm độ trung thực của availability. Quy tắc này bổ sung
các trường còn để mở trong thiết kế, không chuyển trách nhiệm dữ liệu vào Core.
Kiểm chứng: temporal fixture, replay từng mốc, revision counterexample, pivot
known_at muộn. Replay P0 chỉ kiểm tra contract, không nhận là replay thuật toán.
