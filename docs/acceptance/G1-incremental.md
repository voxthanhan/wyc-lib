# G1 bổ sung — incremental state và ZIP artifact

Ngày: 08/11/2026 (Asia/Saigon). Phạm vi theo yêu cầu tối ưu sau G1/P1.7;
local/CI/main của bản mới đang được kiểm chứng, chưa lấy kết quả cũ thay bản mới.

## Thiết kế và sản phẩm

[ADR-0004](../adr/0004-incremental-state.md) chốt trước code. AnalysisState
append chỉ tính 25 sample mới và các sự kiện cấu trúc mới, giữ cache EMA/Wilder,
MACD/RSI/ADX, reference/consumption/trend. Rolling tính một cửa sổ hiện tại,
giữ thứ tự Neumaier/two-pass; kernel số và bước structure dùng chung với batch.
BasicIndicatorState cũng dùng cache chín chỉ báo, không tính advanced thừa;
API snapshot cũ vẫn validate/so toàn lịch sử. Contract row validation dùng chung
batch/append, giữ code/path/thứ tự lỗi. Stage cache nhỏ rồi commit cả hai module;
đường lỗi có thể replay batch để giữ ưu tiên lỗi khi nhiều phép toán cùng lỗi.
New/reset vẫn batch validate rồi dựng cache, không dùng state tương lai.

Public API cũ, wire/schema/config/hash/algorithm p1-v1, seed/tolerance và toàn
golden giữ nguyên. Không dependency mới, unsafe, SIMD hoặc đa luồng. Các kiểu
integration tại indicators::incremental và market_structure chỉ nhận prefix
AvailableBar đã validate/không sửa, binding identity/revision do wyc_lib giữ.

## Kiểm chứng ZIP G1 cũ

Đã tải bốn ZIP của CI nhánh 42398202869 và [CI main 42399120154](https://github.com/voxthanhan/wyc-lib/actions/runs/42399120154)
tại các SHA 7c57220 và 223d742cf6e03ff91934d59647c50f30380c2eb6. Digest tính từ bytes ZIP khớp
SHA-256 do GitHub công bố, CRC đạt. Cả bốn payload có summary đúng SHA/run,
clean_before/clean_after=true, toàn checks exit=0; 88 tests không failed/ignored,
1.008 prefix traces khớp hoàn toàn local Windows và 12 baseline workloads với
năm raw samples, CPU/RSS đầy đủ. Giới hạn chưa đọc ZIP ở báo cáo G1 cũ đã đóng.
[Kiểm ZIP nhánh cũ](logs/g1-incremental-previous-ci/42398202869/verification.json),
[kiểm ZIP main cũ](logs/g1-incremental-previous-ci/42399120154/verification.json).
ZIP gốc ở `.tools/g1-artifacts/<run>/<artifact>/artifact.zip`; metadata/checks
và selected payload logs được lưu nguyên bytes trong repository, toàn bộ file
checksums và đối chiếu 1008 local trace digests nằm trong hai verification JSON.

Tải qua HTTP redirect nightly.link từ artifact công khai, không token hoặc
cài/cấp quyền app; digest GitHub là chuẩn đối chiếu trước unzip. Không ghi URL
redirect có chữ ký. Công cụ Chrome bị dừng do không xác định URL đủ tin cậy;
không tiếp tục thao tác Chrome. Việc kiểm ZIP dùng script không điều khiển UI.
[Verifier](../../scripts/verify_ci_artifacts.py) kiểm size/path/duplicate/symlink,
digest/CRC, summary/checks/test/oracle, trace, raw baseline/min/median/max/CPU/RSS;
[negative controls](../../scripts/test_verify_ci_artifacts.py) không cần network.

## Kiểm chứng bản incremental

Bốn ca mới: 2.560 prefix indicators ở bốn scale (zero/normal/tiny/large) và
lookback 1/7/32/512, checkpoint trong/sau seed; rollback sau indicators thành
công nhưng structure conflict rồi recover; 256 nến cấu trúc/evidence/ID,
revision reset giữa history và append; numeric overflow tại seed rồi recover.
Ca thứ năm kiểm BasicIndicatorState ở 1e200, nơi advanced variance sẽ overflow,
để chứng minh không chạy advanced từ API basic. Bộ cũ tiếp tục kiểm golden
độc lập, delayed/grouped availability, lỗi identity/cursor và input/report cũ.

Local Windows x64 MSVC, Rust/Cargo 1.85.1, Python 3.10.9: runner chung đạt
format/Clippy all-targets -D warnings/build, **93 integration tests**, ví dụ Rust,
release baseline, oracle độc lập, **5 Python verifier controls** và workspace
boundary/link audit. Không failed/ignored, không tắt lint hoặc nới tolerance.
Bộ 16 fixture/golden/manifest cũ giữ nguyên bytes; toàn 1.008 complete-output
prefix hashes khớp G1 cũ, gồm giá trị/mask/time/evidence IDs. Cargo.lock,
workspace dependencies, toolchain, workflow và Rust benchmark generator không đổi.
69 fingerprints nguồn/config/tests/scripts đã ghi.

[Summary local](logs/g1-incremental-windows/summary.json),
[tests](logs/g1-incremental-windows/test.txt),
[trace](logs/g1-incremental-windows/replay-traces.json),
[baseline](logs/g1-incremental-windows/baseline.json),
[verifier controls](logs/g1-incremental-windows/artifact-verifier.txt),
[review](logs/g1-incremental-windows/review.txt),
[69 fingerprints](logs/g1-incremental-windows/sources.json).
Summary ghi base HEAD 64fa87f, clean=false vì working tree chưa commit; không
coi đây là checkout sạch. Các lượt Clippy đầu bắt needless_borrow ở kernel đã
tách; sửa đúng tham chiếu, lượt đầy đủ cuối PASS. Checkout sạch/CI bản mới
sẽ được bổ sung trước nghiệm thu cuối.

## Baseline trước–sau

Đã đo bản 64fa87f trước sửa (binary giữ nguyên), rồi đo một cặp trước–sau tuần
tự sau sửa bằng cùng generator/config/hash/toolchain/profile trên cùng Windows,
CPU Intel Family 6 Model 142, 8 logical CPUs. Mỗi workload 1 warm-up + 5 mẫu.
Binary SHA-256 được lưu, compiler không nằm trong CPU/RSS phép đo. Metadata
commit của cặp còn là base HEAD, clean=false; comparison.json ghi rõ binary
cũ đã copy trước optimized release build, không nhầm với mã trên working tree.

| Workload | Trước 256 (ms) | Sau 256 (ms) | Trước 1024 (ms) | Sau 1024 (ms) |
| --- | ---: | ---: | ---: | ---: |

| batch_indicators | 4.4648 | 6.9070 | 17.5164 | 20.4787 |
| batch_structure | 0.4128 | 1.4191 | 1.4337 | 3.0216 |
| batch_both | 4.5733 | 4.8405 | 18.4477 | 21.7290 |
| append_one_both | 5.1532 | 0.6952 | 20.5307 | 2.0454 |
| replay_both | 612.0742 | 24.7656 | 8418.0689 | 174.1173 |
| watchlist_8_sequential | 39.8139 | 39.3348 | 137.5301 | 147.4732 |

Median append N=1024 giảm khoảng 10.0×, replay N=1024 khoảng 48.3× trong cặp đo
này. CPU toàn benchmark process từ 61.140625 s xuống 3.09375 s; peak RSS từ
14,946,304 xuống 13,594,624 bytes. Không suy thành speedup mọi input/hệ thống.
Các batch-only samples có chỗ chậm hơn; giữ đầy đủ min/max/raw, không bỏ các
hàng này hoặc gọi tối ưu state là cải thiện toàn Core. Shared structure step
đã tránh clone cache/cấp phát delta mỗi nến trên batch; máy đo còn noise,
không có thí nghiệm kiểm soát/statistical significance cho chênh batch.

[Raw/comparison và phạm vi](logs/g1-incremental-windows/comparison/comparison.json),
[trước](logs/g1-incremental-windows/comparison/before.json),
[sau](logs/g1-incremental-windows/comparison/after.json),
[baseline trước sửa lần đầu](logs/g1-incremental-windows/initial-before/baseline.json).
Không đo speedup riêng BasicIndicatorState từ workload AnalysisState này.


## Giới hạn còn lại

Tính toán append O(period + các seed window chưa đầy) và O(L+R+1) cho structure.
Evidence insertion có thể dịch O(E) phần tử; owned full result vẫn clone O(N),
history/output O(N). Tổng update không O(1). Basic snapshot API còn validation
O(M), so history/output O(N). New/reset xây cache từ toàn prefix một lần và
có thêm chi phí so bản batch-only cũ; không có serialized-state ABI hoặc FFI.
Không tuyên bố hiệu quả đầu tư hoặc độ chính xác point-in-time ngoài metadata
ứng dụng. P2/đa timeframe/ML và phát hành package/tag ngoài phạm vi.
