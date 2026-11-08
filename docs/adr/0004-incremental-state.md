# ADR-0004 — State incremental sau G1

Ngày: 08/11/2026. Theo yêu cầu tối ưu state, kiểm ZIP artifact độc lập và tự
tích hợp main khi local/CI Windows/Linux đạt.

Giữ public AnalysisState new/result/identity/history_len/update_closed_bar/reset,
wire schema/config/hash/algorithm p1-v1, công thức, thứ tự tính và golden cũ.
Batch vẫn là đường tham chiếu. Append chỉ tính sample mới: rolling/seed dùng
cửa sổ giới hạn và cộng bù theo cùng thứ tự; EMA/Wilder giữ giá trị trước;
MACD/RSI/ADX giữ các dependency smoothing; structure giữ reference high/low,
consumption và trend, chỉ kiểm cửa sổ pivot vừa đủ R nến.

Các kernel smoothing/ratio/directional/bands và bước structure được dùng chung
giữa batch và incremental; không sao chép công thức vào orchestration. Các
kiểu integration tại indicators::incremental/market_structure dùng prefix
AvailableBar đã validate, không phải FFI hoặc contract state serialization.
Caller cấp prefix append-only cùng config/identity; wyc_lib giữ bất biến đó.

Validation hàng mới dùng cùng helper contract như batch, giữ path index toàn
history và thứ tự lỗi cũ. Tính delta trên cache nhỏ tạm, chỉ commit sau khi cả
hai module thành công. Lỗi không thay history/output/cache/identity. Để giữ
ưu tiên lỗi engine cũ khi nhiều phép tính cùng thất bại, đường lỗi có thể gọi
batch trên bản append tạm; đường thành công không gọi batch. Reset xây state
mới/replay rồi thay thế atomic; revision mới vẫn phải reset.

Chi phí tính toán append indicators O(period + seed windows đang warm-up),
structure O(L+R+1); không chạy lại các nến cũ. Evidence giữ thứ tự canonical;
chèn vào vector đã sắp có thể dịch O(E) phần tử. History/output O(N), API trả
owned full report nên clone O(N); không tuyên bố tổng update O(1) hoặc bộ nhớ
cố định. Không đổi accuracy/tolerance để lấy hiệu năng. Đo lại đúng baseline
P1.7 trước/sau cùng profile/toolchain, raw năm mẫu; tách tính toán và output
copy khi báo chi phí.

BasicIndicatorState cũ cũng dùng cache chín chỉ báo nền, không tính advanced
hoặc rolling variance để tránh lỗi từ module không được chọn. API này vẫn
nhận snapshot tích lũy nên validation O(M) và so history O(N) là bắt buộc;
so prefix bằng slices thay vì cấp phát bản copy cũ mỗi update.

Nghiệm thu: golden độc lập giữ nguyên; replay mọi prefix và lịch availability,
seed/warm-up n=1/nondefault/large/tiny, lỗi/rollback sau overflow/conflict và
recover, new/reset từ snapshot giữa chuỗi rồi append, lịch sử revision, module
selection, input/report cũ bất biến. Local và CI Windows/Linux phải đạt. ZIP
CI phải khớp SHA-256 GitHub công bố; kiểm summary commit/clean/checks, test/oracle,
trace và baseline trước khi tuyên bố payload đã xác minh.
