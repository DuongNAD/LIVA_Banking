# Original User Request

## 2026-09-14T06:14:50Z

Use the full multi-agent team. Hoàn tất Milestone M4 & M5 của dự án LIVA Banking: Thiết lập và đồng bộ hóa sổ theo dõi nợ kỹ thuật (tech-debt-ledger.json), cập nhật toàn diện tài liệu tổng quan dự án PROJECT.md phản ánh trung thực hiện trạng mã nguồn thực tế, và thực thi kiểm định toàn diện qua Full Verification Matrix kết hợp kiểm thử đối kháng (Adversarial Challenge).

Working directory: e:/Project/01_AI_Agents/LIVA_Banking
Integrity mode: development

## Requirements

### R1. Đồng bộ hóa Sổ nợ kỹ thuật và Hồ sơ dự án (Milestone M4)
- Thiết lập hoặc cập nhật tệp tech-debt-ledger.json ghi nhận đầy đủ, chuẩn xác trạng thái khép lại của 10 hạng mục nợ kỹ thuật cốt lõi (D-01 đến D-10 từ Milestone M1–M3) với bằng chứng mã nguồn (file:dòng) và kết quả kiểm thử tương ứng.
- Phân loại rõ ràng các khoản nợ kỹ thuật còn tồn đọng (C1, C3, H4, H5 và danh mục Medium/Low) với kế hoạch xử lý hoặc ghi chú rủi ro có chủ đích.
- Cập nhật hoặc khởi tạo tài liệu PROJECT.md tại thư mục gốc của dự án phản ánh chính xác cấu trúc sau di trú Rust Native Core (liva-native-core), trạng thái các phân hệ (Desktop Tauri, UI Vue 3, SQLite WAL DB Actor, Voice Pipeline, Security Scrubber), và lộ trình kỹ thuật hiện hành.

### R2. Rà soát và chuẩn hóa tài liệu kỹ thuật
- Rà soát các tài liệu đánh giá trong thư mục docs/03-danh-gia/ để đóng băng các bảng rủi ro cũ đã được khắc phục, đảm bảo tính nhất quán tuyệt đối giữa tài liệu thiết kế, hồ sơ nghiệm thu và mã nguồn.
- Đảm bảo toàn bộ liên kết nội bộ, trích dẫn file và front-matter trong tài liệu tuân thủ chuẩn kiểm toán tài liệu của dự án.

### R3. Thực thi và nghiệm thu Full Verification Matrix (Milestone M5)
- Thực thi toàn bộ chuỗi kiểm định chất lượng nghiêm ngặt của dự án đảm bảo tính trọn vẹn, không có cảnh báo hay lỗi tiềm ẩn trên môi trường Windows x64.
- Kích hoạt và vượt qua bộ kiểm thử đối kháng IPC/WebSocket (Adversarial Challenge) chống payload dị dạng, flood request và tấn công leo quyền.
- Đo kiểm và xác nhận cơ chế giải phóng bộ nhớ nhàn rỗi (idle memory reclamation) cho các mô hình âm thanh (STT Parakeet và TTS VieNeu) đưa RAM tiến trình về ngưỡng an toàn.

### R4. Quy tắc an toàn tài nguyên & hạ tầng
- Tuân thủ nguyên tắc giới hạn tài nguyên khi biên dịch và chạy kiểm thử: luôn truyền cờ -j 2 cho cargo check, cargo build, cargo test, và -- --test-threads 2 cho test runner.
- Chạy các lệnh kiểm thử và build tuần tự, tuyệt đối không chạy song song các tiến trình biên dịch nặng để bảo toàn RAM hệ thống.
- Tuyệt đối không tự động thực hiện các thao tác git remote (git push, git pull, git commit). Ranh giới git dừng lại ở staging (git add).

## Acceptance Criteria

### Tính toàn vẹn của Sổ nợ & Hồ sơ dự án (M4)
- [ ] Tệp tech-debt-ledger.json tồn tại ở định dạng JSON chuẩn, liệt kê chi tiết 10 hạng mục D-01 đến D-10 kèm trạng thái RESOLVED, file nguồn, commit/bằng chứng, cùng phân loại rõ ràng các mục nợ còn lại (C1, C3, H4, H5).
- [ ] Tệp PROJECT.md tại thư mục gốc phản ánh đầy đủ kiến trúc Native Rust Core, trạng thái phân hệ, ma trận kiểm định và hướng dẫn chạy kiểm tra hệ thống.
- [ ] node scripts/docs-check.mjs chạy thành công (exit code 0) không có cảnh báo liên kết hỏng hoặc trích dẫn sai lệch.

### Cổng kiểm định Mã nguồn Rust Native Core
- [ ] cargo fmt --all -- --check đạt chuẩn format (exit code 0).
- [ ] cargo clippy --workspace --all-targets -j 2 -- -D warnings đạt 0 warning, 0 error (exit code 0).
- [ ] cargo test --workspace -j 2 -- --test-threads 2 vượt qua 100% các unit test và integration test mà không có bất kỳ test nào thất bại (0 failed).

### Cổng kiểm định Giao diện & Frontend
- [ ] npx vue-tsc --noEmit -p liva-ui/tsconfig.app.json không phát hiện lỗi kiểu dữ liệu (0 type errors).
- [ ] npx eslint . --max-warnings 0 đạt 0 warning, 0 error.
- [ ] npm run test:coverage -w liva-ui đạt 100% pass và vượt ngưỡng ratchet coverage được cấu hình trong vitest.config.ts.

### Kiểm thử Vận hành, Đối kháng & Quản lý Bộ nhớ
- [ ] cargo test --test m3_ipc_sync_adversarial_challenge -j 2 -- --test-threads 2 vượt qua toàn bộ các kịch bản kiểm thử đối kháng (5/5 kịch bản pass).
- [ ] node scripts/e2e-gateway-ci.mjs kết nối WebSocket thành công và vượt qua tất cả các kịch bản E2E (8/8 pass).
- [ ] node scripts/e2e-memory.mjs xác nhận giải phóng bộ nhớ khi nhàn rỗi đưa RAM về <= 350MB.
- [ ] npm run skills:audit xác nhận tính toàn vẹn của các plugin/skills trong hệ sinh thái.

## 2026-09-14T08:35:05Z

Build a comprehensive, human-designed, light-themed 25-slide presentation deck in HTML, PPTX, and PDF formats for **LIVA Banking Harness** competing at **INNOSTART 2026 Demo Day (16/09/2026)**. The deck contains the 5-minute Pitch (Round 1), the 1-Year Development Plan (Round 2 Top 3), and the 7-minute Q&A Defense Appendix, with clean corporate aesthetics, authentic engineering metrics, and rich visual diagrams.

Working directory: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_slides`
Integrity mode: demo

Reference material:
- Dossier Manifest & Master Navigation: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_harness/README.md`
- 10-Slide Specification: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_harness/PITCH_DECK_INNOSTART_2026.md`
- 5-Minute Speech Playbook: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_harness/PITCH_SCRIPT_5MIN.md`
- 7-Minute Defense & Q&A Playbook: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_harness/QA_DEFENSE_7MIN.md`
- Executive Summary: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_harness/EXECUTIVE_SUMMARY.md`

## Requirements

### R1. Tri-Format Deliverables (HTML, PPTX, PDF)
Produce the complete slide presentation across three accessible, production-grade formats:
- **Interactive HTML5 Presentation (`index.html`)**: Single-file, zero-runtime dependency web deck with modern light theme, smooth slide navigation (arrow keys, spacebar, swipe), full-screen F11 mode, presenter overview, responsive card layouts, and CSS print stylesheet configured for 1-click clean export.
- **PowerPoint Presentation (`liva_innostart_2026.pptx`)**: Fully editable 16:9 widescreen presentation file with structured slide layouts, editable text cards, KPI metric badges, tables, and vector shapes formatted in professional corporate styling.
- **High-Resolution PDF (`liva_innostart_2026.pdf`)**: Print-ready, vector-sharp document matching competition submission criteria.

### R2. 25-Slide Deck Content Architecture (3 Integrated Modules)
- **Module A: Vòng 1 — Pitching Đề Án 5 Phút (Slides 1–12)**:
  1. *Title*: LIVA Banking Harness — Tác tử AI Cục bộ Điều phối Đối soát & Nguồn vốn Doanh nghiệp.
  2. *Problem*: Ác mộng đối soát sổ phụ 2-4h/ngày & Vùng mù thanh khoản T+1..T+3.
  3. *Regulatory Red Line*: Lằn ranh đỏ Nghị định 13/2023/NĐ-CP & Thông tư 09/2020/TT-NHNN.
  4. *Customer & Insight*: Khách hàng kép (Khối Vận hành Ngân hàng & CFO Doanh nghiệp Vừa/Lớn).
  5. *Solution*: LIVA Banking Harness — Nền tảng Local-First trên lõi Rust Native (`liva-native-core`).
  6. *Value Proposition*: Tam giác giá trị định lượng (-75% thời gian, dự báo 24-48h, 100% Zero Cloud Leakage).
  7. *Product & MVP*: Kiến trúc Rust Native, bóc tách thực thể bằng SLM + tính toán số học xác định 100%.
  8. *Security & Compliance*: Hàng rào 5 tầng (AES-256-GCM, DPAPI/TPM 2.0, PolicyEngine Xác nhận 2 pha).
  9. *Competitive Moat*: Ma trận so sánh LIVA vs RPA truyền thống (UiPath) vs Cloud AI SaaS.
  10. *Business Model*: B2B Dual-Track (License Ngân hàng & Subscription CFO), biên gộp >88%.
  11. *Traction & Lab Validation*: Báo cáo thực chứng 50.000 dòng sao kê (<0.5ms/dòng, RAM 680MB, khớp 99.8%).
  12. *Vision & Seed Ask*: Chủ quyền AI Tài chính, Vòng Seed $500k–$750k (Định giá $4.0M–$5.0M).

- **Module B: Vòng 2 — Kế Hoạch Phát Triển 1 Năm Top 3 (Slides 13–19)**:
  13. *1-Year Vision*: Trở thành Đai an toàn Tác tử Tài chính số 1 Việt Nam.
  14. *Current State*: Lõi Native Rust hoàn chỉnh, MVP Treasury Workbench đo kiểm 50k dòng.
  15. *1-Year Product Roadmap & KPI*: Q4/2026 - Q3/2027 (Connector 35+ ngân hàng VN, ERP SAP/MISA/FAST).
  16. *Marketing & GTM*: Kênh đối tác ERP/ISV, Direct Sales 200+ CFO, Hội thảo CFO Summit.
  17. *Financial Plan*: Dự phóng hòa vốn T11/2027, ARR Base Case $3.9M vs Bull Target $7.2M năm 2028.
  18. *Team & Resources*: 5 Nhà sáng lập (Dương, H.Hiếu, Minh, M.Hiếu, Đại) + Kế hoạch tuyển dụng kỹ sư.
  19. *Milestones & Risk Management*: Pháp lý Sandbox SBV, ISO 27001, DPIA A05, và kịch bản ứng phó rủi ro.

- **Module C: Q&A Phản Biện Chuyên Sâu 7 Phút (Slides 20–25)**:
  20. *Deep Dive SLM vs Cloud LLM*: Tại sao ngân hàng bắt buộc dùng mô hình 3B-8B Q4 cục bộ.
  21. *Deep Dive Khử Ảo Giác Kế Toán*: Kiến trúc Disentanglement phân tách SLM ngữ nghĩa và Rust số học.
  22. *Deep Dive Tích Hợp Non-Invasive*: Track 1 Hot-Folder máy trạm CFO vs Track 2 DMZ SFTP Ngân hàng.
  23. *Deep Dive Đo Kiểm 50k Dòng*: Biểu đồ độ trễ, mức tiêu thụ tài nguyên và cơ chế Fail-Closed 0% sai sót.
  24. *Deep Dive Phân Bổ Nguồn Vốn*: Chi tiết 50% R&D, 25% GTM, 15% Sandbox/Pháp lý, 10% Runway.
  25. *Executive Team Profile*: Danh sách 5 thành viên với vai trò linh hoạt sẵn sàng tùy biến.

### R3. Human-Designed Light Aesthetic
- **Color Scheme**: Clean light theme — Pure White canvas (`#FFFFFF`), Soft Slate card background (`#F8FAFC`), Deep Navy Blue primary brand (`#0F2042`), Trust Emerald accent (`#059669`), Subdued slate text (`#1E293B`, `#475569`), subtle card outlines (`#E2E8F0`).
- **Typography & Layout**: Standard 16:9 widescreen layout, large readable headings, pill badges, structured comparison grids, and generous breathing space.
- **Visual Assets**: Crisp SVG flowcharts, architectural block diagrams, pipeline steps, and visual callout metrics.

## Acceptance Criteria

### Format Deliverables
- [ ] `index.html` exists in the working directory, opens without errors in modern browsers, and contains all 25 slides navigable with keyboard (Left/Right arrow, Spacebar) and on-screen controls.
- [ ] `liva_innostart_2026.pptx` exists in the working directory, is valid, non-corrupted, opens cleanly in PowerPoint / Google Slides / LibreOffice, and contains 25 corresponding editable slides.
- [ ] `liva_innostart_2026.pdf` exists in the working directory with 25 vector-sharp slides ready for printing or direct competition submission.

### Content Completeness & Accuracy
- [ ] All 25 slides reflect exact business and technical figures from the verified project dossier (50,000 rows, 0.38ms, 680MB RAM, $500k-$750k Seed ask, $4.0M-$5.0M post-money).
- [ ] All 5 team members (Nguyễn Anh Dương, Nguyễn Hoàng Hiếu, Nguyễn Huy Anh Minh, Nguyễn Minh Hiếu, Cao Xuân Đại) are accurately listed with placeholder/adjustable roles.
- [ ] All sections required by INNOSTART 2026 (Problem, Customer, Solution, Value Prop, Product/MVP, Advantage, Business Model, Traction, Feasibility, Vision, plus 1-Year Plan & Q&A) are fully covered.

## 2026-09-14T08:56:52Z

CRITICAL USER FEEDBACK:
The user explicitly requested: "à tôi k muốn nó có quá nhiều chữ và phải chuyên nghiệp" (Do not make the slides text-heavy; keep them concise, visual, high-impact, and ultra-professional).

Requirements to enforce immediately:
1. Reduce slide body text drastically: Maximum 3-4 bullet points or concise metric cards per slide. No long narrative paragraphs or walls of text on the slide surface.
2. Emphasize big bold typography for key metrics (e.g. 75%, <0.5ms, 99.8%, $500k-$750k, 100% Zero Cloud Leakage, 680MB RAM).
3. Move lengthy explanations and full narrative arguments into the Speaker Notes (Presenter Notes) so the presenter can read them, while the visual slide remains clean, spacious, elegant, and punchy.
4. Ensure generous whitespace and clean visual card containers matching top-tier tech startups (Y Combinator / Stripe / Apple style).

Please refine both index.html, liva_innostart_2026.pptx, and the resulting PDF to reflect this light, clean, low-text, highly professional aesthetic.

## 2026-09-14T08:58:52Z

CRITICAL USER REQUIREMENT: RADICAL HONESTY & TRANSPARENT LIMITATIONS
The user explicitly demanded: "nói điểm yếu cx đc hoặc cái j không nào đc cx đc nhưng k đc nói dối phải nói thật" (It is completely fine to state weaknesses or what cannot be done, but DO NOT LIE — MUST TELL THE TRUTH).

Mandatory rules to reflect across all slides, script notes, and Q&A defense:
1. Radical Honesty on Traction & Commercial State:
   - Clearly state that Traction is from LAB BENCHMARK on 50,000 standardized simulated multi-bank statement rows, NOT audited production commercial revenue.
   - Do NOT inflate or invent fake paying enterprise clients. State clearly: "Giai đoạn hiện tại: MVP Lab Benchmarked & Sẵn sàng PoC/Sandbox với đối tác ngân hàng và doanh nghiệp".
2. Transparent About Weaknesses & Boundaries (What LIVA Can and Cannot Do):
   - Cannot do: Open-domain creative reasoning or generic chatbot tasks (Local SLM 3B-8B is strictly scoped to structured entity extraction, not replacing general LLMs).
   - Cannot do: Auto-execute arbitrary unverified transfers without human review (Strict PolicyEngine Fail-Closed: 0.2% ambiguous records MUST go to Human-in-the-Loop review; transfers require two-phase confirmation).
   - Limitation: Requires local hardware setup on workstation/server (min 8GB system RAM, though LIVA only consumes 680MB peak), cannot run as an instant public multi-tenant cloud SaaS due to Decree 13 compliance.
3. In Slide 8 (Traction) & Slide 14 (Current State) & Slide 19 (Risks):
   - Highlight the 0.2% exceptions (100 rows out of 50,000) as a feature of integrity, not hidden away.
   - Transparently list technical & regulatory risks and concrete mitigation steps.

Ensure all 25 slides, PPTX, and HTML uphold this radical honesty standard!

## 2026-09-14T12:19:19Z

Nâng cấp và hoàn thiện hệ sinh thái LIVA Banking Harness: xây dựng bộ công cụ MCP Native và hệ thống Agent Skills chuyên sâu phục vụ trọn gói các nghiệp vụ ngân hàng cốt lõi (đối soát sao kê tự động, quản trị ngân quỹ & ủy nhiệm chi Maker-Checker, tuân thủ AML/Nghị định 13, và thẩm định rủi ro tín dụng/thanh khoản).

Working directory: e:/Project/01_AI_Agents/LIVA_Banking
Integrity mode: development

## Requirements

### R1. Banking Operations MCP Tool Suite
Phát triển và mở rộng bộ công cụ Model Context Protocol (MCP) trong tầng lõi Native Rust, cung cấp các tools nghiệp vụ ngân hàng chuẩn hóa:
- **Đối soát & Bóc tách (Reconciliation)**: Phân tích sao kê ngân hàng (VCB, TCB, BIDV, chuẩn ISO 20022), đối chiếu 1-1 và 1-N với sổ cái ERP, phát hiện sai lệch số dư/phí ngầm.
- **Ngân quỹ & Thanh toán (Treasury & Payments)**: Lập lệnh ủy nhiệm chi (payment orders), quản lý hạn mức dòng tiền, cơ chế kiểm soát 2 vòng Maker-Checker trước khi ký lệnh.
- **Tuân thủ & Giám sát (Compliance & AML)**: Rà soát giao dịch theo quy định AML/CTF, phát hiện giao dịch đáng ngờ (STR), kiểm soát và ẩn danh hóa dữ liệu cá nhân theo Nghị định 13/2023/NĐ-CP.
- **Phân tích rủi ro & Tín dụng (Credit & Risk Scoring)**: Đánh giá sức khỏe tài chính doanh nghiệp, tính toán chỉ số thanh khoản (DSCR, Quick Ratio), cảnh báo rủi ro thâm hụt dòng tiền.

### R2. Standardized Banking Agent Skills
Xây dựng và hoàn thiện bộ kịch bản nghiệp vụ Agent Skills chuẩn hóa (`.agents/skills/liva-banking-*`) tương thích với Antigravity, Claude, Codex, bao gồm:
- Hướng dẫn ngữ cảnh (prompts, persona, workflows, input/output contracts) cho từng vai trò ngân quỹ.
- Kịch bản phối hợp đa tác tử (Orchestration): điều phối luồng bóc tách sao kê -> đối chiếu sai lệch -> tạo báo cáo chênh lệch -> chuyển Maker duyệt -> chuyển Checker xác nhận.
- Đầy đủ tài liệu định nghĩa SKILL.md với metadata, hướng dẫn xử lý biên, và quy trình xử lý ngoại lệ (Exception Handling).

### R3. Safe Sandbox & Resource Guardrails
Mọi tác vụ xử lý dữ liệu tài chính phải tuân thủ nghiêm ngặt nguyên tắc Zero Data Egress (chạy hoàn toàn cục bộ, không gửi dữ liệu nhạy cảm ra ngoài), giới hạn tài nguyên máy chủ (`-j 2`, RAM khả dụng >= 4GB), và tuân thủ ranh giới an toàn Git (chỉ staging `git add`, không tự ý commit/push).

## Acceptance Criteria

### Banking MCP Implementation
- [ ] Tất cả các MCP tools ngân hàng mới được định nghĩa schema chặt chẽ (JSON Schema) và đăng ký đầy đủ vào MCP Server của hệ thống.
- [ ] Các tools đối soát xử lý chính xác sai số số học tuyệt đối (0 sai số làm tròn, bảo toàn toàn vẹn số liệu tiền tệ).
- [ ] Chức năng Maker-Checker bắt buộc yêu cầu 2 trạng thái phê duyệt độc lập trước khi xuất lệnh thanh toán hợp lệ.
- [ ] Dữ liệu PII (số tài khoản, CCCD, tên khách hàng cá nhân) được tự động nhận diện và che giấu (masking) khi xuất ra báo cáo hoặc audit log.

### Banking Agent Skills
- [ ] Mỗi nghiệp vụ ngân hàng mới có thư mục skill hoàn chỉnh tại `.agents/skills/` với file `SKILL.md` hợp lệ (frontmatter, role, trigger phrases, detailed instructions, verification steps).
- [ ] Có kịch bản kiểm thử mẫu (playbook/example walkthrough) chứng minh Agent gọi đúng MCP tools tương ứng theo đúng ngữ cảnh yêu cầu.

### Verification & Quality Assurance
- [ ] Toàn bộ test suite liên quan (`cargo test -p liva-native-core -j 2 -- --test-threads 2`) biên dịch sạch sẽ và vượt qua 100% tests không có cảnh báo/lỗi nghiêm trọng.
- [ ] Bộ kịch bản demo hoặc benchmark chạy thành công, xác nhận tính năng hoạt động ổn định và đáp ứng thời gian thực.
## 2026-09-14T12:22:28Z

Redesign and elevate the **LIVA Banking Harness** 25-slide presentation deck for **INNOSTART 2026 Demo Day** into an ultra-compelling, visually stunning, easy-to-present startup pitch deck. The deck must prioritize human storytelling, intuitive business metaphors, crystal-clear value visualization (Before vs. After), and high-end corporate minimalist design matching the user's reference aesthetic, making it effortless for the speaker to pitch and captivating for non-technical judges.

Working directory: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_slides`
Integrity mode: demo

Reference material:
- User Design Reference: `C:/Users/Admin/.gemini/antigravity/brain/7b22ce54-7d30-45e5-aada-b91d4645f455/.user_uploaded/media_1789382596814.png`
- Working slide codebase: `e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_slides/`
- Existing Google Drive Target: `G:\My Drive\01_Du_An_Khoi_Nghiep\LIVA_InnoStart_2026\`

## Requirements

### R1. Storytelling & Narrative Flow ("Dễ hiểu - Dễ thuyết trình - Cuốn hút")
Transform every slide from a static report into an engaging story that anyone (even non-finance, non-tech judges) instantly grasps:
- **Relatable Hook & Conflict (Slides 1–3)**:
  - Open with a visceral everyday scene: Kế toán trưởng mở 15 file sao kê mỗi sáng, dò từng con số 1.000đ, mắt hoa và căng thẳng; CFO đối mặt nguy cơ trích nợ ngân hàng thất bại lúc 16h chiều thứ Sáu chỉ vì dòng tiền bị chậm đối soát 48 giờ.
  - The Legal Trap: Nhân viên vì quá tải đành copy dữ liệu nhạy cảm dán vào ChatGPT — vô tình phạm luật bảo vệ dữ liệu (Nghị định 13) với án phạt tới 5% doanh thu.
- **Crystal-Clear Solution Metaphor (Slides 4–6)**:
  - "LIVA như một Đai an toàn và Tháp canh nguồn vốn 24/7": Tự động bóc tách và khớp sổ phụ trong 30 giây; phát hiện nguy cơ thiếu tiền trước 24–48 tiếng để CFO chủ động điều vốn.
  - "Before vs. After" high-contrast visual comparison:
    * Trước khi có LIVA: 2–4 tiếng căng thẳng, dò tay từng dòng, mù mờ thanh khoản suốt 3 ngày.
    * Khi có LIVA: 30 giây xong toàn bộ, độ chính xác 99.8%, tiền đi đâu về đâu hiển thị tức thì trên 1 màn hình.
- **Simple, Common-Sense Technology Explanation (Slides 7–8)**:
  - Không nói thuật ngữ cao siêu; dùng nguyên lý bảo vệ 2 tầng dễ hiểu:
    1. *Tầng đọc hiểu thông minh*: AI đọc hiểu mọi loại file sao kê (kể cả scan mờ hay viết tắt lộn xộn).
    2. *Tầng khóa số học chuẩn xác*: Phép tính tiền do thuật toán chuyên dụng khóa cứng, đảm bảo đúng từng đồng từng cắc (tuyệt đối không để AI làm toán mò).
  - Vận hành 100% tại văn phòng doanh nghiệp, không gửi dữ liệu lên mạng, an toàn tuyệt đối.
- **Clear Business Opportunity & Roadmap (Slides 9–19)**:
  - Mô hình kinh doanh rõ ràng: Bán phần mềm cho ai? Thu tiền thế nào? Tại sao khách hàng sẵn sàng trả tiền ngay?
  - Kế hoạch 1 năm rõ ràng: từng quý làm gì, đo bằng gì, khi nào hòa vốn.
- **Confidence in Q&A Defense (Slides 20–25)**:
  - Bộ câu hỏi "chí mạng" mà BGK thường hỏi (Tại sao không dùng Excel? Nếu ngân hàng tự làm thì sao? Dùng mô hình nhỏ có thông minh bằng ChatGPT không?) cùng câu trả lời sắc bén, thuyết phục.

### R2. Visual Design & Presentation Elegance (Matching Reference Template)
- **Aesthetic Benchmark**: Pure white background (`#FFFFFF`), executive blues (`#0284C7`, `#1E3A8A`), slate text (`#0F172A`), spacious padding, zero clutter.
- **Card-Based Visual Scaffolding**:
  - Distinct 3-to-4 card grids with square colored icon headers.
  - High-impact Big Stat Callouts (e.g. `75%`, `24-48h`, `99.8%`, `30 Giây`, `5% Doanh thu`).
  - Chevron flow diagrams for step-by-step processes.
  - Native SVG / PowerPoint Donut charts for financial allocations and market breakdowns.
  - Professional architectural and business photographic accents (half-and-half layout).

### R3. Tri-Format Delivery & Google Drive Synchronization
- Deliver all 3 synchronized formats:
  1. `index.html`: Interactive web deck with full presenter notes (Key 'S'), slide overview grid (Key 'G'), full screen (F11), and 1-click clean PDF print.
  2. `liva_innostart_2026.pptx`: Native 16:9 PowerPoint file with editable shapes, vector charts, and presenter notes.
  3. `liva_innostart_2026.pdf`: Pristine 25-page 16:9 vector PDF export.
- Automatically sync updated files directly to `G:\My Drive\01_Du_An_Khoi_Nghiep\LIVA_InnoStart_2026\`.

## Acceptance Criteria

### Content & Storytelling Quality
- [ ] Every slide presents a clear, non-technical, human-relatable message with zero engineer jargon.
- [ ] Contains a dedicated "Before vs After" high-contrast comparison slide showing immediate tangible ROI.
- [ ] Speaker Notes contain conversational, ready-to-speak pitch scripts for each slide (matching a natural 5-minute pitch pace).
- [ ] Radical honesty maintained: Lab benchmark clearly labeled, 0.2% exceptions highlighted as safety feature, boundaries honestly stated.

### Design & Polish
- [ ] 100% matching the clean luxury white/blue template aesthetic from the user's reference image.
- [ ] All 25 slides rendered uniquely without formatting overlap or text clipping.
- [ ] Donut charts on Slides 12, 16, 24 render cleanly in both HTML and PPTX.

### Verification & Delivery
- [ ] PDF export contains 25 distinct pages with zero duplicates.
- [ ] PPTX file opens cleanly in PowerPoint and Google Slides.
- [ ] Google Drive folder contains the updated files with fresh timestamps.

## Follow-up — 2026-09-14T12:26:29Z

USER DIRECTIVE: ADOPT OPEN-SOURCE BEST PRACTICES (PPTAgent + SlideSage + SlideMason + Presenton + PPT Master)

The user provided key open-source reference patterns and constraints:
1. "Tôi thích màu sáng, clean và hiện đại, số slide tối đa khoảng 30 slide" (Light theme, clean, modern, human-designed look, max ~30 slides e.g. 26-28 slides).
2. Incorporate lessons from the top 5 open-source slide projects:
   - **SlideSage Storytelling**: Use "Action Titles" (e.g. "CƠN ÁC MỘNG 4 GIỜ ĐỐI SOÁT & VÙNG MÙ THANH KHOẢN T+3" instead of dry generic titles), narrative sequence (Opening Hook -> Context -> Pain -> Before vs After -> Solution -> Proof -> Roadmap -> Ask), and clear transition beats in Presenter Notes.
   - **SlideMason Primitives**: Structure slides with clean reusable design primitives:
     * `StatBox`: Big number callout + concise badge label + 1-line impact.
     * `CompareGrid`: High-contrast "Trước khi có LIVA" (Đỏ/Xám - 4h căng thẳng, sai lệch, phạt 5% doanh thu) vs "Khi có LIVA" (Xanh Emerald/Blue - 30 giây, 99.8% tự động, an toàn 100% tại chỗ).
     * `StepChevron`: Clean horizontal 4-5 step pipeline with subtle arrows.
     * `FeatureCards`: 3-4 cards with square colored icon headers.
     * `DonutChart`: Native vector donut chart with center metric and clear legend.
   - **Presenton & PPT Master Editable Standards**: Clean light theme (`#FFFFFF`, `#F8FAFC`, `#0284C7`, `#1E3A8A`, `#0F172A`), native editable shapes/tables/charts in PPTX, zero cluttered text.
   - **PPTAgent Reflection**: Multi-step verification ensuring design elegance, coherence, and storytelling flow.

Ensure the final deck stays around 26-28 slides (strictly <= 30 slides), renders cleanly in index.html, liva_innostart_2026.pptx, and liva_innostart_2026.pdf, and syncs to Google Drive.

## Follow-up — 2026-09-14T12:43:14Z

CRITICAL USER DIRECTIVE: ABSOLUTE PARITY ACROSS ALL 3 FORMATS (PPTX, HTML, PDF)

The user explicitly requested: "các pptx html pdf phải giống nhau nhé" (PPTX, HTML, and PDF must be completely identical to each other).

Mandatory rules for verification and quality gate:
1. 100% Structural Parity: Exactly 27 slides in identical sequence across `index.html`, `liva_innostart_2026.pptx`, and `liva_innostart_2026.pdf`.
2. 100% Verbatim Text Parity: All slide titles (Action Titles), subtitles, category badges, card bullets, KPI stat boxes, and key takeaway footers must match word-for-word between PPTX and HTML/PDF.
3. 100% Speaker Notes Parity: Every slide's speaker notes in `index.html` (accessible via 'S') must be identical to the presenter notes embedded in `liva_innostart_2026.pptx`.
4. 100% Visual Parity: The visual layout (CompareGrid Before/After, StatBoxes, StepChevrons, DonutCharts on slides 12, 16, 24) must match in structure and visual hierarchy across all formats.
5. Automated Parity Check: Ensure the challenger and auditor run an automated cross-format parity script (comparing slide titles, notes, and metrics between HTML and PPTX/PDF) before declaring victory!

Enforce this strictly across the active workers and review gates.

## Follow-up — 2026-09-14T14:12:23Z

<USER_REQUEST>
Nghiên cứu kiến trúc hiện tại của LIVA Banking nhằm đánh giá tính khả thi, phân tích rủi ro, phát hiện lỗi tương thích và xây dựng thiết kế kiến trúc chuyển đổi toàn diện sang mô hình Hybrid Web & Desktop.

Working directory: teamwork_projects/liva_bank_web_migration
Integrity mode: development

## Requirements

### R1. Deep Architectural Gap & Runtime Audit
Rà soát toàn diện hiện trạng mã nguồn của LIVA Banking (`liva-ui`, `liva-native-core`, `liva-desktop`), lập danh mục chi tiết các điểm phụ thuộc chặt vào Tauri runtime (`invokeBackend`, `@tauri-apps/api/core`, OS file paths, Windows Keystore/DPAPI), và chỉ ra mọi lỗi tiềm ẩn (runtime exceptions, broken state, file upload limitations) khi chạy ứng dụng trên trình duyệt web tiêu chuẩn.

### R2. Hybrid Architecture Blueprint (Web + Desktop)
Thiết kế kiến trúc hệ thống tổng thể hỗ trợ song song hai nền tảng (Desktop Tauri và Web Browser):
- Đặc tả giao thức truyền thông client-server (REST API, WebSocket/SSE cho streaming).
- Thiết kế lớp trừu tượng `PlatformAdapter` hoàn chỉnh cho frontend (phân tách rõ ràng giữa `TauriAdapter` và `WebAdapter`).
- Xây dựng mô hình ingestion sao kê ngân hàng qua Web (multipart upload streaming thay vì đọc file path cục bộ).

### R3. Banking Security, Compliance & Data Protection Analysis
Phân tích và thiết kế cơ chế bảo mật cho phiên bản Web nhằm đáp ứng nghiêm ngặt các quy định ngân hàng:
- Cơ chế bảo vệ dữ liệu khách hàng theo Nghị định 13/2023/NĐ-CP và nguyên tắc Zero Data Egress (mã hóa đường truyền TLS 1.3, CSP, CORS, CSRF, Secure HttpOnly Cookie / Session tokens).
- Cơ chế kiểm soát kép Maker-Checker (Thông tư 09/2020/TT-NHNN) và ghi nhật ký kiểm toán bất biến (tamper-evident audit log).
- Đề xuất giải pháp lưu trữ bí mật (credentials, API keys) an toàn thay thế cho OS Keystore của Desktop.

### R4. Migration Roadmap & Verification Strategy
Lập kế hoạch chuyển đổi phân kỳ (phases) với các mốc kiểm thử định lượng:
- Ma trận đối chiếu API giữa các lệnh Tauri IPC hiện hữu và Web endpoints.
- Chiến lược kiểm thử tự động (Unit test, Integration test, E2E test, Security penetration test).
- Dự báo rủi ro triển khai và kế hoạch dự phòng (rollback / fallback).

## Acceptance Criteria

### Audit Report & Vulnerability Analysis
- [ ] Báo cáo phân tích rủi ro chỉ rõ 100% các điểm đứt gãy khi chạy trên Web mà không có môi trường Tauri (được kiểm chứng bằng phân tích mã nguồn thực tế).
- [ ] Đánh giá chi tiết các giới hạn của trình duyệt đối với việc đọc sao kê (tệp lớn VCB, TCB, BIDV, ISO 20022) kèm giải pháp xử lý.
- [ ] Phân tích rủi ro an toàn thông tin theo Nghị định 13/2023/NĐ-CP và Thông tư 09/2020/TT-NHNN trên môi trường Web.

### Architectural Specification & Blueprint
- [ ] Bản thiết kế kiến trúc chi tiết (Mermaid diagrams) thể hiện luồng dữ liệu giữa Web UI, Backend Gateway trong Rust, và SQLite WAL Database.
- [ ] Đặc tả đầy đủ OpenAPI/REST và WebSocket schema thay thế cho toàn bộ các lệnh Tauri IPC của banking module.
- [ ] Thiết kế kiến trúc `IPlatformAdapter` đa nền tảng cho `liva-ui`, đảm bảo không làm gãy tính năng Desktop hiện có.

### Roadmap & Quality Gate
- [ ] Kế hoạch chuyển đổi theo lộ trình từng giai đoạn kèm ước lượng nỗ lực và thứ tự ưu tiên.

## Follow-up — 2026-09-15T04:29:31Z

<USER_REQUEST>
Transform LIVA Banking into a specialized multi-role portal for commercial bank staff, remove Demo Day tour, and enable full Client-Server JWT multi-machine synchronization.

Working directory: teamwork_projects/liva_banking_universal
Integrity mode: development

## Requirements

### R1. Specialized Role-Based Portals for Bank Employees (Cổng Tác Nghiệp Theo Phân Quyền)
Restructure the application away from a generic personal treasury dashboard into a dedicated workspace for bank staff, with dedicated views and permissions tailored to each specific banking role:
- **Cán Bộ Vận Hành & Đối Soát Thanh Toán (Operations & Settlement Specialist / Maker)**:
  - Tiếp nhận sao kê liên ngân hàng (VCB, TCB, BIDV, Citad, Napas, ISO 20022).
  - Khởi chạy engine đối soát 3 tầng tự động (Khớp 1:1, Heuristic thời gian/nội dung, và Subset-Sum chia tách 1:N / N:1).
  - Bóc tách sai lệch phí giao dịch ẩn và quản lý hàng đợi giao dịch treo/chưa khớp (Exception & Unmatched Resolution Queue).
- **Kiểm Soát Viên Phê Duyệt Lệnh (Operations Supervisor / Checker)**:
  - Thực thi quy trình kiểm soát kép (Maker-Checker) tuân thủ Thông tư 09/2020/TT-NHNN.
  - Thẩm định hồ sơ chứng từ, xác thực OTP/Chữ ký số, duyệt hoặc từ chối lệnh chi với biên bản giải trình.
  - Đóng dấu băm mật mã Merkle Tree SHA-256 vào sổ cái kiểm toán bất biến (Forward Audit Ledger).
- **Cán Bộ Giám Sát Tuân Thủ & Phòng Chống Rửa Tiền (Compliance & AML/CFT Specialist)**:
  - Giám sát luồng tiền nghi vấn theo Thông tư 09/2023/TT-NHNN và Luật PCRT 2022.
  - Phân tích rủi ro: Chia nhỏ dòng tiền né ngưỡng 400M (Structuring/Smurfing), tài khoản trung chuyển vốn thần tốc (Rapid Pass-through Churn), giao dịch đêm bất thường (Night Velocity).
  - Tích hợp System Prompt Inspector để thẩm tra logic suy luận của AI và xuất hồ sơ Báo cáo Giao dịch Đáng ngờ (Form STR chuẩn Cục PCRT - NHNN).
- **Cán Bộ Quản Trị Thanh Khoản & Ngân Quỹ (Treasury & Liquidity Desk)**:
  - Giám sát vị thế thanh khoản tập trung đa tài khoản ngân hàng, dòng tiền vào/ra ròng (Net Inflow/Outflow).
  - Quản trị hạn mức rủi ro thanh khoản và cân đối nguồn vốn trong ngày/qua đêm.

### R2. Complete Removal of Demo Day Elements & Enterprise Hardening
- Loại bỏ hoàn toàn nút nổi "Trình Diễn Demo Day (5 Phút)", overlay kịch bản giới thiệu và các nút tour demo trên thanh điều hướng.
- Chuyển đổi giao diện sang phong cách Cổng tác nghiệp Ngân hàng chuyên nghiệp (Core Banking Operations Workstation).
- Bổ sung thanh trạng thái định danh cán bộ (Officer ID, Mã chi nhánh, Phiên làm việc bảo mật) và thời gian phiên đăng nhập.

### R3. Client-Server Backend & JWT Authentication Synchronization
- Tích hợp Backend API Server (`server.mjs`) cung cấp cơ chế xác thực JWT Bearer Token.
- Lưu trữ `liva_auth_token` trực tiếp trong `localStorage` và gửi qua header `Authorization: Bearer <token>`.
- Đồng bộ hóa dữ liệu thời gian thực giữa các máy tính khác nhau: Khi cán bộ Maker tạo lệnh trên máy A, Kiểm soát viên Checker trên máy B sẽ nhìn thấy ngay lập tức để phê duyệt.
- Cơ chế phục hồi Dual-Mode: Tự động chạy với Server khi Online và chuyển sang Local-First Standalone khi Offline (Zero Cloud Egress theo Nghị định 13/2023/NĐ-CP).

## Acceptance Criteria

### Bank Staff Portals & Functional Workflows
- [ ] Giao diện có thanh chuyển đổi vai trò cán bộ ngân hàng (Vận hành/Maker, Kiểm soát/Checker, Thẩm định AML, Quản trị vốn).
- [ ] Mỗi vai trò có trang nghiệp vụ chuyên biệt với các quyền thao tác tương ứng (RBAC).
- [ ] Toàn bộ các nút, banner và overlay liên quan đến "Demo Day (5 Phút)" bị loại bỏ 100%.
- [ ] Token JWT được cấp và lưu vào `localStorage.getItem('liva_auth_token')` sau khi đăng nhập.
- [ ] Thao tác tạo lệnh ở quyền Maker và duyệt lệnh ở quyền Checker đồng bộ qua Backend API Server.
- [ ] Engine đối soát 3 tầng, module AML/STR và sổ cái Merkle Tree hoạt động chính xác không có lỗi suy diễn.
- [ ] Mã nguồn biên dịch sạch sẽ (`npm run build`), vượt qua toàn bộ các bài kiểm thử tự động.
</USER_REQUEST>

## Follow-up — 2026-09-15T05:08:43Z

<USER_DIRECTIVE>
[USER DIRECTIVE & PROMPT APPROVED]
The user has reviewed and approved the updated prompt_draft.md.
Specification confirmed:
1. Internal Commercial Bank Operations Platform (White-label Universal Core, not a corporate client multi-bank dashboard, no single-bank hardcoding).
2. Interbank Clearing & Settlement Channels: CITAD (NHNN), NAPAS 24/7, Song phương / Vostro-Nostro, SWIFT.
3. 3-Tier Reconciliation between Core Banking internal ledger and Interbank clearing statements, including fee discrepancy separation and dispute/exception handling.
4. Automated Anomaly & Fraud Detection (AML/STR per Circular 09/2023/TT-NHNN: Structuring < 400M, Rapid pass-through churn, Night velocity, clearing discrepancy) with automated Form STR generation for Cục PCRT - NHNN.
5. Maker-Checker dual control workflow (Circular 09/2020/TT-NHNN) for liquidity transfers, settlement adjustments, and high-value payments.
6. Local AI Qwen 14B on GPU with strict Database Safeguard (Read-Only, strictly no DELETE/DROP/WIPE).

All 309 unit tests and 236 E2E tests are passing, and the build succeeds. Please proceed through Milestone M2 and subsequent milestones.
</USER_DIRECTIVE>

## 2026-09-15T06:52:49Z

<USER_REQUEST>
Phân rã cấu trúc công việc (Work Breakdown Structure - WBS) cấp task chi tiết cho GĐ0 (Khởi động & Khám phá, Tuần 1–4) và GĐ1 (Nền tảng lõi & Sandbox Zero-Egress, Tuần 3–10) của dự án LIVA Banking Harness, bao gồm ước lượng ngày công, ma trận phân vai, luồng phụ thuộc và tiêu chí nghiệm thu kỹ thuật.

Working directory: e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_harness
Integrity mode: development

## Requirements

### R1. Phân rã WBS cấp task cho GĐ0 (Khởi động & Khám phá)
Phân rã chi tiết toàn bộ các hạng mục GĐ0 thành danh mục task có mã WBS chuẩn hóa (WBS 0.1 .. 0.x). Mỗi task bắt buộc có:
- Mã task & Tên task
- Mô tả chi tiết phạm vi công việc
- Vai trò phụ trách chính và phối hợp (dựa trên đội ngũ 8-9 FTE: Solution Architect, Rust Engineer, AI Engineer, Full-stack, Security, QA, Kế toán trưởng, Tư vấn pháp lý)
- Ước lượng ngày công (man-days)
- Task phụ thuộc (Dependencies)
- Sản phẩm bàn giao (Deliverable)
- Tiêu chí nghiệm thu (Acceptance Criteria)

Các luồng bắt buộc gồm:
1. Phỏng vấn nghiệp vụ kế toán/thủ quỹ & chuẩn hóa user journey.
2. Thu thập và ẩn danh tập golden files (≥ 3 ngân hàng + CAMT.053).
3. Rà soát căn cứ pháp lý (NĐ 13/2023, TT 09/2020, Luật PCTL 2022, nghĩa vụ DPIA).
4. Khảo sát định dạng tích hợp ERP (MISA, FAST, Bravo, SAP).
5. Đánh giá lựa chọn công nghệ lõi Rust, Database (SQLite vs PostgreSQL), UI framework, và shortlist SLM kèm rà soát bản quyền thương mại.
6. Mô hình hóa mối đe dọa STRIDE + Data Flow Diagram (DFD).
7. Thiết lập hạ tầng dev nội bộ 100% self-hosted (GitLab CE, CI runner, chat, issue tracker).
8. Xây dựng công cụ sinh dữ liệu tổng hợp (Synthetic Data Generator v0) cho giao dịch ngân hàng VN (VietQR, Napas 247).

### R2. Phân rã WBS cấp task cho GĐ1 (Nền tảng lõi & Sandbox Zero-Egress)
Phân rã chi tiết toàn bộ các hạng mục GĐ1 thành danh mục task có mã WBS chuẩn hóa (WBS 1.1 .. 1.x) với cùng cấu trúc thông tin như R1.
Các luồng bắt buộc gồm:
1. Thiết kế và phát triển crate `liva-money`: kiểu dữ liệu `Money` trên `i64`, quy tắc làm tròn VND, zero-float clippy enforcement.
2. Thiết kế và phát triển crate `liva-ledger`: state machine kế toán, event store append-only, bất biến `Closing = Opening + ΣCredit − ΣDebit`.
3. Bộ kiểm thử dựa trên thuộc tính (Property-based test / proptest) đạt ≥ 1.000.000 test cases fuzzing không vi phạm bất biến.
4. Xây dựng `liva-netguard`: cơ chế sandbox bằng nftables, seccomp, systemd unit; bộ kiểm thử tự động xác minh chặn 100% egress ngoài loopback.
5. Định nghĩa Canonical Transaction Schema nội bộ và trait `BankParser` cho plugin registry.
6. Bộ khung ứng dụng Local Web UI: chỉ lắng nghe `127.0.0.1`, xác thực cục bộ và phân quyền RBAC (Maker, Checker, Admin).

### R3. Ma trận tổng hợp phân bổ nguồn lực & đối soát Exit Criteria
1. Bảng tổng hợp phân bổ ngày công (Man-Days Rollup) cho GĐ0 và GĐ1 theo từng vai trò nhân sự, đối chiếu với tổng công suất đội ngũ (8-9 FTE).
2. Ma trận ánh xạ (Traceability Matrix) liên kết 1:1 toàn bộ 4 exit criteria của GĐ0 và 3 exit criteria của GĐ1 với các task nghiệm thu cụ thể, đảm bảo không có tiêu chí nào bị bỏ trống.

## Acceptance Criteria

### Tính đầy đủ của WBS
- [ ] Tất cả 8 luồng công việc của GĐ0 và 6 luồng công việc của GĐ1 đều được phân rã thành các task khả thi (thời lượng 1–5 man-days mỗi task).
- [ ] 100% task có đầy đủ: Mã WBS, Tên task, Vai trò phụ trách, Ước lượng ngày công, Deliverable, Tiêu chí nghiệm thu.
- [ ] Không có task nào gán vai trò nằm ngoài danh sách đội ngũ 8-9 FTE quy định.

### Tính chặt chẽ về mặt kỹ thuật & bảo mật
- [ ] Quy định rõ ràng cổng CI/CD clippy deny float cho module kế toán trong task phát triển `liva-money`.
- [ ] Định nghĩa tiêu chí nghiệm thu cụ thể cho bài kiểm thử 1.000.000 fuzzing case bất biến kế toán của `liva-ledger`.
- [ ] Quy định cụ thể kịch bản kiểm thử tự động cho `liva-netguard` (thực hiện kết nối TCP/UDP ra ngoài loopback và khẳng định 100% bị từ chối).

### Cân đối nguồn lực và Ma trận nghiệm thu
- [ ] Bảng tổng hợp phân bổ man-days theo từng vai trò phản ánh đúng thời gian biểu (GĐ0: 4 tuần; GĐ1: 8 tuần gối sóng).
- [ ] Ma trận Traceability đối soát đầy đủ 100% Exit Criteria của GĐ0 (SRS/HĐTK duyệt, Golden files ≥ 3 ngân hàng, Threat model duyệt, Môi trường dev sẵn sàng) và GĐ1 (Demo ledger CRUD + invariant pass, Egress test 100% block, CI/CD xanh).
- [ ] Văn bản được lưu thành file tài liệu hoàn chỉnh tại thư mục làm việc quy định `teamwork_projects/liva_banking_harness/WBS_PHASE_0_1.md`.

</USER_REQUEST>

## 2026-09-15T08:36:41Z

<USER_REQUEST>
Triển khai hoàn thiện các cấu phần cốt lõi của hệ thống LIVA Banking Harness theo lộ trình Giai đoạn 0 & Giai đoạn 1: bao gồm công cụ sinh dữ liệu giao dịch tổng hợp `tools/synthgen`, cơ chế an ninh Zero-Egress `crates/liva-netguard`, và chuẩn hóa Canonical Transaction Schema cùng trait `BankParser` và động cơ đối soát 3 tầng `crates/liva-recon`.

Working directory: e:/Project/01_AI_Agents/LIVA_Banking
Integrity mode: development

## Requirements

### R1. Công cụ sinh dữ liệu tổng hợp `tools/synthgen` (GĐ0 - Luồng 0.8)
Xây dựng công cụ dòng lệnh (CLI binary) `tools/synthgen` độc lập:
- Sinh tập dữ liệu giao dịch ngân hàng Việt Nam ngẫu nhiên có kiểm soát (deterministic PRNG với seed cấu hình được).
- Hỗ trợ phân vùng đa ngân hàng (VCB, TCB, BIDV, CTG, MB, VBA) với các trường đặc thù: mã tham chiếu FT/Trace ID, nội dung giao dịch chứa cú pháp VietQR, Napas 24/7, số hóa đơn VAT, tên công ty tiếng Việt thực tế.
- Khả năng xuất định dạng CSV và JSON theo cấu trúc sao kê từng ngân hàng và bảng hóa đơn nội bộ.
- Tích hợp kiểu `liva-money::Money`, đảm bảo tính bảo toàn số học số dư `Closing = Opening + ΣCredit - ΣDebit`.

### R2. Crate an ninh Zero-Egress `crates/liva-netguard` (GĐ1 - Luồng 1.4)
Xây dựng crate `crates/liva-netguard` thực thi rào chắn an ninh mạng on-premise:
- Cung cấp profile cấu hình an ninh: nftables ruleset và seccomp filter sandbox cho Linux daemon.
- Cung cấp module kiểm toán mạng cục bộ (Local Egress Verifier) giám sát socket connections.
- Bộ kiểm thử tự động xác minh: bất kỳ nỗ lực mở socket TCP/UDP ra ngoài địa chỉ loopback `127.0.0.1` đều bị phát hiện hoặc chặn 100%.

### R3. Crate Canonical Schema & Bank Parser `crates/liva-parse` (GĐ1 - Luồng 1.5)
Xây dựng crate `crates/liva-parse` định nghĩa chuẩn giao dịch nội bộ:
- Định nghĩa `CanonicalTransaction` và `BankStatement` chuẩn hóa, sử dụng kiểu `Money` từ `liva-money`.
- Định nghĩa trait `BankParser` với các hàm `sniff(&self, raw: &[u8], filename: &str) -> bool` và `parse(&self, raw: &[u8], filename: &str) -> Result<BankStatement, ParseError>`.
- Triển khai parser cho ISO 20022 CAMT.053 (XML) và parser mẫu cho Vietcombank (VCB CSV/Excel) & Techcombank (TCB CSV).
- Kiểm tra tính bất biến số dư sao kê tự động thông qua `liva-ledger`.

### R4. Crate Động cơ đối soát 3 tầng `crates/liva-recon` (GĐ3)
Xây dựng crate `crates/liva-recon` tích hợp `liva-money` và `liva-ledger`:
- **Tier 1 (Exact Hash Matcher)**: Khớp 1:1 chính xác mã tham chiếu + số tiền trong cửa sổ thời gian ±24h.
- **Tier 2 (Fuzzy Heuristic Matcher)**: Chuẩn hóa tiếng Việt (loại bỏ dấu, token sort) + khoảng cách Jaro-Winkler ≥ 0.85, tách và khấu trừ phí Napas/ngân hàng có thể cấu hình.
- **Tier 3 (Composite Split Solver)**: Giải thuật quy hoạch động / Branch-and-Bound subset-sum (1:N và N:1) bảo toàn tuyệt đối số tiền ($\Delta = 0$).
- **HITL Quarantine Queue**: Cách ly các giao dịch không khớp hoặc có độ lệch, sinh token xác thực UUIDv4 với TTL 15 phút phục vụ phê duyệt hai pha (Maker-Checker).

## Acceptance Criteria

### Kiểm định chất lượng mã nguồn & Biên dịch
- [ ] Toàn bộ các crate và tool (`tools/synthgen`, `crates/liva-netguard`, `crates/liva-parse`, `crates/liva-recon`) được đăng ký hợp lệ trong workspace root `Cargo.toml`.
- [ ] `cargo check -j 2` trên toàn bộ workspace thoát mã 0 (không lỗi biên dịch).
- [ ] Tất cả crate tính toán tài chính (`liva-money`, `liva-ledger`, `liva-recon`, `liva-parse`) tuân thủ nghiêm ngặt `#![deny(clippy::float_arithmetic)]`, kiểm tra bằng `cargo clippy -j 2` không phát sinh cảnh báo float.

### Kiểm thử chức năng & Tính bất biến
- [ ] `tools/synthgen` chạy tạo được file mẫu sao kê 1.000+ giao dịch với tỷ lệ cân đối số dư 100%.
- [ ] `crates/liva-netguard` vượt qua unit test kiểm tra cơ chế chặn egress ngoài loopback.
- [ ] `crates/liva-parse` parse chính xác file CAMT.053 XML và CSV ngân hàng mẫu.
- [ ] `crates/liva-recon` chạy qua bộ test đối soát 3 tầng (Tier 1, Tier 2, Tier 3) với tỷ lệ khớp chính xác và hàng đợi HITL cách ly đúng quy cách.
- [ ] `cargo test -j 2 -- --test-threads 2` vượt qua 100% unit tests và proptests trên các crate mới.
</USER_REQUEST>

## 2026-09-15T15:49:02Z

<USER_REQUEST>
Rà soát toàn diện, đồng bộ hóa và nâng cấp hệ thống tài liệu kỹ thuật của dự án LIVA Banking. Cập nhật rõ ràng mô hình triển khai kép: Bản Client-Only Web Demo (Zero-Backend, chạy 100% trên trình duyệt) phục vụ trải nghiệm và trình diễn, song song với Bản Production Hybrid On-Premise (Rust Native Core) phục vụ vận hành thực tế; bổ sung cẩm nang hướng dẫn triển khai mạng nội bộ cô lập (LAN/Intranet/VPN) và bộ câu hỏi phản biện kỹ thuật; đảm bảo 100% tính toàn vẹn và sạch lỗi liên kết.

Working directory: e:/Project/01_AI_Agents/LIVA_Banking
Integrity mode: development

## Requirements

### R1. Xây dựng Cẩm nang Triển khai Chuyên sâu (`docs/02-van-hanh/07-trien-khai-web-client-va-mang-noi-bo.md`)
Biên soạn tài liệu kỹ thuật hoàn chỉnh hướng dẫn hai bài toán cốt lõi:
1. **Triển khai Web Demo Độc lập (Client-Only)**: Hướng dẫn chi tiết từng bước đưa thư mục `teamwork_projects/liva_banking_universal/dist` hoặc gói `LIVA_Banking_Web_Demo.zip` lên các nền tảng hosting tĩnh (Netlify Drop, Vercel, Cloudflare Pages, Nginx, Local Static Server), cơ chế xử lý file sao kê hoàn toàn phía client không phụ thuộc backend.
2. **Cơ chế Bảo mật & Cách ly Mạng Nội bộ (Intranet Access Control)**:
   - *Phương án 1 (LAN / Intranet On-Premise)*: Hướng dẫn bind port nội bộ RFC 1918, ngắt cổng port-forwarding trên router văn phòng, cơ chế chỉ máy kết nối Wi-Fi/dây mạng nội bộ mới truy cập được.
   - *Phương án 2 (VPN Nội bộ / Mesh VPN)*: Cấu hình mạng riêng ảo (Tailscale, WireGuard) cho kịch bản nhân viên làm việc từ xa (WFH) kết nối an toàn vào mạng công ty.
   - *Phương án 3 (IP Whitelisting & Cloudflare Zero Trust)*: Thiết lập tường lửa lọc IP Public tĩnh của văn phòng và cổng xác thực email doanh nghiệp nếu triển khai trên Cloud.
3. **Kịch bản Thuyết trình Nhanh (Presenter Cheat-Sheet)**: Tóm tắt 3 ý cốt lõi bằng ngôn ngữ súc tích, chuyên nghiệp để người trình bày tự tin trả lời hội đồng và đối tác về cách vận hành mạng nội bộ và an toàn dữ liệu.

### R2. Chuẩn hóa và Đồng bộ `README.md` & `PROJECT.md`
Cập nhật tài liệu mặt tiền của dự án phản ánh chính xác hiện trạng:
1. Bổ sung bảng ma trận so sánh trực quan giữa hai phiên bản: **Web Demo Tức thì (Universal Client-Side)** vs **Bản Doanh nghiệp Đầy đủ (Rust Native Core On-Premise)** về mục đích sử dụng, công nghệ, yêu cầu hạ tầng và mức độ an toàn.
2. Cập nhật mục **Quick Start (Khởi động Nhanh)**: Bổ sung chỉ dẫn 1 lệnh để chạy ngay Web Demo từ bản build tĩnh hoặc mở file `index.html`.
3. Khắc phục các đoạn mô tả cũ hoặc gây hiểu nhầm về sự phụ thuộc backend đối với bản web demo, đồng thời khẳng định rõ ràng ranh giới bảo mật Nghị định 13/2023/NĐ-CP và Thông tư 09/2020/TT-NHNN.

### R3. Cập nhật Kịch bản Trình diễn `DEMO_PLAYBOOK_BANKING.md` & Tài liệu Vận hành `docs/02-van-hanh/03-trien-khai-va-runtime.md`
1. Bổ sung mục hỏi đáp phản biện chuyên sâu (Q&A Defense) trong `DEMO_PLAYBOOK_BANKING.md` cho các câu hỏi giám khảo thường gặp:
   - *"Trang web này có gửi dữ liệu sao kê tài chính của khách hàng lên máy chủ bên thứ ba không?"* (Giải thích cơ chế xử lý 100% trong RAM trình duyệt).
   - *"Làm thế nào để chỉ nhân viên ngồi trong văn phòng mới mở được hệ thống?"* (Giải thích cơ chế IP LAN và cách ly mạng).
2. Đồng bộ hóa `docs/02-van-hanh/03-trien-khai-va-runtime.md` để kết nối liên kết tới tài liệu triển khai mới `07-trien-khai-web-client-va-mang-noi-bo.md`.

### R4. Kiểm tra Toàn vẹn Tài liệu (Documentation Quality Gate)
1. Chạy công cụ kiểm tra tài liệu tự động của dự án `node scripts/docs-check.mjs`.
2. Đảm bảo toàn bộ các tài liệu mới và sửa đổi đều có Front-matter YAML hợp lệ, các liên kết tương đối trỏ chính xác, các khối sơ đồ Mermaid đóng mở chuẩn xác, không có liên kết đứt gãy (0 dead links, exit code 0).

## Acceptance Criteria

### Tính Đầy đủ & Chuẩn xác của Nội dung
- [ ] Tệp `docs/02-van-hanh/07-trien-khai-web-client-va-mang-noi-bo.md` tồn tại, chứa đầy đủ hướng dẫn deploy client tĩnh, 3 phương án mạng nội bộ (LAN, VPN, IP Whitelist) và cheat-sheet thuyết trình.
- [ ] `README.md` và `PROJECT.md` có bảng so sánh rõ ràng giữa Web Demo Client-Only và Production On-Premise, cùng hướng dẫn chạy nhanh Web Demo trong vòng 1 phút.
- [ ] `DEMO_PLAYBOOK_BANKING.md` có thêm phần giải đáp phản biện về an toàn mạng nội bộ và tính độc lập của client-side web demo.
- [ ] `docs/02-van-hanh/03-trien-khai-va-runtime.md` được cập nhật liên kết đồng bộ đến tài liệu triển khai mới.

### Cổng Kiểm định Chất lượng Tự động
- [ ] Lệnh `node scripts/docs-check.mjs` chạy thành công với mã thoát 0 (exit code 0), không có lỗi cú pháp hoặc liên kết đứt gãy.
- [ ] Không có tệp tài liệu nào chứa placeholder chưa hoàn thiện (như TODO, TBD, placeholder text).

</USER_REQUEST>

## 2026-09-15T17:34:07Z

<USER_REQUEST>
Remake toàn diện các thành phần lõi của dự án LIVA Banking (`crates/liva-parse`, `crates/liva-recon`, `crates/liva-ledger`) thành kiến trúc hoàn toàn hướng dữ liệu (data-driven / DSL / configuration-driven), xóa bỏ 100% các giá trị, định dạng và quy tắc fix cứng (hardcode). Toàn bộ hệ thống phải nạp động và diễn giải quy tắc từ tệp cấu hình DSL (YAML/JSON) trong khi vẫn bảo toàn tuyệt đối 5 nguyên tắc cốt lõi: Math Engine số nguyên bất biến (`u64/u128`), Zero-Egress (hoàn toàn offline/air-gap), Merkle Audit Ledger, Segregation of Duties (Maker-Checker), và cơ chế cách ly HITL.

Working directory: e:/Project/01_AI_Agents/LIVA_Banking
Integrity mode: development
Requested team: Dual-Swarm Banking (Dev Swarm kỹ thuật + Persona Simulation Swarm kiểm thử đối kháng Maker-Checker như chuẩn LIVA Harness)

## Requirements

### R1. Dynamic Ingestion & Parser DSL Engine (`liva-parse`)
Xây dựng một kiến trúc parser cấu hình động thay thế cho các struct parser cứng:
- Định nghĩa cấu hình mẫu sao kê (Statement Template DSL) bằng YAML/JSON mô tả quy tắc nhận diện (sniffing headers/keywords), phân định cấu trúc cột (mapping ngày tháng, số tiền, số dư sau giao dịch, nội dung, bên đối tác), định dạng ngày và số tiền, cùng logic phân tách số dư đầu kỳ/cuối kỳ.
- Parser Engine thông dịch tệp cấu hình DSL để tự động phân tích các tệp sao kê ngân hàng (CSV, XLSX, XML/CAMT.053) mà không cần viết thêm mã nguồn Rust khi bổ sung ngân hàng mới.
- Hỗ trợ nạp động nhiều tệp template ngân hàng từ một thư mục cấu hình và tự động chọn template phù hợp thông qua cơ chế sniffing linh hoạt.

### R2. Declarative Reconciliation & Rule Engine (`liva-recon`)
Xóa bỏ toàn bộ các hằng số và mảng cố định trong logic đối soát, chuyển đổi sang Rule Engine động:
- Cấu hình động cho Tier 1: Cửa sổ thời gian dung sai (±N giờ/ngày), quy tắc chuẩn hóa mã chứng từ/ref code.
- Cấu hình động cho Tier 2: Danh sách stop-words và tiền tố pháp nhân (CONG TY, TNHH, CP...), thuật toán đo độ tương đồng văn bản cùng ngưỡng chấp nhận (threshold), danh mục các dải phí ngân hàng hợp lệ được cấu hình động từ bên ngoài thay vì mảng hằng số cứng, và công thức tính điểm tin cậy (confidence score basis points).
- Cấu hình động cho Tier 3: Giới hạn tập con $k \le 8$, độ sâu tìm kiếm, chiến lược cắt tỉa và thời gian ngân sách tối đa (timeout) cho bài toán subset-sum.
- Cơ chế HITL Quarantine: Cấu hình động thời hạn sống của token (TTL), quy tắc phân loại mức độ rủi ro và điều kiện kích hoạt phê duyệt hai pha.

### R3. Dynamic Chart of Accounts (COA) & Posting Engine (`liva-ledger`)
Xây dựng công cụ ánh xạ bút toán và sơ đồ tài khoản động:
- Cho phép cấu hình sơ đồ tài khoản (Thông tư 200/2014, Thông tư 133/2016 hoặc COA tùy chỉnh) từ tệp cấu hình.
- Định nghĩa quy tắc sinh bút toán tự động (Journal Posting Rules) dựa trên loại giao dịch, ngân hàng, loại phí và đối tác đối ứng.
- Tự động bóc tách phí ngân hàng và định tuyến tài khoản chi phí (6425, 6427, 635) hoàn toàn theo cấu hình mà không hardcode mã tài khoản.

### R4. Dynamic Policy Engine for AML & Maker-Checker Policies
- Khai báo các quy tắc giám sát phòng chống rửa tiền (AML) qua tệp cấu hình (ngưỡng giao dịch lớn QĐ 11/2023, quy tắc gom giao dịch phân tách dưới ngưỡng trong 24h, danh sách cảnh báo nội bộ).
- Cấu hình chính sách phê duyệt Maker-Checker đa cấp theo hạn mức tiền tệ (ví dụ: hạn mức chuyển tiền, hạn mức duyệt chênh lệch phí) với cơ chế bảo vệ phân tách quyền hạn (SoD) nghiêm ngặt.

### R5. Mathematical & Security Invariants Preservation
Bảo toàn tuyệt đối các bảo chứng an toàn và tính tất định:
- Tuyệt đối không dùng số thực (`f32`, `f64`) trong tính toán tài chính; sử dụng đơn vị cơ sở nhỏ nhất với số nguyên `u64/u128 checked_add/sub`, cấm hoàn toàn qua lint `#![deny(clippy::float_arithmetic)]`.
- Đảm bảo bất biến số dư: $\text{Số dư cuối} \equiv \text{Số dư đầu} + \sum \text{Có} - \sum \text{Nợ}$.
- Môi trường hoàn toàn offline: Zero-Egress, không mở socket mạng ra bên ngoài loopback/intranet cho phép.
- Lưu vết Merkle tree tamper-evident cho mọi thay đổi trạng thái và thay đổi cấu hình hệ thống.
- Thực thi nghiêm ngặt Segregation of Duties: Người đề xuất không thể tự phê duyệt (`maker != checker`).

## Acceptance Criteria

### Configuration Flexibility & Verifiability
- [ ] Thêm mới hoặc thay đổi định dạng sao kê ngân hàng chỉ cần bổ sung 1 tệp cấu hình DSL (YAML/JSON) vào thư mục mẫu; parser tự động nhận diện và trích xuất chính xác 100% dữ liệu không cần biên dịch lại mã Rust.
- [ ] Không còn bất kỳ mảng hằng số phí cố định (như `standard_fees = [1100, 2200, ...]`), danh sách từ dừng pháp nhân fix cứng, hay điểm tin cậy hardcoded nào trong mã nguồn `liva-recon`.
- [ ] Quy tắc sinh bút toán sổ cái và sơ đồ tài khoản (COA) được nạp hoàn toàn từ cấu hình; hỗ trợ thay đổi tài khoản hạch toán phí (ví dụ từ 6425 sang 6427) qua tệp cấu hình.
- [ ] Toàn bộ tệp cấu hình DSL có schema định nghĩa rõ ràng, được validate khi nạp (báo lỗi tường minh nếu cấu hình sai định dạng hoặc thiếu trường bắt buộc).

### Determinism & Regression Testing
- [ ] Bộ kiểm thử hồi quy trên Golden Dataset (`data/demo_ready/` và `crates/liva-recon/src/golden_dataset.rs`) đạt tỷ lệ khớp chính xác $\ge 99.8\%$, tỷ lệ khớp sai $= 0\%$.
- [ ] Bộ kiểm thử thuộc tính (`proptest`) kiểm chứng $\ge 10^5$ ca giao dịch ngẫu nhiên xác nhận bất biến toán học và không xảy ra panic hay tràn số.
- [ ] Lệnh kiểm tra `cargo clippy --workspace -- -D clippy::float_arithmetic` vượt qua 100% không có cảnh báo.

### Security & Compliance Gates
- [ ] Ràng buộc Segregation of Duties ngăn chặn thành công 100% các nỗ lực tự phê duyệt giao dịch (`maker == checker`).
- [ ] Kiểm thử Canary Egress xác nhận hệ thống hoàn toàn chặn mọi kết nối Internet ra ngoài (chế độ air-gap fail-closed).
- [ ] Mọi thay đổi cấu hình và sự kiện đối soát đều sinh lá băm Merkle hợp lệ, cho phép kiểm toán chứng thực toàn vẹn độc lập.

## Verification Resources
- Golden dataset và các mẫu sao kê chuẩn: `data/demo_ready/` (VCB XLSX, TCB CSV, BIDV CSV, Sổ cái kế toán CSV).
- Test suites hiện có: `crates/liva-recon/tests/proptest_split_solver.rs`, `crates/liva-money/tests/proptest_invariants.rs`, `crates/liva-ledger/tests/proptest_ledger.rs`.
</USER_REQUEST>

## 2026-09-15T18:01:29Z

<USER_REQUEST>
Kiểm tra toàn bộ dự án LIVA Banking, xóa bỏ hoàn toàn tất cả các dữ liệu giả lập (mock data) và chuỗi cứng (hardcode). Chuyển đổi toàn bộ trạng thái tài khoản ngân hàng, đối soát quyết toán, cảnh báo AML và lệnh chi sang cơ chế động thời gian thực (real-time dynamic state). Kết nối toàn bộ tính năng Trợ lý AI Copilot và Hệ thống thanh tra System Prompt Inspector tới mô hình AI cục bộ trên máy (mô hình Qwen 14B 4-bit qua endpoint OpenAI-compatible / LIVA Native Core) đảm bảo vận hành trơn tru và tuân thủ Zero Cloud Egress.

Working directory: e:\Project\01_AI_Agents\LIVA_Banking\teamwork_projects\liva_banking_universal
Integrity mode: development

## Requirements

### R1. Loại bỏ toàn bộ Hardcode & Chuyển đổi trạng thái Động toàn diện (Zero Hardcode & Dynamic State)
- Rà soát và loại bỏ tất cả các giá trị số dư tĩnh (ví dụ: 1.380.000.000 VND, 6.200.000.000 VND), danh sách tài khoản cứng, giao dịch mẫu và tỷ lệ khớp đối soát cố định (99.8%) ở frontend, stores (`treasuryStore`, `bankingStore`, `reconciliationStore`, `amlStore`) và backend server.
- Toàn bộ dữ liệu ngân quỹ, vị thế thanh khoản, tỷ lệ đối soát và danh mục lệnh chi phải được tính toán động từ cơ sở dữ liệu / sổ cái Merkle thực tế và tự động đồng bộ thời gian thực qua Server-Sent Events (SSE).

### R2. Tích hợp AI Local (Qwen 14B 4-bit) cho Trợ lý Copilot & Giám sát AML
- Thay thế hoàn toàn cơ chế đối sánh biểu thức chính quy (regex/heuristic) và câu trả lời mẫu trong `copilotEngine.ts` và `bankingAiTools.ts`.
- Kết nối Copilot Drawer và Hệ thống thanh tra Prompt Inspector tới endpoint AI cục bộ trên máy (OpenAI-compatible endpoint `http://127.0.0.1:8000/v1` hoặc `http://127.0.0.1:8002/v1` / Ollama `http://127.0.0.1:11434/v1` chạy Qwen 14B 4-bit).
- Copilot phải gọi các công cụ đọc dữ liệu ngân hàng (Banking Read-Only Tools) để lấy dữ liệu sổ cái thời gian thực và tổng hợp câu trả lời tự nhiên cho người dùng.
- Sandbox phòng thử nghiệm đối kháng (Evaluation Sandbox) trong tab AML phải gửi prompt trực tiếp tới model local và trả về kết quả phân tích rủi ro thực tế thay vì mảng alert tĩnh.
- Có cơ chế xử lý lỗi (graceful degradation) và hiển thị trạng thái kết nối tới AI Local trực quan trên giao diện.

### R3. Đảm bảo toàn vẹn nghiệp vụ Tuân thủ & An toàn bảo mật
- Duy trì nghiêm ngặt các chốt chặn kiểm soát: Nghị định 13/2023/NĐ-CP (Zero Cloud Egress, khử định danh PII cục bộ), Thông tư 09/2020/TT-NHNN (Cơ chế 4 mắt Maker-Checker không tự duyệt), và Thông tư 09/2023/TT-NHNN (Giám sát giao dịch đáng ngờ AML/STR).

## Verification Plan & Resources

### Automated Tests
- Chạy toàn bộ bộ kiểm thử tự động của dự án qua `npm run test` (Vitest) và kiểm tra E2E verification:
  - `tests/copilot.test.ts`: Xác thực Copilot xử lý động câu hỏi và kết nối AI tool.
  - `tests/serverSyncIntegration.test.ts`: Xác thực đồng bộ dữ liệu đa phân hệ qua SSE không bị trễ hay mang giá trị cứng.
  - `tests/makerChecker.test.ts` & `tests/merkleAudit.test.ts`: Kiểm tra tính toàn vẹn của sổ cái bất biến.
  - `tests/amlSurveillance.test.ts`: Kiểm thử phát hiện rủi ro rửa tiền động theo các siêu tham số.

### Acceptance Criteria
- [ ] Không còn bất kỳ biến số hardcode hoặc phản hồi giả định (mock response) nào trong luồng runtime của Copilot, AML Inspector và Bảng điều khiển ngân quỹ.
- [ ] Dữ liệu hiển thị trên thanh trạng thái (Thanh khoản, Tỷ lệ đối soát, Cảnh báo AML, Lệnh chi chờ duyệt) phản ánh chính xác 100% dữ liệu từ sổ cái/backend.
- [ ] Gửi câu hỏi bất kỳ trên Copilot Drawer kích hoạt suy luận của Local AI (Qwen 14B), trích xuất đúng thông tin số dư/giao dịch thực tế và phản hồi bằng tiếng Việt chuyên nghiệp.
- [ ] Chức năng "Kiểm Thử Prompt Với Kịch Bản Này" trong Phòng thử nghiệm AML thực hiện suy luận qua Local AI và hiển thị phân tích rủi ro động.
- [ ] Bộ kiểm thử Vitest (`npm run test`) pass 100% mà không xảy ra lỗi hồi quy (regression errors).
</USER_REQUEST>

## 2026-09-16T02:23:21Z

<USER_REQUEST>
Khảo sát, đối chiếu và tích hợp có chọn lọc các cải tiến công nghệ tiên tiến từ các nhánh mới của dự án LIVA gốc (`feat/openai-api-va-don-dep-avatar`, `test/perf-threshold-baseline` tại `e:/Project/01_AI_Agents/LIVA`) vào nền tảng `LIVA_Banking` nhằm nâng cao hiệu năng, chuẩn hóa giao tiếp Local AI và tăng cường an ninh bảo mật.

Working directory: e:/Project/01_AI_Agents/LIVA_Banking
Integrity mode: development

## Reference Material
- Nhánh gốc LIVA: `e:\Project\01_AI_Agents\LIVA` (các nhánh `feat/openai-api-va-don-dep-avatar`, `test/perf-threshold-baseline`, `mac-v2`)
- Dự án hiện tại: `e:\Project\01_AI_Agents\LIVA_Banking` (nhánh `main` và các crate ngân hàng `liva-compliance`, `liva-ledger`, `liva-recon`, `liva-parse`)

## Requirements

### R1. Khảo sát & Phân tích Ma trận Khác biệt (Deep Diff & Architectural Audit)
- Đối chiếu mã nguồn giữa các nhánh mới của `LIVA` (`feat/openai-api-va-don-dep-avatar`, `test/perf-threshold-baseline`) và `LIVA_Banking`.
- Đánh giá khả năng tương thích, xác định các module tối ưu có thể kế thừa (bóc tách kiến trúc cửa sổ widget, tối ưu hóa telemetry prompt token, giảm độ trễ TTFT, nâng cấp cổng kiểm tra giấy phép/lỗ hổng).
- Xác lập danh mục thành phần đề xuất gộp hoặc nâng cấp, phân loại rõ lợi ích kỹ thuật và phạm vi ảnh hưởng (blast radius).

### R2. Chuẩn hóa Cổng kết nối Local AI & Đo kiểm Hiệu năng (Local AI & Telemetry Upgrade)
- Kế thừa cơ chế chuẩn hóa OpenAI-compatible API cho mô hình AI cục bộ (Qwen 14B / Ollama / LIVA Core) trên máy tính mà không làm rò rỉ dữ liệu ra đám mây (Zero Cloud Egress).
- Bổ sung cơ chế đo lường độ trễ (TTFT - Time To First Token) và giám sát số lượng token prompt nền, giúp tối ưu hóa bộ nhớ và chi phí tính toán cho Trợ lý Tài chính Copilot.

### R3. Hiện đại hóa Cổng Kiểm tra An ninh & Phụ thuộc (Security & Dependency Gate Modernization)
- Tích hợp công cụ `cargo-deny` thay thế cho `cargo-audit` trong quy trình CI/CD cục bộ để kiểm soát chặt chẽ giấy phép bản quyền và cảnh báo lỗ hổng của các crate Rust.
- Cập nhật và vá các lỗ hổng phụ thuộc bảo mật (như npm `js-yaml` CVE-2026-59870) trên toàn bộ các workspace frontend và backend.

### R4. Bảo toàn Bất biến Tài chính & Kiểm soát Tài nguyên Nghiêm ngặt (Invariants & Resource Guardrails)
- Bảo toàn tuyệt đối các bất biến toán học ngân hàng: không sử dụng số thực floating-point (`f32`/`f64`) cho số dư/tiền tệ, tuân thủ `checked_add/sub` trên số nguyên, và lưu vết kiểm toán Merkle tree bất biến.
- Tuân thủ nghiêm ngặt quy tắc giới hạn tài nguyên hệ thống: chỉ biên dịch tuần tự với cờ `-j 2` và chạy kiểm thử với `-- --test-threads 2`; không tự động thực hiện các thao tác git remote (`push`, `pull`, `commit`).

## Acceptance Criteria

### Báo cáo So sánh Khác biệt & Lộ trình Nâng cấp
- [ ] Lập bảng ma trận so sánh chi tiết các thay đổi giữa nhánh mới của `LIVA` và `LIVA_Banking` (tệp thay đổi, chức năng cải tiến, rủi ro xung đột).
- [ ] Báo cáo khuyến nghị danh mục các tính năng nên gộp (cherry-pick / integrate) ngay lập tức và các tính năng cần giữ riêng biệt theo nghiệp vụ ngân hàng.

### Cổng Kiểm định Mã nguồn & Biên dịch (Rust & Frontend)
- [ ] `cargo check -j 2` và `cargo test --workspace -j 2 -- --test-threads 2` đạt kết quả biên dịch và kiểm thử thành công 100% không có lỗi.
- [ ] Bộ kiểm tra phụ thuộc `cargo-deny` hoặc kiểm toán bảo mật đạt 0 lỗ hổng nghiêm trọng (zero high/critical vulnerabilities).
- [ ] Kiểm thử frontend `teamwork_projects/liva_banking_universal` (`npm run test`) vượt qua toàn bộ các bài test bao gồm kiểm thử đối kháng Copilot và đồng bộ dữ liệu thời gian thực.

### Tuân thủ An ninh & Zero Data Egress
- [ ] Không có bất kỳ kết nối mạng nào ra ngoài Internet trong suốt quá trình vận hành của Local AI Client và Core Engine.
- [ ] Toàn bộ các thay đổi mã nguồn được chuẩn bị dừng lại tại ranh giới staging (`git add`), không tự ý commit hay push.
</USER_REQUEST>

## 2026-09-16T02:29:21Z

<USER_REQUEST>
[USER SCOPE EXPANSION UPDATE]
The user has confirmed they just pushed 2 new branches to `DuongNAD/LIVA_Banking` on GitHub, making 3 active branches in total:
1. `main` (`ba560edd09824a191daa8bf144134c04c56af2e5`): Current branch with Universal Web Client, Local AI Client (Qwen 14B), and Dynamic DSL crates (`liva-ledger`, `liva-recon`, `liva-compliance`, `liva-parse`, `liva-netguard`).
2. `feat/liva-banking-harness-p0` (`a4d9c9a500ff0462636997beec762b7436780b52`): Monolithic banking engine crate (`crates/liva-banking` with ISO 20022 XML, VCB Excel, TCB CSV, Tier 1/2/3 recon, period close, RBAC, maker-checker, merkle audit).
3. `feat/v2-production-ready-sprint0-1` (`a421f17bf3a0beaac5baf7140bf91d8b04b8c09a`): Modular micro-crates enterprise architecture (`apps/liva-server`, `crates/liva-audit`, `crates/liva-ingest` [MT940, CAMT.053, VCB, TCB, BIDV, profile sniffing, dedup], `crates/liva-match`, `crates/liva-normalize`, `migrations/` [PostgreSQL + SQLite WAL with audit triggers]).

Please expand the Sentinel / Orchestrator scope to perform a comprehensive Tri-Branch Architectural Diff and Synthesis:
- Compare the strengths of all 3 branches.
- Formulate the optimal unified architecture and harmonization upgrade plan (e.g. combining the robust multi-bank ingest and server from v2 with the dynamic YAML DSL and Web Client Universal from main).
- Preserve all financial and safety invariants (zero float arithmetic, Merkle audit trail, zero cloud egress, sequential `-j 2` builds, git staging boundary only).
</USER_REQUEST>

## 2026-09-16T02:40:25Z

Refactor and eliminate all hardcoded security credentials, personal tokens, developer-specific file paths, and personal ngrok domains across the LIVA Banking repository, replacing them with dynamic environment variables, declarative configurations, and portable fallbacks while preserving standalone demo mock data and passing all existing verification test suites.

Working directory: e:\Project\01_AI_Agents\LIVA_Banking
Integrity mode: development

## Requirements

### R1. Redact and Parameterize Security Credentials
- Remove hardcoded ngrok authtokens (deploy/ngrok/ngrok-treasury-workbench.yml) and personal operator emails from repository files. Provide an example configuration template (.env.example or sanitized yaml) referencing $env:NGROK_AUTHTOKEN.
- Eliminate fallback JWT secret strings in teamwork_projects/liva_banking_universal/server/server.mjs. Ensure the backend server securely enforces or derives dynamic secrets when JWT_SECRET is not provided in local environments, preventing predictable static tokens.

### R2. Parameterize Machine-Specific File Paths
- Replace the hardcoded DEFAULT_MODELS_DIR (E:\AI_Models) in liva-native-core/src/paths.rs and related probe binaries with dynamic path resolution that honors LIVA_MODELS_DIR, data/liva-config.json, or a standardized portable project-local/OS fallback directory.

### R3. Generalize Ingress Domains and Network Origins
- Remove hardcoded personal ngrok domain strings (rd_3JHcwHJK1iUTdwx1D22XRMeUA5K.ngrok-free.app) from liva-native-core/src/lib.rs (DEFAULT_WS_ALLOWED_ORIGINS) and PowerShell deployment scripts (scripts/deploy-ngrok-tunnel.ps1), allowing custom domains to be injected via environment variables (LIVA_WS_ALLOWED_ORIGINS, $Domain).

### R4. Preserve Standalone Demo & Test Invariants
- Retain sample demo data and mock accounts necessary for zero-backend browser demonstrations (liva_banking_universal) and tests, while ensuring production code paths do not rely on static default credentials.

## Acceptance Criteria

### Credential & Domain Sanitization
- [ ] No personal ngrok authtokens or personal email addresses exist in any git-tracked configuration files under deploy/ngrok/ or scripts.
- [ ] No hardcoded personal .ngrok-free.app subdomains exist in liva-native-core/src/lib.rs.
- [ ] teamwork_projects/liva_banking_universal/server/server.mjs generates a secure ephemeral fallback key or logs an explicit warning without using a static public password string when process.env.JWT_SECRET is absent.

### Path Portability
- [ ] Running path resolution functions in liva-native-core on a machine without drive E:\ returns a valid fallback directory path without panicking.
- [ ] node scripts/models.mjs doctor and cargo check -p liva-native-core -j 2 execute successfully.

### System Verification & Zero Regression
- [ ] cargo test -p liva-native-core --lib banking -j 2 -- --test-threads 2 passes with zero failures.
- [ ] node scripts/demo_runner.mjs --dry-run finishes with exit code 0.
- [ ] npm run check:installer completes without configuration errors.

## 2026-09-16T03:16:54Z

<USER_REQUEST>
Tích hợp và mở khóa toàn diện năng lực Dual-Engine Local AI (ONNX Runtime cho embedding/anomaly detection và llama-cpp-2 SLM cho giải thích ngữ nghĩa) vào hệ sinh thái ngân hàng LIVA: nâng cấp banking crates, bổ sung triggers phát hiện bất thường giao dịch đa chiều, kết nối MCP tools & skills, và xây dựng harness kiểm thử tự động tuân thủ nghiêm ngặt Zero Data Egress và RAM <= 4GB.

Working directory: e:\Project\01_AI_Agents\LIVA_Banking
Integrity mode: development

## Requirements

### R1. Mở khóa và Tích hợp Local AI Engine vào Phân hệ Ngân hàng
- Kết nối bộ suy luận cục bộ sẵn có trong `liva-native-core` (ONNX Runtime và `llama-cpp-2`) vào các pipeline xử lý ngân hàng (`liva-compliance`, `liva-recon`, `liva-ledger`).
- Cung cấp kiến trúc module hóa với cơ chế Fallback tất định (Deterministic Fallback) bảo đảm hệ thống luôn hoạt động bình thường kể cả khi model chưa được nạp hoặc tài nguyên phần cứng bị giới hạn.
- Đảm bảo 100% xử lý in-process trên máy cục bộ, kích hoạt `liva-netguard` ngăn chặn mọi kết nối non-loopback (Zero Data Egress).

### R2. Bộ Kích Hoạt & Mô Hình Nhận Diện Bất Thường Giao Dịch (Local AI Anomaly Detection Triggers)
- Xây dựng engine phát hiện bất thường giao dịch đa chiều (Multi-Dimensional Anomaly Detection) kết hợp:
  1. *Phân tích chuỗi thời gian & vận tốc (Temporal & Velocity Anomaly)*: Đột biến tần suất, dòng tiền ra vào ngoài khung giờ kinh doanh thông thường.
  2. *Phân tích phân phối số tiền (Amount Distribution Outliers)*: Nhận diện smurfing/structuring tinh vi bằng phương pháp thống kê / Isolation Forest nhẹ.
  3. *Phân tích ngữ nghĩa nội dung chuyển khoản (Semantic Narration Anomaly)*: Dùng ONNX embedding so khớp vector ngữ nghĩa của nội dung thanh toán để phát hiện cụm từ đáng ngờ hoặc tài khoản mượn danh (money mule).
- Kích hoạt triggers tự động phát cảnh báo (Alert Generation) kèm điểm số bất thường (Anomaly Score) và lý do giải thích bằng tiếng Việt tạo từ mô hình SLM cục bộ.

### R3. Nâng cấp Banking Skills và Giao Diện MCP Tools
- Cập nhật các MCP tools trong `liva-native-core/src/banking/mcp.rs`:
  - `compliance_aml_screen`: Bổ sung trường đánh giá bất thường AI (`ai_anomaly_score`, `anomaly_reasons`, `risk_profile`).
  - `reconcile_statement_batch`: Bổ sung đối chiếu ngữ nghĩa (Semantic Reconciliation) ở Tier 2 kết hợp cùng Integer Jaro-Winkler.
  - `credit_risk_scoring`: Bổ sung tóm tắt phân tích rủi ro định tính tự động sinh bởi SLM local.
- Cập nhật và đồng bộ hóa các tài liệu kỹ năng tại `.agents/skills/` (`liva-banking-compliance`, `liva-banking-reconciliation`, `liva-banking-risk`, `liva-teamwork-harness`) phản ánh chính xác các schema mới và năng lực Local AI.

### R4. Harness Kiểm Thử Tự Động & Tiêu Chuẩn Hiệu Năng (Automated Test & Benchmark Suite)
- Phát triển bộ kiểm thử tự động toàn diện cho Local AI Banking:
  - Unit tests và integration tests bao phủ các kịch bản bình thường và đối kháng (adversarial test cases).
  - Benchmark kiểm tra độ trễ suy luận P90 (< 50ms cho ONNX anomaly detection, < 500ms TTFT cho local SLM).
  - Đo đạc và xác nhận giới hạn bộ nhớ RAM tối đa tiêu thụ <= 4GB theo quy chuẩn hệ thống.
  - Tự động chạy tuần tự với tham số bắt buộc `-j 2` và `-- --test-threads 2`.

## Acceptance Criteria

### Tính Toàn Vẹn & Khả Năng Thực Thi (Build & Execution)
- [ ] Mã nguồn biên dịch hoàn toàn sạch không lỗi (`cargo check -j 2 --all-targets`).
- [ ] Không thêm bất kỳ phụ thuộc đám mây hoặc gọi API bên ngoài; `liva-netguard` xác nhận Zero Cloud Leakage.

### Năng Lực Phát Hiện Bất Thường (AI Anomaly Detection)
- [ ] Bộ kích hoạt (triggers) nhận diện chính xác các kịch bản bất thường đa chiều (velocity surge, structuring, semantic anomaly) trên bộ dữ liệu kiểm thử vàng với F1-score >= 0.90.
- [ ] Điểm số bất thường (`anomaly_score` từ 0.0 đến 1.0) và lý do cảnh báo được gắn kèm vào từng giao dịch bị gắn cờ mà không làm rò rỉ dữ liệu cá nhân (PII) đã được che mặt nạ theo Nghị định 13.

### Đối Chiếu Ngữ Nghĩa (Semantic Reconciliation)
- [ ] Module đối chiếu Tier 2 hỗ trợ kết hợp giữa Jaro-Winkler và vector embedding ngữ nghĩa, giải quyết được các trường hợp tên đơn vị/nội dung viết tắt hoặc biến thể không gian.

### Tuân Thủ MCP & Skills Parity
- [ ] Các công cụ MCP (`compliance_aml_screen`, `reconcile_statement_batch`, `credit_risk_scoring`) trả về đúng định dạng có tích hợp dữ liệu Local AI và vượt qua toàn bộ integration tests.
- [ ] Toàn bộ skill tương ứng trong `.agents/skills/` được cập nhật khớp với input/output contract mới.

### Kiểm Thử Tự Động & Tài Nguyên (Resource & Test Guardrails)
- [ ] Toàn bộ test suites chạy thành công với lệnh: `cargo test -p liva-native-core -p liva-compliance -p liva-recon -j 2 -- --test-threads 2`.
- [ ] Bộ nhớ RAM đỉnh của tiến trình trong quá trình kiểm thử không vượt quá 4GB.
</USER_REQUEST>

## 2026-09-16T03:18:01Z

<USER_REQUEST_UPDATE>
Yêu cầu bổ sung từ người dùng:
Sử dụng model Qwen3 14B (`E:\AI_Models\Qwen_Qwen3-14B-Q5_K_M.gguf`, kích thước 10,514,570,272 bytes).
Hãy cập nhật cấu hình router/expert model và tích hợp hỗ trợ cho model này trong quá trình mở khóa Local AI cho LIVA Banking. Đảm bảo tuân thủ cơ chế artifact trust (sha256 whitelist trong models-manifest.json).
</USER_REQUEST_UPDATE>

## 2026-09-16T03:18:07Z

<USER_REQUEST_UPDATE>
Đã băm SHA-256 thực tế của file `E:\AI_Models\Qwen_Qwen3-14B-Q5_K_M.gguf`:
`bf19bf5c77c530762012f71215b62aa8f0bae08111ee0d537b8496479e17cfa3`
(bytes: 10514570272).
Hãy dùng hash này nếu cần cập nhật vào `data/models-manifest.json` cho trust anchor verification.
</USER_REQUEST_UPDATE>

## 2026-09-16T05:23:35Z

<USER_REQUEST>
Fix all compilation and type compatibility errors in `liva-native-core` so that it seamlessly interoperates with `liva-money` and `liva-parse`, and passes all test suites cleanly.

Working directory: e:\Project\01_AI_Agents\LIVA_Banking
Integrity mode: development

## Requirements

### R1. Type & Interface Alignment
Resolve all API mismatches, missing crate imports, and constructor discrepancies in `liva-native-core/src/banking/mcp.rs` and its module dependencies to ensure full compatibility with the current versions of `liva-money` and `liva-parse`.

### R2. Zero Compiler Warnings and Zero Test Failures
Ensure `liva-native-core` compiles with zero compiler errors and passes all test cases in its test suite without any flaky tests or broken invariants.

### R3. Safe Resource & RAM Boundaries
Adhere strictly to resource guardrails: all Cargo commands must use `-j 2` concurrency and `-- --test-threads 2` for test runners to maintain system stability.

### R4. Non-Regression Invariant
Ensure no regressions or breaking changes are introduced to neighboring crates (`crates/liva-money`, `crates/liva-parse`, `crates/liva-recon`, `crates/liva-compliance`, and `crates/liva-netguard`).

## Acceptance Criteria

### Compilation & Build
- [ ] `cargo check -p liva-native-core -j 2` exits with code 0 and 0 errors.
- [ ] `cargo test -p liva-native-core -j 2 --no-run` compiles test binaries with code 0.

### Test Execution
- [ ] `cargo test -p liva-native-core -j 2 -- --test-threads 2` completes with 100% of tests passing and 0 failures.

### Neighboring Crates Regression Verification
- [ ] `cargo test -p liva-money -j 2 -- --test-threads 2` passes 100%.
- [ ] `cargo test -p liva-parse -j 2 -- --test-threads 2` passes 100%.
- [ ] `cargo test -p liva-recon -j 2 -- --test-threads 2` passes 100%.
</USER_REQUEST>
