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

## 2026-09-15T09:28:28Z

<USER_REQUEST>
# Teamwork Project Prompt

Requested team: Full multi-agent team (Architecture, Rust Core/Math, Parser/Integrations, Local AI, QA)

Nâng cấp kiến trúc LIVA Banking Harness, thiết lập nền tảng monorepo đa crate hoàn chỉnh và hoàn thành Milestone M1 & M2 (Sprint 0 đến Sprint 4): Bộ lõi tính toán số học chuẩn kế toán VAS, engine đối soát 2 tầng (Tier 1 Exact & Tier 2 Fuzzy/Fee), cây kiểm toán Merkle Tree RFC 6962, bộ trích xuất sao kê 6 ngân hàng Việt Nam kèm CAMT.053/MT940 và mô hình trích xuất thực thể cục bộ SLM NER.

Working directory: /Users/duongnad/Documents/project/LIVA_Banking
Integrity mode: development

## Requirements

### R1. Workspace Modularization & Exact Math Engine
- Tổ chức workspace monorepo với các crates độc lập: `crates/liva-money` (số học u64 cents không float, quy tắc làm tròn VAS, kiểm tra bất biến sổ cái), `crates/liva-audit` (cây băm Merkle Tree RFC 6962 append-only, HMAC-SHA256 log chuỗi, hàm xuất và xác minh inclusion proof).
- Cung cấp module `liva-match` hỗ trợ: Tier 1 (khớp chính xác O(1) qua hash key chuẩn hóa trong cửa sổ thời gian ±24h) và Tier 2 (khớp mờ Jaro-Winkler với tiếng Việt không dấu, nhận diện và tách phí chuyển khoản theo bảng phí ngân hàng hạch toán TK 6425).

### R2. Banking Statement Ingest & Normalization Engine
- Cung cấp crate `crates/liva-ingest` và `crates/liva-normalize` có khả năng tự động nhận diện mẫu biểu, trích xuất dữ liệu giao dịch từ 6 ngân hàng Việt Nam (Vietcombank, Techcombank, BIDV, VietinBank, MBBank, Agribank) từ các định dạng XLSX, CSV, chuẩn quốc tế CAMT.053 (XML), MT940 và tài liệu PDF text-layer.
- Chuẩn hóa toàn bộ ngày tháng sang ISO 8601, tiền tệ chuẩn VND/u64 cents, mã tham chiếu chuẩn và tên đối tác viết hoa không dấu.

### R3. Local SLM NER Extraction for Transaction Memo
- Cung cấp crate `crates/liva-nlp` tích hợp mô hình ngôn ngữ nhỏ chạy cục bộ (llama.cpp GGUF Q4_K_M) để phân tích diễn giải thanh toán tiếng Việt (Napas247, VietQR, UNC, POS).
- Trích xuất cấu trúc JSON đảm bảo ngữ pháp (grammar-constrained JSON) gồm: số hóa đơn (`invoice_no`), mã đơn hàng (`order_no`), tên đối tác chuẩn hóa, cờ phí (`fee_flag`), và điểm tự tin (`confidence`). Các bản ghi có độ tin cậy < 0.6 được gắn cờ chuyển sang hàng đợi kiểm duyệt HITL.

### R4. Verification & Hardening Infrastructure
- Cung cấp bộ test suites tự động bao gồm: proptest kiểm tra tính toán tiền tệ và bất biến sổ cái, golden file tests đối soát với dữ liệu sao kê mẫu trong `fixtures/statements/`, và integration tests đo lường thông lượng khớp giao dịch.
- Đảm bảo cơ chế app-layer Zero-Egress guard (không mở socket ra ngoài ngoại trừ loopback 127.0.0.1) và cấu hình mã hóa lưu trữ CSDL SQLCipher AES-256.

## Verification Resources
- Tập tin mẫu kiểm thử sẵn có trong kho: `fixtures/statements/` (`vcb_aug2026.xlsx`, `tcb_aug2026.csv`, `bidv_aug2026.pdf`, `vcb_adversarial_merged.xlsx`) và `fixtures/erp_ledger/open_invoices.json`.

## Acceptance Criteria

### Correctness & Financial Integrity
- [ ] `cargo test -p liva-money` chạy thành công với proptest >= 100.000 mẫu ngẫu nhiên không xảy ra tràn số (overflow) và không có sai lệch float (zero float drift).
- [ ] Invariant bảo toàn số dư: `closing_cents == opening_cents + SUM(credit) - SUM(debit)` được kiểm chứng tự động và luôn đúng trên toàn bộ phiên đối soát.
- [ ] `cargo test -p liva-audit` chứng minh Merkle Tree tuân thủ RFC 6962; thay đổi bất kỳ 1 byte nào trong dữ liệu đã chốt phiên sẽ khiến kiểm tra xác minh Root Hash thất bại 100%.

### Parsing & Matching Precision
- [ ] Parse thành công 100% các file sao kê mẫu trong `fixtures/statements/` (VCB, TCB, BIDV) và CAMT.053/MT940 mà không làm rơi dòng hay sai lệch số tiền.
- [ ] Thuật toán Tier 1 và Tier 2 đạt tỷ lệ khớp chính xác (precision) >= 97% trên tập dữ liệu fixtures tiêu chuẩn.
- [ ] Bộ tách phí (Fee Splitter) nhận diện và hạch toán đúng các khoản phí chênh lệch (1.100 - 22.000 VND và VAT) vào tài khoản chi phí 6425.

### SLM & System Performance
- [ ] SLM NER trích xuất thực thể đúng schema JSON định nghĩa sẵn và đạt thông lượng >= 25 tokens/s trên CPU AVX2/AVX512.
- [ ] Toàn bộ mã nguồn vượt qua kiểm tra `cargo clippy -- -D warnings` và `cargo check -j 2`.
- [ ] Socket allowlist chặn hoàn toàn các kết nối mạng ngoại vi (chỉ chấp nhận 127.0.0.1).

</USER_REQUEST>

## 2026-09-15T17:24:18Z

<USER_REQUEST>
Requested team: Full team đa tác tử (Architect, Backend Rust Engineer, Test/QA Engineer)

Nâng cấp nền tảng kiến trúc LIVA Banking Harness từ Desktop Standalone sang On-Premise Client-Server V2 (Sprint 0 & Sprint 1): khởi tạo `apps/liva-server` (Axum/Tokio), hoàn thiện hệ thống migrations PostgreSQL với DB triggers bảo vệ `audit_logs` append-only, chuẩn hóa `liva-core::Money` checked arithmetic và mở rộng `liva-ingest` xử lý sao kê đa định dạng chống trùng lặp.

Working directory: /Users/duongnad/Documents/project/LIVA_Banking
Integrity mode: development

## Requirements

### R1. Kiến trúc Monorepo & Khởi tạo Service `apps/liva-server`
Khởi tạo crate `apps/liva-server` trong Cargo workspace sử dụng Axum và Tokio làm trung tâm điều phối cho mạng LAN nội bộ; cấu hình kết nối cơ sở dữ liệu tập trung qua SQLx; triển khai cấu trúc handler/router module hóa và endpoint kiểm tra trạng thái dịch vụ (healthcheck).

### R2. Chuẩn hóa Schema Cơ sở dữ liệu & Bất biến Audit Trail
Thiết lập các bản migration SQLx (PostgreSQL / SQLite tương thích) cho các thực thể: `legal_entities`, `fiscal_periods`, `counterparties`, `counterparty_aliases`, `holidays`, `bank_accounts`, `bank_profiles`, `bank_transactions`, `quarantine_items`, và `audit_logs`. Cài đặt Database Trigger bảo vệ bảng `audit_logs` ở chế độ append-only (cấm UPDATE và DELETE), liên kết các bản ghi bằng chuỗi hash chain (`prev_hash` và `row_hash`).

### R3. Hoàn thiện Động cơ Tiền tệ `liva-core` (Checked Arithmetic & Continuity Invariant)
Chuẩn hóa struct `Money` hỗ trợ đơn vị `i64 minor unit`, scale đa tiền tệ (VND scale=0, USD scale=2), phủ toàn bộ phép toán số học an toàn (`checked_add`, `checked_sub`, `checked_mul_ratio` với Banker's rounding) tuyệt đối cấm số thực float (`clippy::float_arithmetic`). Cài đặt hàm kiểm tra bất biến số dư liên tục giữa các kỳ: `Opening(Period N) == Closing(Period N-1)`.

### R4. Engine Ingest Đa định dạng & Cơ chế Chống Trùng lặp
Cập nhật crate `liva-ingest` để đọc và chuẩn hóa dữ liệu sao kê ngân hàng từ file Excel (.xlsx thông qua calamine) và CSV theo cấu hình cột linh hoạt từ `bank_profiles`. Triển khai cơ chế sinh `statement_fingerprint` (SHA-256 nội dung + kỳ sao kê) và `txn_hash` cho từng dòng giao dịch để ngăn chặn triệt để tình trạng nạp trùng lặp sao kê hoặc chồng kỳ.

### R5. Ràng buộc Hạ tầng & Giới hạn Tài nguyên (Defense-in-Depth & RAM Guardrails)
Tuân thủ nghiêm ngặt quy tắc Zero-Egress: tuyệt đối không phát sinh traffic ra ngoài Internet. Mọi lệnh biên dịch và kiểm thử Rust phải tuân thủ RAM guardrails: luôn truyền `-j 2` cho `cargo check`/`cargo build`/`cargo test` và `-- --test-threads 2`. Giữ an toàn Git: ranh giới can thiệp mã nguồn dừng ở staging (`git add`), không tự ý commit hay push remote.

## Acceptance Criteria

### Tính Đúng đắn Số học & Bất biến Số dư
- [ ] `cargo test -p liva-core -j 2 -- --test-threads 2` vượt qua 100% unit tests và property-based tests (`proptest`).
- [ ] Phép tính tiền tệ `Money` không bao giờ panic hoặc tràn số âm trong phạm vi `i64`; từ chối thực hiện phép tính nếu sai lệch loại tiền tệ (`MoneyError::CurrencyMismatch`).
- [ ] Hàm kiểm tra bất biến số dư phát hiện chính xác mọi độ lệch Δ ≠ 0 giữa số dư đầu kỳ N và cuối kỳ N-1.

### Ingest & Khử trùng lặp (Idempotency)
- [ ] `liva-ingest` parse thành công các file mẫu thực tế trong `fixtures/statements/` (VCB .xlsx, TCB .csv, v.v.) mà không phát sinh lỗi hoảng loạn (panic).
- [ ] Import lại cùng một file sao kê hoặc dòng giao dịch trùng lặp kích hoạt cơ chế nhận diện `statement_fingerprint` / `txn_hash` và từ chối nạp đè.

### Cơ sở Dữ liệu & Audit Log Bất biến
- [ ] Các tệp migration SQLx áp dụng thành công và tạo đầy đủ bảng, khóa ngoại và ràng buộc `CHECK (maker_id != checker_id)` trên `quarantine_items`.
- [ ] Trigger bảo vệ trên bảng `audit_logs` lập tức ném lỗi ngoại lệ ngăn chặn mọi thao tác `UPDATE` hoặc `DELETE`.
- [ ] Mỗi bản ghi mới trong `audit_logs` tính toán chính xác chuỗi hash liên tục bảo đảm tính toàn vẹn kiểm toán.

### Biên dịch & Tích hợp Workspace
- [ ] Toàn bộ workspace biên dịch thành công không có lỗi (`cargo check -j 2`).
- [ ] Crate `apps/liva-server` khởi động được máy chủ Axum trên cổng nội bộ và phản hồi endpoint health check hợp lệ.

</USER_REQUEST>

