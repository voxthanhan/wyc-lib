# Public API mục tiêu — đặc tả P0

Tất cả chữ ký dưới đây là **thiết kế**, không được xuất thành stub P0. Kiểu result
của từng engine được chốt cùng phase trước implementation; không trả generic
JSON tùy ý như public API Rust. Hiện chỉ types và contract validation tồn tại.

## P1: Foundation

```rust,ignore
fn calculate_indicators(data: &MarketData, config: &IndicatorConfig, as_of: i64)
    -> Result<ModuleResult<IndicatorResult>, AnalysisError>;
fn analyze_structure(data: &MarketData, config: &StructureConfig, as_of: i64)
    -> Result<ModuleResult<StructureResult>, AnalysisError>;
fn analyze_batch(request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError>;
impl AnalysisState {
    fn new(request: &AnalysisRequest) -> Result<Self, AnalysisError>;
    fn update_closed_bar(&mut self, update: &ClosedBarUpdate)
        -> Result<AnalysisResult, AnalysisError>;
    fn reset(&mut self, request: &AnalysisRequest) -> Result<AnalysisResult, AnalysisError>;
}
```

Ownership: borrow input, owned immutable outputs. No input mutation; state owns
copies of necessary history. Result metadata: schema/engine/algorithm version,
symbol/timeframe, data_revision, config_hash, as_of; optional typed module outputs
chỉ có payload ở các module được yêu cầu (field module khác là null theo typed
AnalysisResult). Series output có N bản ghi tương ứng tiền tố
khả dụng, NumericSample cho mỗi vị trí (warm-up không bị bỏ). StructureResult có
pivots, labels, trend history, breaks và evidence theo danh mục P1.
Chỉ indicators không chạy structure; chỉ structure không tự tính indicators.
Dependency không có input bắt buộc trả missing_dependency; bản P1 hai module
không cần external input, nên không có trường hợp thiếu benchmark giả định.
Module chưa triển khai trả unsupported_module, không âm thầm bỏ qua.

`ClosedBarUpdate`: bar một hàng đủ OHLCV/availability, toàn metadata định danh,
schema/engine/config/module identity và `as_of`. Chỉ nhận nến closed,
timestamp > last_timestamp, available_at >= last availability và <= as_of,
as_of >= last_as_of. Không append vào state đang có nến đóng chưa khả dụng;
khởi tạo state chỉ giữ tiền tố đã khả dụng. Update trùng/đảo timestamp là lỗi,
đổi revision/config/units/source/symbol/timeframe/module là state_mismatch;
gap hoặc sửa lịch sử trả reset_required. Thất bại phải atomic: state không đổi.

`reset` xây state mới bằng batch trên snapshot hợp lệ rồi thay thế atomic, không
ghi đè các report trước mà caller đã giữ. Truy vấn as_of cũ dùng analyze_batch
và snapshot/revision đúng mốc; không có query quá khứ từ state mới hơn.
Update có thể batch tính lại trong P1; không cam kết O(1) hay throughput.
`AnalysisError` bao bọc ContractError, numeric-overflow và state errors có code/path;
không dùng panic cho dữ liệu hoặc lỗi dự kiến. Các typed output DTO P1 đã có
trong `contracts/src/result.rs`; P1 bổ sung validation toàn kết quả và
AnalysisError trước khi triển khai phép tính.

## P3–P6: chữ ký khung và dependency

Các result/config/input sau còn **draft**, phải được cụ thể hóa khi tới phase.
Các hàm đều nhận borrowed typed input và trả owned Result, as_of bắt buộc.

| Phase | Chữ ký mục tiêu | Input/dependency bắt buộc |
| --- | --- | --- |
| P3 | `detect_price_zones(&ZoneInput, &ZoneConfig, as_of) -> Result<ZoneResult, AnalysisError>` | MarketData, confirmed pivots, ATR |
| P3 | `analyze_volume(&VolumeInput, &VolumeConfig, as_of) -> Result<VolumeResult, AnalysisError>` | OHLCV và rolling statistics |
| P3 | `analyze_relative_strength(&RsInput, &RsConfig, as_of) -> Result<RsResult, AnalysisError>` | symbol, benchmark, universe snapshot cùng mốc/revision policy |
| P4 | `analyze_wyckoff(&WyckoffInput, &WyckoffConfig, as_of) -> Result<WyckoffResult, AnalysisError>` | structure/zones/volume, completed higher-timeframe bars |
| P5 | `evaluate_evidence(&EvidenceInput, &EvidenceConfig, as_of) -> Result<DecisionReport, AnalysisError>` | typed facts/context/hypotheses với known_at <= as_of |
| P6 | `extract_features(&FeatureInput, &FeatureConfig, as_of) -> Result<FeatureResult, AnalysisError>` | typed results và feature schema/version |

P3+ mở rộng AnalysisRequest bằng schema version mới tương ứng với config và
auxiliary inputs có kiểu. Orchestrator chỉ chạy dependency cần thiết; caller
cung cấp dữ liệu ngoại sinh, Core không tải benchmark. Dependency bắt buộc thiếu
trả missing_dependency trước tính toán. Mỗi extension phải có batch/update/reset,
availability, golden và parity tại cổng của nó. Không ép spec tương lai thành
contract đã nghiệm thu hoặc tuyên bố phần chưa có implementation là chạy được.
