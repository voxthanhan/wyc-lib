# ADR-0003 — Điều phối và state P1.6

Ngày: 08/11/2026. Quyết định chốt trước implementation P1.6.

## API và cấu hình

Ba entry points `calculate_indicators`, `analyze_structure`, `analyze_batch`
đặt tại wyc_lib. Hai API đơn module nhận `&MarketData, &AnalysisConfig, as_of`,
thay cho IndicatorConfig/StructureConfig trong dự thảo P0: hash toàn config đã
được các engine P1.3–P1.5 sử dụng; không chèn default cấu hình ẩn để tạo hash.
Không thay chữ ký engine đã có, schema 0.1.0, config p1-v1 hoặc công thức.

Indicators trả đúng 25 IndicatorKind: chín series basic, 11 advanced, cùng
sum/min/max/variance/std rolling của close. SMA chính là rolling mean, không
thêm series mean. N samples mỗi series, kể cả warm-up/missing. Dùng primitives
và engines đã nghiệm thu, không sao chép công thức vào orchestrator.
Kiểm catalog, shape, thời gian và warm-up trước khi xuất kết quả. Structure
dùng nguyên engine đã kiểm lineage/reference/evidence. Batch validate request,
chuẩn hóa module, chỉ chạy module được chọn; P1 hai module độc lập. Các module
tương lai vẫn unsupported_module. Thứ tự lỗi tính toán: indicators rồi structure.

## Append protocol

Contracts thêm hai DTO có serde deny_unknown_fields, không thay DTO đã có:

- `ClosedBar`: timestamp, available_at, open/high/low/close/volume
  (Option<FiniteF64>, null được báo missing_data), is_closed.
- `ClosedBarUpdate`: identity: StateIdentity, metadata: MarketMetadata,
  bar: ClosedBar, as_of: i64.

Caller lấy `state.identity().clone()` và metadata của snapshot để tạo update.
Identity phải khớp chính xác, gồm phiên bản, hash, modules đã sắp, revision,
last_timestamp và last_as_of; cursor cũ không dùng lại được. Metadata đầy đủ
phải giữ nguyên. Không có timestamp hoặc config ẩn từ giờ hệ thống.

`AnalysisState::new` chạy batch và chỉ lưu prefix đã khả dụng; future trong
snapshot không được giữ. Getters result/identity/history_len chỉ đọc. Mỗi
update thêm đúng một hàng; timestamp tăng nghiêm ngặt, available_at không giảm,
nến đã đóng và khả dụng tại as_of; as_of không giảm. Equality availability và
as_of được chấp nhận. Chênh timestamp không tự được coi là gap.

Thứ tự lỗi update: gaps không rỗng → reset_required tại metadata.gaps;
identity/metadata khác → state_mismatch tại state.identity;
as_of sai miền → invalid_timestamp tại as_of; as_of giảm →
invalid_availability tại as_of; nến chưa đóng → invalid_availability tại
state.new_bar. Sau đó append vào bản sao, dùng batch validation cho timestamp,
availability, OHLCV (path index trong history tích lũy). Nến chưa khả dụng →
invalid_availability tại state.new_bar. Numeric overflow/structure conflict
giữ nguyên lỗi engine. Mọi lỗi giữ toàn bộ state cũ.

Đây là protocol append một nến, không nhận history để sửa hoặc tự phát hiện
caller sửa history ngoài state mà vẫn khai cùng revision. Khi sửa lịch sử,
caller phải đổi revision và gọi reset với snapshot mới; update đổi revision
bị state_mismatch. Reset xây replacement bằng new, chỉ thay thế khi thành công;
có thể đổi config/modules/metadata/as_of. Báo cáo owned caller giữ không đổi.
Truy vấn quá khứ dùng batch trên snapshot/revision phù hợp, không đọc state mới.

## Chi phí và nghiệm thu

Update tính lại batch trên N nến retained. Không O(1), không state serialization,
cache giới hạn cố định hoặc tuyên bố point-in-time ngoài metadata caller cung cấp.
Chi phí indicators O(N×min(n,N)), structure O(N×min(L+R+1,N)+N log N), thêm
validation snapshot; bộ nhớ O(N), mỗi lần trả clone output O(N). Việc gọi engine
đã có có thể lặp validation/tính primitives; tối ưu và baseline thuộc P1.7.

Nghiệm thu: đối chiếu golden độc lập hiện có qua entry points; đủ 25 keys,
JSON/null và metadata ổn định; chứng minh không chạy module không chọn bằng
overflow/conflict chỉ xuất hiện ở module đó; replay state so batch mỗi prefix,
delayed/open/future/revision, identity/stale cursor và atomic lỗi/reset.
Local và CI Windows/Linux phải chạy target p1_orchestration. G1 vẫn chờ P1.7.
