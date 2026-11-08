# Regression

Ca nền temporal/revision/non-finite và ca âm harness nằm trong `tests/foundation.rs`.
Regression thuật toán mới phải có fixture và expected xác minh độc lập cùng version.

P1.7 không thay thuật toán/golden. Target `p1_replay` bổ sung regression qua API
công khai trên toàn 15 fixture structure, ba lựa chọn module và hai lịch
availability (gốc và từng cặp đến trễ); so sánh mọi prefix với state, giữ report
cũ rồi query lại cùng as_of, thay OHLCV tương lai hợp lệ. Trường hợp cùng as_of
chỉ so full snapshot khi cả nhóm đã append. Revision mới rescale history rồi
reset tại nửa chuỗi và append phần còn lại; revision chưa khả dụng phải atomic.
Lookback 1/2/3/8 kiểm first-valid, SMA/EMA/WMA đối chiếu closed form của ramp.

Runner xuất `ci-evidence/replay-traces.json`: config fixture, modules, prefix,
as_of và digest toàn output. Đây là dấu vết parity/causality, không tạo expected
bằng implementation để thay golden số độc lập. Golden cũ được checksum và
oracle Fraction/Decimal/hashlib kiểm lại, không tự cập nhật khi chạy test.

Sau G1, target p1_incremental kiểm cache qua chuỗi dài, zero/tiny/large,
lookback 1/7/32/512, checkpoint giữa seed, rollback sau module indicators trước
lỗi structure và recover, revision/reset rồi append. BasicIndicatorState được
kiểm ở 1e200 để chứng minh không tính advanced variance ngoài chín series.
Toàn bộ 1008 prefix hashes của G1 cũ phải giữ nguyên. Các artifact đã tải được
kiểm digest/CRC rồi đối chiếu summary/checks/tests/trace/raw baseline độc lập;
verified CI payload logs được giữ nguyên bytes bằng .gitattributes -text.
