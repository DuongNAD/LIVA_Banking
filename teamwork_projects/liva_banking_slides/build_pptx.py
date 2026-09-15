#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
LIVA Banking Harness — Presentation Generator for INNOSTART 2026 Demo Day
Generates a 16:9 Widescreen 27-Slide PowerPoint Deck (liva_innostart_2026.pptx)
Adhering strictly to Presenton, PPT Master, SlideSage, and SlideMason standards:
- Palette: Ultra-clean White (#FFFFFF), Cards (#F8FAFC), Subtle Border (#E2E8F0),
  Corporate Sky Blue (#0284C7), Sky Blue (#0EA5E9), Royal Navy (#1E3A8A),
  Charcoal Slate (#0F172A), Slate Body (#334155), Trust Emerald (#10B981),
  Danger Red (#DC2626), Caution Amber (#D97706)
- Composition Primitives:
  * add_compare_grid: Side-by-side high-contrast Trước khi có LIVA vs Khi có LIVA (Slide 6, 21, 23)
  * add_step_chevron: Clean horizontal chevrons for Slide 7 (3-step journey), Slide 11 (GTM), Slide 14 (Roadmap)
  * add_donut_chart: Native editable Doughnut Charts (XL_CHART_TYPE.DOUGHNUT) on Slides 13, 17, 25
  * Half-and-half layout with architectural photography on Slides 1, 5, 27
  * FeatureCards and StatBoxes with big numbers and zero text clutter
- 100% Native editable PowerPoint shapes: Rounded rectangles, text frames, runs, charts.
- Zero flat rasterized card images. Zero banned engineer jargon.
- All 27 speaker notes embedded directly from speaker_notes_27.json.
"""

import os
import sys
import json
import re

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')
if hasattr(sys.stderr, 'reconfigure'):
    sys.stderr.reconfigure(encoding='utf-8')

from pptx import Presentation
from pptx.util import Inches, Pt
from pptx.enum.text import PP_ALIGN
from pptx.enum.shapes import MSO_SHAPE, MSO_CONNECTOR
from pptx.enum.chart import XL_CHART_TYPE, XL_LEGEND_POSITION
from pptx.chart.data import CategoryChartData
from pptx.dml.color import RGBColor

# -----------------------------------------------------------------------------
# CONSTANTS & PALETTE SPECIFICATION
# -----------------------------------------------------------------------------
SLIDE_WIDTH = Inches(13.333)
SLIDE_HEIGHT = Inches(7.5)

MARGIN_X = Inches(0.8)
CONTENT_WIDTH = Inches(11.733)

# Brand Color Palette
COLOR_WHITE = RGBColor(255, 255, 255)          # #FFFFFF Pure White Canvas
COLOR_BLUE_PRIMARY = RGBColor(2, 132, 199)     # #0284C7 Corporate Sky Blue 600
COLOR_BLUE_SKY = RGBColor(14, 165, 233)        # #0EA5E9 Sky Blue 500
COLOR_BLUE_NAVY = RGBColor(30, 58, 138)        # #1E3A8A Deep Royal Navy 900
COLOR_DARK_SLATE = RGBColor(15, 23, 42)        # #0F172A Charcoal / Slate 900
COLOR_TEXT_BODY = RGBColor(51, 65, 85)         # #334155 Slate 700
COLOR_TEXT_MUTED = RGBColor(100, 116, 139)     # #64748B Slate 500
COLOR_TEXT_LIGHT = RGBColor(148, 163, 184)     # #94A3B8 Slate 400

COLOR_EMERALD = RGBColor(16, 185, 129)         # #10B981 Trust Emerald 500
COLOR_EMERALD_BG = RGBColor(240, 253, 244)     # #F0FDF4 Emerald Tint
COLOR_EMERALD_BORDER = RGBColor(167, 243, 208) # #A7F3D0 Mint Border

COLOR_DANGER = RGBColor(220, 38, 38)           # #DC2626 Red 600
COLOR_DANGER_BG = RGBColor(254, 242, 242)      # #FEF2F2 Red Tint
COLOR_DANGER_BORDER = RGBColor(254, 202, 202)  # #FECACA Red Border
COLOR_RED_BG = COLOR_DANGER_BG
COLOR_RED_BORDER = COLOR_DANGER_BORDER

COLOR_AMBER = RGBColor(217, 119, 6)            # #D97706 Amber 600
COLOR_AMBER_BG = RGBColor(255, 251, 235)       # #FFFBEB Amber Tint
COLOR_AMBER_BORDER = RGBColor(253, 230, 138)   # #FDE68A Amber Border

COLOR_CARD_BG = RGBColor(248, 250, 252)        # #F8FAFC Soft Slate Card
COLOR_CARD_BORDER = RGBColor(226, 232, 240)    # #E2E8F0 Subtle Border
COLOR_STRIPE_BLUE = RGBColor(240, 249, 255)    # #F0F9FF Sky Tint
COLOR_STRIPE_BORDER = RGBColor(186, 230, 253)  # #BAE6FD Sky Border

FONT_FAMILY = "Segoe UI"
FONT_MONO = "Consolas"

# Asset Paths
ASSETS_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets")
IMG_SKYSCRAPER = os.path.join(ASSETS_DIR, "glass_skyscraper.jpg")
IMG_GEOMETRY = os.path.join(ASSETS_DIR, "architecture_geometry.jpg")
IMG_CURVE = os.path.join(ASSETS_DIR, "architecture_curve.jpg")

class LivaDeckBuilder:
    def __init__(self, output_path):
        self.output_path = output_path
        self.prs = Presentation()
        self.prs.slide_width = SLIDE_WIDTH
        self.prs.slide_height = SLIDE_HEIGHT
        self.blank_layout = self.prs.slide_layouts[6]

        # Load speaker notes from JSON
        self.speaker_notes = {}
        base_dir = os.path.dirname(os.path.abspath(__file__))
        candidate_paths = [
            os.path.join(os.path.dirname(os.path.dirname(base_dir)), ".agents", "worker_narrative_blueprint", "speaker_notes_27.json"),
            os.path.join(os.path.dirname(base_dir), ".agents", "worker_narrative_blueprint", "speaker_notes_27.json"),
            os.path.join(base_dir, ".agents", "worker_narrative_blueprint", "speaker_notes_27.json"),
            os.path.join(base_dir, "speaker_notes_27.json"),
            os.path.join(os.getcwd(), ".agents", "worker_narrative_blueprint", "speaker_notes_27.json"),
        ]
        for p in candidate_paths:
            if os.path.exists(p):
                try:
                    with open(p, "r", encoding="utf-8") as f:
                        self.speaker_notes = json.load(f)
                    print(f"[*] Loaded {len(self.speaker_notes)} speaker notes from {p}")
                    break
                except Exception as e:
                    print(f"[!] Warning: Could not parse speaker notes from {p}: {e}")

    def get_speaker_note(self, slide_num):
        """Formats conversational Vietnamese speaker notes for slide."""
        key = str(slide_num)
        if key in self.speaker_notes:
            n = self.speaker_notes[key]
            timing = n.get("timing", "25s")
            wpm = n.get("wpm", "~135 wpm")
            tone = n.get("tone", "Đĩnh đạc, tự tin")
            spoken = n.get("spoken", "")
            takeaway = n.get("takeaway", "")
            punchline = n.get("punchline", "")
            obj_q = n.get("objection_q", "")
            obj_a = n.get("objection_a", "")
            proof = n.get("context_proof", "")

            return (
                f"[Slide {slide_num:02d} | Thời lượng: {timing} | Tốc độ: {wpm} | Âm sắc: {tone}]\n\n"
                f"LỜI THOẠI DIỄN GIẢ:\n{spoken}\n\n"
                f"KEY TAKEAWAY:\n{takeaway}\n\n"
                f"PUNCHLINE:\n{punchline}\n\n"
                f"CÂU HỎI VẶN & PHẢN XẠ (OBJECTION Q&A):\n- Q: {obj_q}\n- A: {obj_a}\n\n"
                f"ĐIỂM TỰA CHỨNG MINH:\n{proof}"
            )
        return f"[Slide {slide_num:02d}] Ghi chú diễn giả LIVA Banking Harness."

    def add_blank_slide(self):
        slide = self.prs.slides.add_slide(self.blank_layout)
        bg = slide.background
        fill = bg.fill
        fill.solid()
        fill.fore_color.rgb = COLOR_WHITE

        # Signature 4-Corner Accent Framing
        tl_tab = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, 0, 0, Inches(0.8), Inches(0.05))
        tl_tab.fill.solid()
        tl_tab.fill.fore_color.rgb = COLOR_TEXT_LIGHT
        tl_tab.line.fill.background()

        tr_tab = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, SLIDE_WIDTH - Inches(1.5), 0, Inches(1.5), Inches(0.06))
        tr_tab.fill.solid()
        tr_tab.fill.fore_color.rgb = COLOR_BLUE_PRIMARY
        tr_tab.line.fill.background()

        bl_tab = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, 0, SLIDE_HEIGHT - Inches(0.04), Inches(0.8), Inches(0.04))
        bl_tab.fill.solid()
        bl_tab.fill.fore_color.rgb = COLOR_BLUE_SKY
        bl_tab.line.fill.background()

        br_tab = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, SLIDE_WIDTH - Inches(1.0), SLIDE_HEIGHT - Inches(0.04), Inches(1.0), Inches(0.04))
        br_tab.fill.solid()
        br_tab.fill.fore_color.rgb = COLOR_EMERALD
        br_tab.line.fill.background()

        return slide

    def set_speaker_notes(self, slide, notes_text):
        """Injects detailed speaker notes into the slide's notes slide."""
        slide.notes_slide.notes_text_frame.text = notes_text.strip()

    def add_header(self, slide, badge_text, title_text, subtitle_text,
                   badge_color=COLOR_BLUE_PRIMARY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER):
        # Category Badge
        badge_w = Inches(5.8)
        badge_h = Inches(0.28)
        badge = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, MARGIN_X, Inches(0.38), badge_w, badge_h)
        badge.fill.solid()
        badge.fill.fore_color.rgb = badge_bg
        badge.line.color.rgb = badge_border
        badge.line.width = Pt(1)
        tf_b = badge.text_frame
        tf_b.word_wrap = False
        tf_b.margin_left = tf_b.margin_top = tf_b.margin_right = tf_b.margin_bottom = 0
        p_b = tf_b.paragraphs[0]
        p_b.text = badge_text.upper()
        p_b.font.name = FONT_FAMILY
        p_b.font.size = Pt(8.8)
        p_b.font.bold = True
        p_b.font.color.rgb = badge_color
        p_b.alignment = PP_ALIGN.CENTER

        # Title
        tx_title = slide.shapes.add_textbox(MARGIN_X, Inches(0.70), CONTENT_WIDTH, Inches(0.48))
        tf_t = tx_title.text_frame
        tf_t.word_wrap = True
        tf_t.margin_left = tf_t.margin_top = tf_t.margin_right = tf_t.margin_bottom = 0
        p_t = tf_t.paragraphs[0]
        p_t.text = title_text
        p_t.font.name = FONT_FAMILY
        p_t.font.size = Pt(20)
        p_t.font.bold = True
        p_t.font.color.rgb = COLOR_DARK_SLATE

        # Subtitle
        tx_sub = slide.shapes.add_textbox(MARGIN_X, Inches(1.18), CONTENT_WIDTH, Inches(0.32))
        tf_s = tx_sub.text_frame
        tf_s.word_wrap = True
        tf_s.margin_left = tf_s.margin_top = tf_s.margin_right = tf_s.margin_bottom = 0
        p_s = tf_s.paragraphs[0]
        p_s.text = subtitle_text
        p_s.font.name = FONT_FAMILY
        p_s.font.size = Pt(11.0)
        p_s.font.color.rgb = COLOR_TEXT_MUTED

    def add_footer(self, slide, slide_num, total_slides=27, note_text=""):
        # Divider line
        line = slide.shapes.add_connector(
            MSO_CONNECTOR.STRAIGHT,
            MARGIN_X, Inches(6.82),
            MARGIN_X + CONTENT_WIDTH, Inches(6.82)
        )
        line.line.color.rgb = COLOR_CARD_BORDER
        line.line.width = Pt(1)

        # Footer Left Note
        tx_l = slide.shapes.add_textbox(MARGIN_X, Inches(6.88), Inches(8.0), Inches(0.3))
        tf_l = tx_l.text_frame
        tf_l.word_wrap = False
        tf_l.margin_left = tf_l.margin_top = tf_l.margin_right = tf_l.margin_bottom = 0
        p_l = tf_l.paragraphs[0]
        default_note = "LIVA BANKING HARNESS • INNOSTART 2026 DEMO DAY"
        p_l.text = note_text if note_text else default_note
        p_l.font.name = FONT_FAMILY
        p_l.font.size = Pt(9.5)
        p_l.font.color.rgb = COLOR_TEXT_LIGHT

        # Footer Right Page
        tx_r = slide.shapes.add_textbox(MARGIN_X + Inches(8.2), Inches(6.88), Inches(3.533), Inches(0.3))
        tf_r = tx_r.text_frame
        tf_r.word_wrap = False
        tf_r.margin_left = tf_r.margin_top = tf_r.margin_right = tf_r.margin_bottom = 0
        p_r = tf_r.paragraphs[0]
        p_r.text = f"Trang {slide_num:02d} / {total_slides} • INNOSTART 2026"
        p_r.font.name = FONT_FAMILY
        p_r.font.size = Pt(9.5)
        p_r.font.bold = True
        p_r.font.color.rgb = COLOR_TEXT_LIGHT
        p_r.alignment = PP_ALIGN.RIGHT

    def add_card(self, slide, left, top, width, height, bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER):
        card = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left, top, width, height)
        card.fill.solid()
        card.fill.fore_color.rgb = bg_color
        card.line.color.rgb = border_color
        card.line.width = Pt(1)
        return card

    def add_photo_panel(self, slide, photo_path, left, top, width, height, caption=None):
        """Embeds architectural photography with clean frame and caption."""
        if os.path.exists(photo_path):
            slide.shapes.add_picture(photo_path, left, top, width, height)
            if caption:
                cap_h = Inches(0.40)
                cap_box = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left, top + height - cap_h, width, cap_h)
                cap_box.fill.solid()
                cap_box.fill.fore_color.rgb = COLOR_WHITE
                cap_box.line.color.rgb = COLOR_CARD_BORDER
                cap_box.line.width = Pt(1)
                tf = cap_box.text_frame
                tf.word_wrap = True
                p = tf.paragraphs[0]
                p.text = caption
                p.font.name = FONT_FAMILY
                p.font.size = Pt(9.0)
                p.font.bold = True
                p.font.color.rgb = COLOR_TEXT_MUTED
                p.alignment = PP_ALIGN.CENTER

    def add_square_badge_card(self, slide, left, top, width, height,
                               badge_text, badge_color, title, bullets,
                               stat_val=None, stat_lbl=None,
                               bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER):
        """Feature card with square icon badge at top-left."""
        self.add_card(slide, left, top, width, height, bg_color=bg_color, border_color=border_color)

        badge_size = Inches(0.52)
        sq = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left + Inches(0.20), top + Inches(0.20), badge_size, badge_size)
        sq.fill.solid()
        sq.fill.fore_color.rgb = badge_color
        sq.line.fill.background()
        tf_sq = sq.text_frame
        tf_sq.margin_left = tf_sq.margin_top = tf_sq.margin_right = tf_sq.margin_bottom = 0
        p_sq = tf_sq.paragraphs[0]
        p_sq.text = badge_text
        p_sq.font.name = FONT_FAMILY
        p_sq.font.size = Pt(9.5)
        p_sq.font.bold = True
        p_sq.font.color.rgb = COLOR_WHITE
        p_sq.alignment = PP_ALIGN.CENTER

        tx_t = slide.shapes.add_textbox(left + Inches(0.80), top + Inches(0.18), width - Inches(0.95), Inches(0.58))
        tf_t = tx_t.text_frame
        tf_t.word_wrap = True
        tf_t.margin_left = tf_t.margin_top = tf_t.margin_right = tf_t.margin_bottom = 0
        p_t = tf_t.paragraphs[0]
        p_t.text = title
        p_t.font.name = FONT_FAMILY
        p_t.font.size = Pt(11.5)
        p_t.font.bold = True
        p_t.font.color.rgb = COLOR_DARK_SLATE

        content_top = top + Inches(0.80)
        content_h = height - Inches(0.95)

        if stat_val:
            tx_st = slide.shapes.add_textbox(left + Inches(0.20), content_top, width - Inches(0.40), Inches(0.68))
            tf_st = tx_st.text_frame
            tf_st.word_wrap = True
            tf_st.margin_left = tf_st.margin_top = tf_st.margin_right = tf_st.margin_bottom = 0
            p_val = tf_st.paragraphs[0]
            p_val.text = stat_val
            p_val.font.name = FONT_FAMILY
            p_val.font.size = Pt(26)
            p_val.font.bold = True
            p_val.font.color.rgb = badge_color
            if stat_lbl:
                p_lbl = tf_st.add_paragraph()
                p_lbl.text = stat_lbl.upper()
                p_lbl.font.name = FONT_FAMILY
                p_lbl.font.size = Pt(8.5)
                p_lbl.font.bold = True
                p_lbl.font.color.rgb = COLOR_DARK_SLATE
            content_top += Inches(0.72)
            content_h -= Inches(0.72)

        tx_b = slide.shapes.add_textbox(left + Inches(0.20), content_top, width - Inches(0.40), content_h)
        tf_b = tx_b.text_frame
        tf_b.word_wrap = True
        tf_b.margin_left = tf_b.margin_top = tf_b.margin_right = tf_b.margin_bottom = 0

        first = True
        for item in bullets:
            p_i = tf_b.paragraphs[0] if first else tf_b.add_paragraph()
            first = False
            p_i.space_after = Pt(4)
            if isinstance(item, tuple):
                pre, desc = item
                r_p = p_i.add_run()
                r_p.text = f"• {pre}: "
                r_p.font.name = FONT_FAMILY
                r_p.font.size = Pt(9.5)
                r_p.font.bold = True
                r_p.font.color.rgb = COLOR_DARK_SLATE

                r_d = p_i.add_run()
                r_d.text = desc
                r_d.font.name = FONT_FAMILY
                r_d.font.size = Pt(9.0)
                r_d.font.color.rgb = COLOR_TEXT_BODY
            else:
                r = p_i.add_run()
                r.text = f"• {item}"
                r.font.name = FONT_FAMILY
                r.font.size = Pt(9.0)
                r.font.color.rgb = COLOR_TEXT_BODY

    def add_stacked_metric_stripes(self, slide, left, top, width, height, stripes):
        """Stacked horizontal metric bars/stripes."""
        n = len(stripes)
        gap = Inches(0.12)
        stripe_h = (height - (n - 1) * gap) / n

        for i, (stat_val, stat_lbl, stat_desc, color_accent, is_tinted) in enumerate(stripes):
            sy = top + i * (stripe_h + gap)
            bg = COLOR_STRIPE_BLUE if is_tinted else COLOR_CARD_BG
            bd = COLOR_STRIPE_BORDER if is_tinted else COLOR_CARD_BORDER

            self.add_card(slide, left, sy, width, stripe_h, bg_color=bg, border_color=bd)

            bar = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left, sy, Inches(0.08), stripe_h)
            bar.fill.solid()
            bar.fill.fore_color.rgb = color_accent
            bar.line.fill.background()

            tx_s = slide.shapes.add_textbox(left + Inches(0.24), sy + Inches(0.10), Inches(2.2), stripe_h - Inches(0.20))
            tf_s = tx_s.text_frame
            tf_s.word_wrap = True
            tf_s.margin_left = tf_s.margin_top = tf_s.margin_right = tf_s.margin_bottom = 0
            p_s = tf_s.paragraphs[0]
            p_s.text = stat_val
            p_s.font.name = FONT_FAMILY
            p_s.font.size = Pt(28)
            p_s.font.bold = True
            p_s.font.color.rgb = color_accent

            tx_c = slide.shapes.add_textbox(left + Inches(2.55), sy + Inches(0.10), width - Inches(2.75), stripe_h - Inches(0.20))
            tf_c = tx_c.text_frame
            tf_c.word_wrap = True
            tf_c.margin_left = tf_c.margin_top = tf_c.margin_right = tf_c.margin_bottom = 0
            p_lbl = tf_c.paragraphs[0]
            p_lbl.text = stat_lbl.upper()
            p_lbl.font.name = FONT_FAMILY
            p_lbl.font.size = Pt(10.0)
            p_lbl.font.bold = True
            p_lbl.font.color.rgb = COLOR_DARK_SLATE
            p_lbl.space_after = Pt(2)

            p_desc = tf_c.add_paragraph()
            p_desc.text = stat_desc
            p_desc.font.name = FONT_FAMILY
            p_desc.font.size = Pt(9.2)
            p_desc.font.color.rgb = COLOR_TEXT_BODY

    def add_step_chevron(self, slide, left, top, width, height, steps):
        """Clean horizontal chevrons with container cards below."""
        n = len(steps)
        gap = Inches(0.08)
        ch_w = (width - (n - 1) * gap) / n
        ch_h = Inches(0.48)

        for i, (step_num, step_title, step_desc, step_color) in enumerate(steps):
            cx = left + i * (ch_w + gap)

            ch = slide.shapes.add_shape(MSO_SHAPE.CHEVRON, cx, top, ch_w, ch_h)
            ch.fill.solid()
            ch.fill.fore_color.rgb = step_color
            ch.line.fill.background()
            tf_ch = ch.text_frame
            tf_ch.word_wrap = False
            tf_ch.margin_left = tf_ch.margin_top = tf_ch.margin_right = tf_ch.margin_bottom = 0
            p_ch = tf_ch.paragraphs[0]
            step_str = str(step_num)
            p_ch.text = step_str if (step_str.startswith("Q") or step_str.startswith("QUÝ") or step_str.startswith("BƯỚC") or step_str.startswith("GIAI")) else f"BƯỚC {step_num}"
            p_ch.font.name = FONT_FAMILY
            p_ch.font.size = Pt(9.5)
            p_ch.font.bold = True
            p_ch.font.color.rgb = COLOR_WHITE
            p_ch.alignment = PP_ALIGN.CENTER

            card_y = top + ch_h + Inches(0.10)
            card_h = height - ch_h - Inches(0.10)
            self.add_card(slide, cx, card_y, ch_w, card_h, bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER)

            tx = slide.shapes.add_textbox(cx + Inches(0.14), card_y + Inches(0.14), ch_w - Inches(0.28), card_h - Inches(0.28))
            tf = tx.text_frame
            tf.word_wrap = True
            tf.margin_left = tf.margin_top = tf.margin_right = tf.margin_bottom = 0
            p_t = tf.paragraphs[0]
            p_t.text = step_title
            p_t.font.name = FONT_FAMILY
            p_t.font.size = Pt(10.5)
            p_t.font.bold = True
            p_t.font.color.rgb = COLOR_DARK_SLATE
            p_t.space_after = Pt(4)

            if isinstance(step_desc, list):
                for item in step_desc:
                    p_d = tf.add_paragraph()
                    p_d.space_after = Pt(3)
                    if isinstance(item, tuple):
                        pre, desc = item
                        r_p = p_d.add_run()
                        r_p.text = f"• {pre}: "
                        r_p.font.name = FONT_FAMILY
                        r_p.font.size = Pt(8.8)
                        r_p.font.bold = True
                        r_p.font.color.rgb = COLOR_DARK_SLATE

                        r_d = p_d.add_run()
                        r_d.text = desc
                        r_d.font.name = FONT_FAMILY
                        r_d.font.size = Pt(8.5)
                        r_d.font.color.rgb = COLOR_TEXT_BODY
                    else:
                        r = p_d.add_run()
                        r.text = f"• {item}"
                        r.font.name = FONT_FAMILY
                        r.font.size = Pt(8.5)
                        r.font.color.rgb = COLOR_TEXT_BODY
            else:
                p_d = tf.add_paragraph()
                p_d.text = step_desc
                p_d.font.name = FONT_FAMILY
                p_d.font.size = Pt(9.0)
                p_d.font.color.rgb = COLOR_TEXT_BODY

    def add_chevron_process(self, slide, left, top, width, height, steps):
        """Alias to add_step_chevron."""
        return self.add_step_chevron(slide, left, top, width, height, steps)

    def add_compare_grid(self, slide, left, top, width, height, left_data, right_data):
        """
        Side-by-side high-contrast CompareGrid (Trước khi có LIVA vs Khi có LIVA).
        100% native PowerPoint shapes with colored headers and contrasting item cards.
        """
        gap = Inches(0.25)
        col_w = (width - gap) / 2
        header_h = Inches(0.68)

        for data, cx in [(left_data, left), (right_data, left + col_w + gap)]:
            self.add_card(slide, cx, top, col_w, height, bg_color=data['bg_color'], border_color=data['border_color'])

            hdr = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, cx + Inches(0.10), top + Inches(0.10), col_w - Inches(0.20), header_h)
            hdr.fill.solid()
            hdr.fill.fore_color.rgb = data['header_color']
            hdr.line.fill.background()
            tf_h = hdr.text_frame
            tf_h.word_wrap = True
            tf_h.margin_left = tf_h.margin_top = tf_h.margin_right = tf_h.margin_bottom = 0
            p_ht = tf_h.paragraphs[0]
            p_ht.text = data['title'].upper()
            p_ht.font.name = FONT_FAMILY
            p_ht.font.size = Pt(11.0)
            p_ht.font.bold = True
            p_ht.font.color.rgb = COLOR_WHITE
            p_ht.alignment = PP_ALIGN.CENTER

            if 'subtitle' in data and data['subtitle']:
                p_hs = tf_h.add_paragraph()
                p_hs.text = data['subtitle']
                p_hs.font.name = FONT_FAMILY
                p_hs.font.size = Pt(8.5)
                p_hs.font.color.rgb = COLOR_WHITE
                p_hs.alignment = PP_ALIGN.CENTER

            items = data['items']
            n_items = len(items)
            item_top = top + header_h + Inches(0.18)
            item_gap = Inches(0.10)
            avail_h = height - header_h - Inches(0.30)
            box_h = (avail_h - (n_items - 1) * item_gap) / n_items

            for i, (it_title, it_desc) in enumerate(items):
                iy = item_top + i * (box_h + item_gap)
                item_box = slide.shapes.add_shape(
                    MSO_SHAPE.ROUNDED_RECTANGLE,
                    cx + Inches(0.14), iy, col_w - Inches(0.28), box_h
                )
                item_box.fill.solid()
                item_box.fill.fore_color.rgb = COLOR_WHITE
                item_box.line.color.rgb = data['border_color']
                item_box.line.width = Pt(1)

                tf_ib = item_box.text_frame
                tf_ib.word_wrap = True
                tf_ib.margin_left = Inches(0.14)
                tf_ib.margin_right = Inches(0.14)
                tf_ib.margin_top = Inches(0.08)
                tf_ib.margin_bottom = Inches(0.08)

                p_it = tf_ib.paragraphs[0]
                p_it.text = f"{data.get('bullet_icon', '•')} {it_title}"
                p_it.font.name = FONT_FAMILY
                p_it.font.size = Pt(10.5)
                p_it.font.bold = True
                p_it.font.color.rgb = data['header_color']
                p_it.space_after = Pt(2)

                p_id = tf_ib.add_paragraph()
                p_id.text = it_desc
                p_id.font.name = FONT_FAMILY
                p_id.font.size = Pt(9.0)
                p_id.font.color.rgb = COLOR_TEXT_BODY

    def add_donut_chart(self, slide, left, top, width, height, categories, values, colors, title=None):
        """Native PowerPoint Doughnut Chart with legend and data labels."""
        self.add_card(slide, left, top, width, height, bg_color=COLOR_WHITE, border_color=COLOR_CARD_BORDER)

        title_h = Inches(0.40) if title else Inches(0)
        if title:
            tx_t = slide.shapes.add_textbox(left + Inches(0.2), top + Inches(0.12), width - Inches(0.4), title_h)
            tf_t = tx_t.text_frame
            tf_t.word_wrap = True
            tf_t.margin_left = tf_t.margin_top = tf_t.margin_right = tf_t.margin_bottom = 0
            p_t = tf_t.paragraphs[0]
            p_t.text = title.upper()
            p_t.font.name = FONT_FAMILY
            p_t.font.size = Pt(9.5)
            p_t.font.bold = True
            p_t.font.color.rgb = COLOR_DARK_SLATE
            p_t.alignment = PP_ALIGN.CENTER

        chart_y = top + title_h + Inches(0.08)
        chart_h = height - title_h - Inches(0.16)

        data = CategoryChartData()
        data.categories = categories
        data.add_series(title or "Tỷ lệ", values)

        chart_shape = slide.shapes.add_chart(
            XL_CHART_TYPE.DOUGHNUT, left + Inches(0.15), chart_y, width - Inches(0.3), chart_h, data
        )
        chart = chart_shape.chart
        chart.has_legend = True
        chart.legend.position = XL_LEGEND_POSITION.BOTTOM
        chart.legend.include_in_layout = False
        chart.legend.font.name = FONT_FAMILY
        chart.legend.font.size = Pt(8.5)
        chart.legend.font.color.rgb = COLOR_TEXT_BODY

        series = chart.series[0]
        series.has_data_labels = True
        for idx, point in enumerate(series.points):
            point.format.fill.solid()
            point.format.fill.fore_color.rgb = colors[idx % len(colors)]

        return chart_shape

    # -------------------------------------------------------------------------
    # MODULE A: VÒNG 1 — 5-PHÚT PITCHING ĐỀ ÁN (SLIDES 01 - 13)
    # -------------------------------------------------------------------------

    # =========================================================================
    # SLIDE 01: TIÊU ĐỀ & ĐỊNH VỊ SẢN PHẨM (SPLITHERO)
    # =========================================================================
    def build_slide_01_title(self):
        slide = self.add_blank_slide()
        top_y = Inches(1.50)
        height = Inches(5.05)
        left_w = Inches(6.90)

        # Header Badge
        badge_w = Inches(4.8)
        badge_h = Inches(0.28)
        badge = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, MARGIN_X, Inches(0.42), badge_w, badge_h)
        badge.fill.solid()
        badge.fill.fore_color.rgb = COLOR_STRIPE_BLUE
        badge.line.color.rgb = COLOR_STRIPE_BORDER
        badge.line.width = Pt(1)
        tf_b = badge.text_frame
        p_b = tf_b.paragraphs[0]
        p_b.text = "INNOSTART 2026 DEMO DAY • ROUND 1 PITCH"
        p_b.font.name = FONT_FAMILY
        p_b.font.size = Pt(9.0)
        p_b.font.bold = True
        p_b.font.color.rgb = COLOR_BLUE_PRIMARY
        p_b.alignment = PP_ALIGN.CENTER

        # Title Box
        tx_title = slide.shapes.add_textbox(MARGIN_X, Inches(0.78), left_w, Inches(1.30))
        tf_t = tx_title.text_frame
        tf_t.word_wrap = True
        tf_t.margin_left = tf_t.margin_top = tf_t.margin_right = tf_t.margin_bottom = 0
        p0 = tf_t.paragraphs[0]
        p0.text = "LIVA BANKING HARNESS"
        p0.font.name = FONT_FAMILY
        p0.font.size = Pt(28)
        p0.font.bold = True
        p0.font.color.rgb = COLOR_BLUE_NAVY

        p1 = tf_t.add_paragraph()
        p1.text = "Đai An Toàn AI & Tháp Canh Nguồn Vốn Doanh Nghiệp 24/7"
        p1.font.name = FONT_FAMILY
        p1.font.size = Pt(13.5)
        p1.font.bold = True
        p1.font.color.rgb = COLOR_BLUE_PRIMARY
        p1.space_before = Pt(2)

        p2 = tf_t.add_paragraph()
        p2.text = "Giải pháp tự động hóa đối soát và tối ưu dòng tiền đa ngân hàng chạy 100% tại chỗ cho doanh nghiệp quy mô 100–500 tỷ VNĐ."
        p2.font.name = FONT_FAMILY
        p2.font.size = Pt(10.0)
        p2.font.color.rgb = COLOR_TEXT_MUTED
        p2.space_before = Pt(3)

        # 3 Stacked Metric Stripes
        stripes = [
            ("100%", "ZERO DATA EGRESS", "Vận hành hoàn toàn tại chỗ — Không gửi 1 byte dữ liệu tài chính lên đám mây.", COLOR_BLUE_NAVY, False),
            ("19 Giây", "TỐC ĐỘ ĐỐI SOÁT", "Bóc tách và đối chiếu 50.000 giao dịch tức thì trên máy tính văn phòng.", COLOR_BLUE_PRIMARY, True),
            ("0%", "ẢO GIÁC SỐ HỌC", "Động cơ toán học xác định bảo đảm chính xác từng đồng trước khi hạch toán.", COLOR_EMERALD, False),
        ]
        self.add_stacked_metric_stripes(slide, MARGIN_X, Inches(2.20), left_w, Inches(3.25), stripes)

        # Bottom Team Card
        t_card = self.add_card(slide, MARGIN_X, Inches(5.60), left_w, Inches(0.95), bg_color=COLOR_STRIPE_BLUE, border_color=COLOR_STRIPE_BORDER)
        tx_tm = slide.shapes.add_textbox(MARGIN_X + Inches(0.18), Inches(5.65), left_w - Inches(0.36), Inches(0.85))
        tf_tm = tx_tm.text_frame
        tf_tm.word_wrap = True
        tf_tm.margin_left = tf_tm.margin_top = tf_tm.margin_right = tf_tm.margin_bottom = 0
        p_tm1 = tf_tm.paragraphs[0]
        p_tm1.text = "ĐỘI NGŨ SÁNG LẬP LIVA BANKING • DEMO DAY 2026"
        p_tm1.font.name = FONT_FAMILY
        p_tm1.font.size = Pt(9.0)
        p_tm1.font.bold = True
        p_tm1.font.color.rgb = COLOR_BLUE_NAVY

        p_tm2 = tf_tm.add_paragraph()
        p_tm2.text = "5 Nhà sáng lập 100% Full-time • Làm chủ công nghệ lõi • Am hiểu nghiệp vụ tài chính bản địa."
        p_tm2.font.name = FONT_FAMILY
        p_tm2.font.size = Pt(8.5)
        p_tm2.font.color.rgb = COLOR_TEXT_BODY

        # Right Architectural Photography Panel
        photo_x = MARGIN_X + left_w + Inches(0.25)
        photo_w = CONTENT_WIDTH - left_w - Inches(0.25)
        photo_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets", "glass_skyscraper.jpg")
        self.add_photo_panel(
            slide, photo_path, photo_x, Inches(0.42), photo_w, Inches(6.15),
            caption="THIẾT LẬP CHUẨN MỰC TỰ ĐỘNG HÓA NGÂN QUỸ TẠI VIỆT NAM"
        )

        self.add_footer(slide, 1, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(1))

    # =========================================================================
    # SLIDE 02: BỐI CẢNH VÀ NỖI ĐAU THỰC TẾ (FEATURE CARDS)
    # =========================================================================
    def build_slide_02_problem(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • VẤN ĐỀ THỰC TẾ • NỖI ĐAU THỊ TRƯỜNG",
            title_text="NỖI ĐAU ĐỐI SOÁT THỦ CÔNG: 2-4H MỖI SÁNG & VÙNG MÙ THANH KHOẢN T+3",
            subtitle_text="Doanh nghiệp quy mô 100–500 tỷ VNĐ đang lãng phí hàng trăm giờ lao động và đối mặt rủi ro thâm hụt tiền mặt.",
            badge_color=COLOR_DANGER, badge_bg=COLOR_RED_BG, badge_border=COLOR_RED_BORDER
        )

        card_w = Inches(3.777)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="2-4H", badge_color=COLOR_DANGER,
            title="Lãng Phí Thời Gian Đối Soát",
            bullets=[
                ("Mở 15 file sao kê", "Mỗi sáng kế toán phải đăng nhập vào 15 cổng Internet Banking tải file sao kê thủ công."),
                ("Dò từng dòng giao dịch", "Mất 2 đến 4 giờ làm việc tập trung cao độ để so khớp từng khoản thu chi với sổ cái."),
                ("Sai lệch tê liệt", "Chỉ cần lệch đúng 1.000 đồng tiền phí, cả phòng kế toán phải thức trắng đêm tìm kiếm sai lệch.")
            ],
            stat_val="2 - 4 Giờ", stat_lbl="Thời gian đối soát mỗi sáng"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="15+", badge_color=COLOR_AMBER,
            title="Mẫu Sao Kê Vỡ Mụn",
            bullets=[
                ("Không chuẩn hóa", "Mỗi ngân hàng xuất 1 định dạng Excel khác nhau; tự ý chèn thêm cột hoặc thay đổi vị trí ô."),
                ("Mù chữ viết tắt", "Diễn giải chuyển khoản viết tắt không dấu như 'CK HD 101 bot tien thue' khiến Excel hoàn toàn bó tay."),
                ("Gãy nát công thức", "Các hàm VLOOKUP hoặc macro VBA liên tục báo lỗi khi có thay đổi cấu trúc bảng biểu.")
            ],
            stat_val="15+ Mẫu", stat_lbl="Định dạng sao kê phân mảnh"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="T+3", badge_color=COLOR_BLUE_NAVY,
            title="Vùng Mù Thanh Khoản CFO",
            bullets=[
                ("Mất kiểm soát số dư", "CFO không nắm được số dư khả dụng thực tế giữa các ngân hàng trong 24 đến 72 giờ."),
                ("Bị động thanh toán", "Rủi ro phạt chậm trả hoặc hụt tiền mặt đột xuất do lệnh thanh toán nhà cung cấp đến bất ngờ."),
                ("Lãng phí lãi suất", "Dư tiền ở ngân hàng lãi suất thấp trong khi vẫn phải đi vay thấu chi lãi cao ở ngân hàng khác.")
            ],
            stat_val="T+1 ➔ T+3", stat_lbl="Độ trễ nắm bắt dòng tiền"
        )

        self.add_footer(slide, 2, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(2))

    # =========================================================================
    # SLIDE 03: RÀO CẢN PHÁP LÝ KHẮT KHE (STATBOX & FEATURE CARDS)
    # =========================================================================
    def build_slide_03_regulatory(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • BỐI CẢNH PHÁP LÝ • NGHỊ ĐỊNH 13 & THÔNG TƯ 09",
            title_text="RÀO CẢN PHÁP LÝ KHẮT KHE: NGHỊ ĐỊNH 13 & ÁN PHẠT 5% DOANH THU",
            subtitle_text="Các giải pháp Cloud AI truyền thống hoàn toàn bị cấm tiếp cận dữ liệu tài chính ngân hàng doanh nghiệp.",
            badge_color=COLOR_DANGER, badge_bg=COLOR_RED_BG, badge_border=COLOR_RED_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)

        # Left Column: Regulatory Stat Box
        left_w = Inches(4.2)
        stripes = [
            ("5%", "MỨC PHẠT NGHỊ ĐỊNH 13", "Mức phạt tối đa trên tổng doanh thu nếu để lộ lọt bí mật dữ liệu tài chính cá nhân.", COLOR_DANGER, True),
            ("100%", "CẤM CLOUD CÔNG CỘNG", "Thông tư 09/2020/TT-NHNN nghiêm cấm gửi dữ liệu sao kê tài chính lên máy chủ đám mây công cộng.", COLOR_BLUE_NAVY, False),
        ]
        self.add_stacked_metric_stripes(slide, MARGIN_X, top_y, left_w, height, stripes)

        # Right Column: 2 Legal Framework Detail Cards
        right_x = MARGIN_X + left_w + Inches(0.20)
        right_w = CONTENT_WIDTH - left_w - Inches(0.20)
        c_h = (height - Inches(0.15)) / 2

        self.add_square_badge_card(
            slide, right_x, top_y, right_w, c_h,
            badge_text="NĐ 13", badge_color=COLOR_DANGER,
            title="Nghị Định 13/2023/NĐ-CP Về Bảo Vệ Dữ Liệu Cá Nhân",
            bullets=[
                ("Xử phạt nghiêm khắc", "Mức xử phạt lên đến 5% tổng doanh thu năm trước liền kề nếu phát hiện chuyển dữ liệu ra nước ngoài trái phép."),
                ("Yêu cầu DPIA", "Bắt buộc phải có hồ sơ Đánh giá tác động xử lý dữ liệu (DPIA) gửi Bộ Công an trước khi xử lý dữ liệu nhạy cảm."),
                ("Lưu trữ tại chỗ", "Dữ liệu giao dịch tài chính doanh nghiệp bắt buộc phải được kiểm soát toàn vẹn trên hạ tầng máy tính nội bộ.")
            ]
        )

        self.add_square_badge_card(
            slide, right_x, top_y + c_h + Inches(0.15), right_w, c_h,
            badge_text="TT 09", badge_color=COLOR_BLUE_NAVY,
            title="Thông Tư 09/2020/TT-NHNN Về An Toàn Hệ Thống Ngân Hàng",
            bullets=[
                ("Cấp độ an toàn cao", "Hệ thống thông tin quản trị ngân quỹ được xếp vào cấp độ bảo mật nghiêm ngặt nhất của ngành ngân hàng."),
                ("Cấm dịch vụ công cộng", "Nghiêm cấm chia sẻ dữ liệu giao dịch tài khoản khách hàng lên các dịch vụ đám mây công cộng bên ngoài."),
                ("Nhật ký kiểm toán", "Yêu cầu lưu trữ nhật ký truy cập và đối soát bất biến phục vụ công tác thanh tra độc lập định kỳ.")
            ]
        )

        self.add_footer(slide, 3, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(3))

    # =========================================================================
    # SLIDE 04: KHÁCH HÀNG MỤC TIÊU (FEATURE CARDS)
    # =========================================================================
    def build_slide_04_customer(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • KHÁCH HÀNG MỤC TIÊU • IDEAL CUSTOMER PROFILE",
            title_text="CHÂN DUNG KHÁCH HÀNG MỤC TIÊU: DOANH NGHIỆP 100-500 TỶ VNĐ & 5+ NGÂN HÀNG",
            subtitle_text="Tập trung vào phân khúc doanh nghiệp vừa và lớn có khối lượng giao dịch phức tạp nhưng thiếu công cụ tự động.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(3.777)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="QUY MÔ", badge_color=COLOR_BLUE_NAVY,
            title="Quy Mô Doanh Nghiệp Mục Tiêu",
            bullets=[
                ("Doanh thu năm", "Từ 100 đến 500 tỷ VNĐ; thực hiện trên 2.000 đến 20.000 giao dịch ngân hàng mỗi tháng."),
                ("Mạng lưới đa tài khoản", "Mở từ 5 đến 15 tài khoản tại các ngân hàng thương mại để tối ưu hạn mức tín dụng."),
                ("Đội ngũ kế toán", "Phòng kế toán từ 3 đến 8 nhân sự, thường xuyên quá tải vào các kỳ đối soát cao điểm.")
            ],
            stat_val="100 - 500 Tỷ", stat_lbl="Doanh thu hàng năm"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="NGÀNH", badge_color=COLOR_BLUE_PRIMARY,
            title="Ngành Trọng Tâm Ưu Tiên",
            bullets=[
                ("Bán lẻ & Phân phối", "Chuỗi bán lẻ, siêu thị mini với hàng ngàn giao dịch thanh toán QR code và quẹt thẻ mỗi ngày."),
                ("Sản xuất & Vận tải", "Doanh nghiệp logistics, sản xuất linh kiện có luồng tiền ra vào liên tục với hàng trăm nhà cung cấp."),
                ("Thương mại điện tử", "Các sàn và chuỗi kinh doanh online đối mặt bài toán đối soát COD phức tạp từ nhiều đối tác.")
            ],
            stat_val="3 Ngành", stat_lbl="Trọng tâm chuyển đổi nhanh"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="SẴN SÀNG", badge_color=COLOR_EMERALD,
            title="Mức Độ Sẵn Sàng Chi Trả",
            bullets=[
                ("Ngân sách hàng năm", "Sẵn sàng đầu tư từ $3.000 đến $10.000 USD mỗi năm cho phần mềm chuyên dụng."),
                ("Thời gian quyết định", "Chu kỳ bán hàng nhanh từ 2 đến 4 tuần do CFO và Kế toán trưởng trực tiếp ra quyết định."),
                ("Nhu cầu bức thiết", "Cần lời giải tức thì để giải phóng sức lao động và loại bỏ hoàn toàn rủi ro thất thoát.")
            ],
            stat_val="$3k - $10k", stat_lbl="Ngân sách phần mềm / năm"
        )

        self.add_footer(slide, 4, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(4))

    # =========================================================================
    # SLIDE 05: GIẢI PHÁP TỔNG THỂ (SPLITHERO)
    # =========================================================================
    def build_slide_05_solution(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • GIẢI PHÁP ĐỘT PHÁ • TỔNG QUAN SẢN PHẨM",
            title_text="LIVA BANKING HARNESS: ĐAI AN TOÀN AI & THÁP CANH NGUỒN VỐN TẠI CHỖ",
            subtitle_text="Hệ thống tự động hóa ngân quỹ toàn diện, kết hợp tốc độ tức thì và bảo mật tuyệt đối tại máy trạm.",
            badge_color=COLOR_BLUE_PRIMARY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)
        left_w = Inches(6.90)

        # Vision Banner Box
        v_h = Inches(1.15)
        self.add_card(slide, MARGIN_X, top_y, left_w, v_h, bg_color=COLOR_STRIPE_BLUE, border_color=COLOR_STRIPE_BORDER)

        tx_v = slide.shapes.add_textbox(MARGIN_X + Inches(0.20), top_y + Inches(0.12), left_w - Inches(0.40), v_h - Inches(0.24))
        tf_v = tx_v.text_frame
        tf_v.word_wrap = True
        tf_v.margin_left = tf_v.margin_top = tf_v.margin_right = tf_v.margin_bottom = 0

        p_vt = tf_v.paragraphs[0]
        p_vt.text = "ĐAI AN TOÀN AI & THÁP CANH NGUỒN VỐN DOANH NGHIỆP"
        p_vt.font.name = FONT_FAMILY
        p_vt.font.size = Pt(8.8)
        p_vt.font.bold = True
        p_vt.font.color.rgb = COLOR_BLUE_PRIMARY
        p_vt.space_after = Pt(2)

        p_vc = tf_v.add_paragraph()
        p_vc.text = "Tự động hóa đối soát 35+ ngân hàng, bảo vệ tuyệt đối dữ liệu nội bộ và dự báo thanh khoản 48 giờ không độ trễ."
        p_vc.font.name = FONT_FAMILY
        p_vc.font.size = Pt(11.0)
        p_vc.font.bold = True
        p_vc.font.color.rgb = COLOR_DARK_SLATE

        # 3 Core Solution Pillars
        p_top = top_y + v_h + Inches(0.12)
        p_h = height - v_h - Inches(0.12)
        p_card_h = (p_h - Inches(0.20)) / 3

        pillars = [
            ("GOM & BÓC TÁCH", COLOR_BLUE_NAVY, "Tự Động Bóc Tách Đa Ngân Hàng", "Tự động đọc và chuẩn hóa 35+ mẫu sao kê ngân hàng Việt Nam trong 19 giây, xử lý triệt để chữ viết tắt."),
            ("ĐỐI SOÁT TOÁN HỌC", COLOR_BLUE_PRIMARY, "Động Cơ Đối Soát Không Ảo Giác", "Khớp nối chính xác 99.8% giao dịch với sổ cái kế toán bằng thuật toán toán học xác định; 0% ảo giác số học."),
            ("THÁP CANH 48H", COLOR_EMERALD, "Tháp Canh Thanh Khoản Dự Báo 48 Giờ", "Nhìn xa dòng tiền 48 giờ giúp CFO chủ động điều chuyển vốn giữa các ngân hàng và tối ưu lãi suất CASA.")
        ]

        for i, (badge_text, col, title, desc) in enumerate(pillars):
            py = p_top + i * (p_card_h + Inches(0.10))
            self.add_card(slide, MARGIN_X, py, left_w, p_card_h, bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER)

            # Left accent stripe
            bar = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, MARGIN_X, py, Inches(0.08), p_card_h)
            bar.fill.solid()
            bar.fill.fore_color.rgb = col
            bar.line.fill.background()

            tx_p = slide.shapes.add_textbox(MARGIN_X + Inches(0.20), py + Inches(0.10), left_w - Inches(0.35), p_card_h - Inches(0.20))
            tf_p = tx_p.text_frame
            tf_p.word_wrap = True
            tf_p.margin_left = tf_p.margin_top = tf_p.margin_right = tf_p.margin_bottom = 0

            p_t = tf_p.paragraphs[0]
            p_t.text = title
            p_t.font.name = FONT_FAMILY
            p_t.font.size = Pt(11.0)
            p_t.font.bold = True
            p_t.font.color.rgb = COLOR_DARK_SLATE
            p_t.space_after = Pt(2)

            p_d = tf_p.add_paragraph()
            p_d.text = desc
            p_d.font.name = FONT_FAMILY
            p_d.font.size = Pt(9.0)
            p_d.font.color.rgb = COLOR_TEXT_BODY

        # Right Column: Architectural Photography Panel
        photo_x = MARGIN_X + left_w + Inches(0.20)
        photo_w = CONTENT_WIDTH - left_w - Inches(0.20)
        photo_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets", "architecture_curve.jpg")
        self.add_photo_panel(
            slide, photo_path, photo_x, top_y, photo_w, height,
            caption="KIẾN TRÚC VẬN HÀNH AN TOÀN TẠI CHỖ (ZERO DATA EGRESS)"
        )

        self.add_footer(slide, 5, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(5))

    # =========================================================================
    # SLIDE 06: SO SÁNH HIỆU QUẢ TRƯỚC VÀ SAU KHI CÓ LIVA (COMPAREGRID)
    # =========================================================================
    def build_slide_06_compare_grid(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • SO SÁNH HIỆU QUẢ • TRƯỚC VÀ SAU KHI CÓ LIVA",
            title_text="SO SÁNH HIỆU QUẢ VƯỢT TRỘI: TRƯỚC VÀ SAU KHI TRIỂN KHAI LIVA",
            subtitle_text="Chuyển hóa toàn diện quy trình ngân quỹ từ thủ công phân mảnh sang tự động hóa chính xác tuyệt đối.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        left_data = {
            'title': 'Trước Khi Có LIVA Banking Harness',
            'subtitle': 'Quy trình thủ công rủi ro và phân mảnh',
            'bg_color': COLOR_RED_BG,
            'border_color': COLOR_RED_BORDER,
            'header_color': COLOR_DANGER,
            'bullet_icon': '❌',
            'items': [
                (
                    'Mất 2-4h mỗi sáng',
                    'Mở 15 file sao kê riêng lẻ, căng mắt so khớp thủ công từng dòng với sổ cái kế toán.'
                ),
                (
                    'Vùng mù thanh khoản T+3',
                    'CFO hoàn toàn không biết số dư thực tế giữa các tài khoản để điều phối dòng tiền.'
                ),
                (
                    'Rủi ro thất thoát & Phạt chậm',
                    'Dễ sót giao dịch, chậm trả nợ nhà cung cấp, hoặc hụt tiền mặt đột xuất.'
                )
            ]
        }

        right_data = {
            'title': 'Khi Có LIVA Banking Harness',
            'subtitle': 'Tự động hóa thông minh chạy 100% tại chỗ',
            'bg_color': COLOR_EMERALD_BG,
            'border_color': COLOR_EMERALD_BORDER,
            'header_color': COLOR_EMERALD,
            'bullet_icon': '✅',
            'items': [
                (
                    'Chỉ mất 19 giây',
                    'Tự động gom và đối soát 50.000 dòng giao dịch ngay khi kế toán bật máy tính.'
                ),
                (
                    'Tháp canh dự báo 48h',
                    'Nhìn thấy trước dòng tiền trong 48 giờ để chủ động điều chuyển vốn và tối ưu lãi suất.'
                ),
                (
                    'Chính xác 99.8% & Khóa an toàn',
                    'Động cơ toán học xác định bảo đảm không ảo giác số học, khóa an toàn chuyển người duyệt 0.2%.'
                )
            ]
        }

        self.add_compare_grid(
            slide, MARGIN_X, Inches(1.65), CONTENT_WIDTH, Inches(4.90),
            left_data, right_data
        )

        self.add_footer(slide, 6, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(6))

    # =========================================================================
    # SLIDE 07: HÀNH TRÌNH TRẢI NGHIỆM 3 BƯỚC KHÉP KÍN (STEPCHEVRON)
    # =========================================================================
    def build_slide_07_user_journey(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • TRẢI NGHIỆM NGƯỜI DÙNG • HÀNH TRÌNH 3 BƯỚC KHÉP KÍN",
            title_text="HÀNH TRÌNH 3 BƯỚC KHÉP KÍN: TỰ ĐỘNG HÓA TỪ SAO KÊ ĐẾN HẠCH TOÁN",
            subtitle_text="Quy trình tinh gọn, bảo đảm tính xác định và giữ quyền kiểm soát tối cao cho kế toán trưởng.",
            badge_color=COLOR_BLUE_PRIMARY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        steps = [
            (
                "BƯỚC 1",
                "Tự Động Gom & Bóc Tách",
                [
                    ("Nhận diện thông minh", "Tự động phát hiện và bóc tách dữ liệu từ file sao kê của 35+ ngân hàng Việt Nam."),
                    ("Chuẩn hóa tiếng Việt", "Giải mã chính xác các diễn giải viết tắt không dấu và nội dung chuyển khoản tự do."),
                    ("Bảo mật tại chỗ", "Toàn bộ quá trình diễn ra nội bộ trên máy trạm, không truyền dữ liệu ra bên ngoài.")
                ],
                COLOR_BLUE_NAVY
            ),
            (
                "BƯỚC 2",
                "Đối Soát Số Học Xác Định",
                [
                    ("Khớp nối 99.8%", "So sánh đối chiếu tự động giữa sao kê và các chứng từ trên sổ cái kế toán."),
                    ("Khóa an toàn 0.2%", "Cách ly tức thì các dòng sai lệch số học dù chỉ 1 đồng hoặc nội dung mơ hồ."),
                    ("Không ảo giác", "Thuật toán toán học xác định bảo đảm tuyệt đối không suy đoán số tiền bừa bãi.")
                ],
                COLOR_BLUE_PRIMARY
            ),
            (
                "BƯỚC 3",
                "Đề Xuất & Phê Duyệt Hai Pha",
                [
                    ("Lập dự thảo bút toán", "Chuẩn bị sẵn đề xuất hạch toán và điều chuyển tiền thông minh giữa các tài khoản."),
                    ("Maker-Checker", "Kế toán trưởng hoặc CFO ký duyệt điện tử thì lệnh mới có giá trị pháp lý."),
                    ("Đồng bộ phần mềm", "Tự động cập nhật số liệu vào phần mềm kế toán (MISA, FAST, Bravo) chỉ bằng 1 chạm.")
                ],
                COLOR_EMERALD
            )
        ]

        self.add_step_chevron(slide, MARGIN_X, Inches(1.65), CONTENT_WIDTH, Inches(4.90), steps)

        self.add_footer(slide, 7, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(7))

    # =========================================================================
    # SLIDE 08: CÔNG NGHỆ NÒNG CỐT — KIẾN TRÚC 2 TẦNG (FEATURE CARDS)
    # =========================================================================
    def build_slide_08_core_tech(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • CÔNG NGHỆ NÒNG CỐT • KIẾN TRÚC 2 TẦNG ĐỘC LẬP",
            title_text="KIẾN TRÚC PHÂN TÁCH 2 TẦNG: ĐỘNG CƠ CỤC BỘ & MÔ HÌNH NHỎ CHUYÊN SÂU",
            subtitle_text="Phân tách tuyệt đối giữa logic tính toán số học xác định và tầng suy luận ngữ nghĩa tiếng Việt.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(5.75)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.233)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="TẦNG 1", badge_color=COLOR_BLUE_NAVY,
            title="TẦNG 1: ĐỘNG CƠ XỬ LÝ TOÁN HỌC XÁC ĐỊNH",
            bullets=[
                ("Tính toán chuẩn xác", "Thực hiện các phép cộng trừ và so khớp số dư với độ chính xác tuyệt đối từng đồng bạc."),
                ("Quy tắc kế toán thép", "Chỉ ghi nhận khớp nối khi số tiền cân bằng tuyệt đối và có bằng chứng chứng từ hợp lệ."),
                ("Cơ chế khóa an toàn", "Tự động dừng lại và chuyển sang con người kiểm tra ngay khi phát hiện sai lệch số học dù chỉ 1 đồng."),
                ("Nhật ký kiểm toán", "Ghi vết đầy đủ mọi thao tác đối soát phục vụ thanh tra và kiểm toán độc lập.")
            ],
            stat_val="0% Ảo Giác", stat_lbl="Toán học xác định bảo đảm chuẩn xác từng đồng"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="TẦNG 2", badge_color=COLOR_BLUE_PRIMARY,
            title="TẦNG 2: MÔ HÌNH NGÔN NGỮ NHỎ BẢN ĐỊA (3B-8B)",
            bullets=[
                ("Đọc hiểu viết tắt", "Tinh chỉnh chuyên biệt trên 500.000 mẫu chuyển khoản tiếng Việt thực tế của ngân hàng nội địa."),
                ("Vận hành siêu nhẹ", "Chạy mượt mà trên máy tính văn phòng chỉ với 2GB RAM, hoàn toàn không cần cạc đồ họa đắt đỏ."),
                ("Không rò rỉ dữ liệu", "Mô hình được đóng gói chạy hoàn toàn tại chỗ (Offline), không kết nối Internet ra bên ngoài."),
                ("Tối ưu hóa bản địa", "Hiểu sâu sắc các cấu trúc câu giao dịch đặc thù và tên viết tắt đối tác kinh doanh tại Việt Nam.")
            ],
            stat_val="< 0.5ms", stat_lbl="Độ trễ xử lý ngữ nghĩa trên máy tính thông thường"
        )

        self.add_footer(slide, 8, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(8))

    # =========================================================================
    # SLIDE 09: KẾT QUẢ ĐO KIỂM PHÒNG LAB (STATBOX & FEATURE CARDS)
    # =========================================================================
    def build_slide_09_lab_benchmark(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • THỰC NGHIỆM ĐO KIỂM • KẾT QUẢ PHÒNG LAB",
            title_text="KẾT QUẢ ĐO KIỂM PHÒNG LAB: 19 GIÂY / 50.000 DÒNG & 0% ẢO GIÁC SỐ HỌC",
            subtitle_text="Kiểm thử hiệu năng thực tế trên máy tính văn phòng thông thường chứng minh tính khả thi vượt trội.",
            badge_color=COLOR_EMERALD, badge_bg=COLOR_EMERALD_BG, badge_border=COLOR_EMERALD_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)

        # Left Column: Lab Performance Metrics
        left_w = Inches(4.2)
        stripes = [
            ("19 Giây", "TỐC ĐỘ 50.000 DÒNG", "Thời gian bóc tách và đối soát hoàn tất 50.000 dòng giao dịch sao kê ngân hàng.", COLOR_BLUE_NAVY, False),
            ("< 2.0 GB", "BỘ NHỚ TIÊU THỤ", "Dung lượng bộ nhớ RAM tiêu thụ khi vận hành toàn bộ mô hình và động cơ tại chỗ.", COLOR_BLUE_PRIMARY, True),
            ("0%", "ẢO GIÁC SỐ HỌC", "Tỷ lệ ảo giác số học trong suốt quá trình xử lý và đối chiếu dữ liệu kế toán.", COLOR_EMERALD, False),
        ]
        self.add_stacked_metric_stripes(slide, MARGIN_X, top_y, left_w, height, stripes)

        # Right Column: 2 Methodology & Benchmark Cards
        right_x = MARGIN_X + left_w + Inches(0.20)
        right_w = CONTENT_WIDTH - left_w - Inches(0.20)
        c_h = (height - Inches(0.15)) / 2

        self.add_square_badge_card(
            slide, right_x, top_y, right_w, c_h,
            badge_text="PHƯƠNG PHÁP", badge_color=COLOR_BLUE_NAVY,
            title="Phương Pháp Luận Đo Kiểm Thực Tế",
            bullets=[
                ("Tập dữ liệu thử nghiệm", "Sử dụng bộ dữ liệu tổng hợp mô phỏng 50.000 giao dịch thực tế từ 15 ngân hàng thương mại lớn."),
                ("Phần cứng văn phòng", "Thử nghiệm trên máy tính để bàn tiêu chuẩn (Chip Intel Core i5, RAM 8GB), không có cạc đồ họa rời."),
                ("Kiểm tra sức chịu tải", "Hệ thống vận hành liên tục ổn định, bộ nhớ giải phóng sạch sẽ ngay sau khi hoàn thành chu kỳ tính toán.")
            ]
        )

        self.add_square_badge_card(
            slide, right_x, top_y + c_h + Inches(0.15), right_w, c_h,
            badge_text="SO SÁNH", badge_color=COLOR_EMERALD,
            title="Hiệu Quả Vượt Trội So Với Thủ Công",
            bullets=[
                ("Nhanh gấp 450 lần", "Rút ngắn thời gian xử lý từ 2.5 giờ lao động căng thẳng xuống chỉ còn đúng 19 giây máy chạy."),
                ("Loại bỏ 100% lỗi con người", "Không còn nguy cơ nhập sai số liệu, nhầm lẫn mã đối tác hoặc bỏ sót các khoản phí ngân hàng nhỏ."),
                ("Tiết kiệm chi phí vận hành", "Giảm hơn 85% chi phí nhân sự và thời gian phát sinh cho công tác đối chiếu sổ sách định kỳ.")
            ]
        )

        self.add_footer(slide, 9, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(9))

    # =========================================================================
    # SLIDE 10: MÔ HÌNH DOANH THU & KINH DOANH (FEATURE CARDS)
    # =========================================================================
    def build_slide_10_business_model(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • MÔ HÌNH KINH DOANH • NGUỒN DOANH THU KÉP",
            title_text="MÔ HÌNH KINH DOANH: B2B SAAS TẠI CHỖ & CẤP PHÉP BẢN QUYỀN NGÂN HÀNG",
            subtitle_text="Kết hợp hài hòa giữa dòng tiền định kỳ ổn định từ doanh nghiệp và giá trị hợp đồng lớn từ ngân hàng.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(5.75)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.233)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="KÊNH 1", badge_color=COLOR_BLUE_NAVY,
            title="KÊNH 1: THUÊ BAO B2B SAAS DOANH NGHIỆP",
            bullets=[
                ("Gói Tiêu Chuẩn ($250/tháng)", "Dành cho doanh nghiệp quản lý dưới 5 tài khoản ngân hàng và 3.000 giao dịch mỗi tháng."),
                ("Gói Chuyên Nghiệp ($500/tháng)", "Hỗ trợ tối đa 15 tài khoản, tính năng tháp canh dòng tiền 48 giờ và tích hợp ERP tự động."),
                ("Gói Doanh Nghiệp Lớn ($850/tháng)", "Không giới hạn tài khoản, tích hợp sâu vào hệ thống kế toán nội bộ và hỗ trợ kỹ thuật 24/7."),
                ("Mô hình giá trị bền vững", "Thu phí theo năm, cam kết không phát sinh chi phí máy chủ đám mây biên cho khách hàng.")
            ],
            stat_val="$3k - $10k", stat_lbl="Doanh thu bình quân mỗi khách hàng / năm (ACV)"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="KÊNH 2", badge_color=COLOR_BLUE_PRIMARY,
            title="KÊNH 2: CẤP PHÉP BẢN QUYỀN NGÂN HÀNG (WHITE-LABEL)",
            bullets=[
                ("Tích hợp Internet Banking", "Ngân hàng mua bản quyền LIVA để cung cấp giải pháp giá trị gia tăng cho khách hàng doanh nghiệp tổ chức."),
                ("Gia tăng số dư CASA", "Giúp ngân hàng giữ chân dòng tiền và thu hút thêm hàng ngàn tài khoản tiền gửi không kỳ hạn."),
                ("Phí duy trì thường niên", "Thu phí bảo trì và nâng cấp thuật toán định kỳ 20% trên giá trị hợp đồng hàng năm."),
                ("Quan hệ cộng sinh", "Ngân hàng đóng vai trò kênh bảo chứng và phân phối trực tiếp giải pháp tới khách hàng.")
            ],
            stat_val="$50k - $150k", stat_lbl="Giá trị mỗi hợp đồng bản quyền ngân hàng / năm"
        )

        self.add_footer(slide, 10, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(10))

    # =========================================================================
    # SLIDE 11: CHIẾN LƯỢC TIẾP CẬN THỊ TRƯỜNG (STEPCHEVRON)
    # =========================================================================
    def build_slide_11_gtm(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • TIẾP CẬN THỊ TRƯỜNG • GTM FLYWHEEL",
            title_text="CHIẾN LƯỢC TIẾP CẬN THỊ TRƯỜNG: MẠNG LƯỚI ĐẠI LÝ ERP & CFO SUMMIT",
            subtitle_text="Tận dụng các kênh phân phối sẵn có để rút ngắn chu kỳ bán hàng và tối ưu hóa chi phí thu hút khách hàng.",
            badge_color=COLOR_BLUE_PRIMARY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        steps = [
            (
                "GIAI ĐOẠN 1",
                "Kênh Đại Lý Phần Mềm Kế Toán",
                [
                    ("Hợp tác đại lý", "Bắt tay với các đại lý triển khai MISA, FAST, Bravo để bán kèm LIVA như tiện ích mở rộng."),
                    ("Chia sẻ doanh thu", "Chiết khấu 25% - 30% hoa hồng năm đầu để kích hoạt động lực giới thiệu khách hàng của đại lý."),
                    ("Tiếp cận nhanh", "Tận dụng tệp hơn 100.000 doanh nghiệp đang dùng phần mềm kế toán sẵn có trên thị trường.")
                ],
                COLOR_BLUE_NAVY
            ),
            (
                "GIAI ĐOẠN 2",
                "Chuỗi Hội Thảo CFO Summit",
                [
                    ("Tổ chức hội thảo chuyên đề", "Tài trợ và tổ chức các sự kiện chuyển đổi số tài chính dành riêng cho CFO và Kế toán trưởng."),
                    ("Chương trình dùng thử 30 ngày", "Cung cấp bản dùng thử miễn phí trực tiếp trên dữ liệu sao kê của chính doanh nghiệp tham gia."),
                    ("Tỷ lệ chuyển đổi cao", "Kỳ vọng đạt tỷ lệ chuyển đổi trên 35% từ người dùng thử sang hợp đồng thuê bao trả phí.")
                ],
                COLOR_BLUE_PRIMARY
            ),
            (
                "GIAI ĐOẠN 3",
                "Bản Quyền Ngân Hàng Thương Mại",
                [
                    ("Thí điểm cùng 2 ngân hàng", "Triển khai giải pháp Sandbox đồng hành cùng các ngân hàng thương mại tiên phong."),
                    ("Bán chéo dịch vụ", "Ngân hàng giới thiệu LIVA cho tệp khách hàng doanh nghiệp lớn để tăng cường liên kết CASA."),
                    ("Hiệu ứng mạng lưới", "Tạo rào cản phòng thủ tự nhiên khi giải pháp được bảo chứng bởi các định chế tài chính uy tín.")
                ],
                COLOR_EMERALD
            )
        ]

        self.add_step_chevron(slide, MARGIN_X, Inches(1.65), CONTENT_WIDTH, Inches(4.90), steps)

        self.add_footer(slide, 11, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(11))

    # =========================================================================
    # SLIDE 12: LỢI THẾ CẠNH TRANH & HÀO LŨY PHÒNG THỦ (FEATURE CARDS)
    # =========================================================================
    def build_slide_12_competitive_moat(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • LỢI THẾ CẠNH TRANH • HÀO LŨY PHÒNG THỦ",
            title_text="HÀO LŨY PHÒNG THỦ: TÍNH TRUNG LẬP ĐA NGÂN HÀNG & BỘ NGỮ LIỆU ĐỘC QUYỀN",
            subtitle_text="4 Trụ cột lợi thế bảo đảm vị thế dẫn đầu tuyệt đối trước các đối thủ trong nước và quốc tế.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(2.783)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="TRUNG LẬP", badge_color=COLOR_BLUE_NAVY,
            title="Tính Trung Lập Đa Ngân Hàng",
            bullets=[
                ("Không xung đột", "Ngân hàng không bao giờ xây ứng dụng quản lý tài khoản đối thủ."),
                ("Trọng tài độc lập", "Doanh nghiệp bắt buộc cần bên thứ ba độc lập như LIVA."),
                ("Bảo mật dữ liệu", "Tuyệt đối không để lộ số dư giữa các tổ chức tín dụng.")
            ],
            stat_val="Trung Lập", stat_lbl="Vị thế trọng tài độc lập"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="NGỮ LIỆU", badge_color=COLOR_BLUE_PRIMARY,
            title="Bộ Ngữ Liệu Bản Địa Độc Quyền",
            bullets=[
                ("500k Mẫu thực tế", "Kho dữ liệu bóc tách viết tắt tiếng Việt tài chính độc quyền lớn nhất."),
                ("Hào lũy bản địa", "Mô hình toàn cầu không thể tiếp cận dữ liệu sao kê nội địa."),
                ("Độ chính xác cao", "Hiểu sâu sắc các quy ước viết tắt và từ lóng kế toán.")
            ],
            stat_val="500k Mẫu", stat_lbl="Dữ liệu ngữ liệu bản địa"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="TẠI CHỖ", badge_color=COLOR_EMERALD,
            title="Kiến Trúc Vận Hành Tại Chỗ",
            bullets=[
                ("100% Không Cloud", "Dữ liệu không rời máy trạm; tuân thủ hoàn hảo Nghị định 13."),
                ("Chi phí biên $0", "Không chịu chi phí máy chủ đắt đỏ, biên gộp vững chắc >88%."),
                ("Vận hành mượt mà", "Chạy trên máy tính văn phòng, không phụ thuộc Internet.")
            ],
            stat_val="$0 Cloud", stat_lbl="Chi phí máy chủ biên"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 3, top_y, card_w, card_h,
            badge_text="XÁC ĐỊNH", badge_color=COLOR_AMBER,
            title="Động Cơ Toán Học Xác Định",
            bullets=[
                ("Không ảo giác số", "Động cơ toán học xác định bảo đảm kết quả cân đối 100%."),
                ("Khóa an toàn 0.2%", "Dừng lại và chuyển người duyệt khi có bất kỳ nghi vấn."),
                ("Maker-Checker", "Giữ nguyên quyền quyết định tài chính tối cao cho con người.")
            ],
            stat_val="0% Ảo Giác", stat_lbl="Toán học xác định"
        )

        self.add_footer(slide, 12, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(12))

    # =========================================================================
    # SLIDE 13: KÊU GỌI VỐN SEED & KẾT THÚC VÒNG 1 (DONUTCHART & CARDS)
    # =========================================================================
    def build_slide_13_seed_ask(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 1 • KÊU GỌI ĐẦU TƯ • SEED ROUND ASK",
            title_text="KÊU GỌI VỐN SEED: $500K-$750K CHO 18-24 THÁNG RUNWAY TỰ CHỦ",
            subtitle_text="Sử dụng nguồn vốn có kỷ luật để hoàn thiện sản phẩm và mở rộng kênh phân phối đại lý kế toán.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)

        # Left Column: Seed Terms Summary Cards
        left_w = Inches(5.2)
        c_h = (height - Inches(0.20)) / 3

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, left_w, c_h,
            badge_text="SEED", badge_color=COLOR_BLUE_NAVY,
            title="Quy Mô Vòng Gọi Vốn Seed",
            bullets=[
                ("Mục tiêu huy động", "Từ $500.000 đến $750.000 USD cho 12.0% – 15.0% cổ phần."),
                ("Định giá doanh nghiệp", "$4.0M – $5.0M USD (Post-money), bảo đảm 18-24 tháng runway.")
            ],
            stat_val="$500k - $750k", stat_lbl="Quy mô vốn gọi Seed"
        )

        self.add_square_badge_card(
            slide, MARGIN_X, top_y + c_h + Inches(0.10), left_w, c_h,
            badge_text="MỤC TIÊU", badge_color=COLOR_BLUE_PRIMARY,
            title="Mục Tiêu Thương Mại 12 Tháng",
            bullets=[
                ("Quy mô khách hàng", "Đạt mốc 220 khách hàng doanh nghiệp trả phí định kỳ."),
                ("Đối tác ngân hàng", "Ký kết thỏa thuận thử nghiệm chính thức cùng 02 ngân hàng lớn.")
            ],
            stat_val="220 DN", stat_lbl="Khách hàng doanh nghiệp mục tiêu"
        )

        self.add_square_badge_card(
            slide, MARGIN_X, top_y + (c_h + Inches(0.10)) * 2, left_w, c_h,
            badge_text="RUNWAY", badge_color=COLOR_EMERALD,
            title="Bảo Đảm Đường Băng Tài Chính",
            bullets=[
                ("Tự chủ tài chính", "Runway 18 đến 24 tháng an toàn tuyệt đối với mức đốt vốn tinh gọn."),
                ("Điểm hòa vốn", "Dự kiến chính thức chạm điểm hòa vốn vận hành vào Tháng 11/2027.")
            ],
            stat_val="18 - 24 Th", stat_lbl="Runway tài chính an toàn"
        )

        # Right Column: Native Donut Chart for Capital Allocation
        right_x = MARGIN_X + left_w + Inches(0.20)
        right_w = CONTENT_WIDTH - left_w - Inches(0.20)

        categories = [
            "R&D & Kết Nối 35+ NH (50%)",
            "Phát Triển Thị Trường GTM (25%)",
            "Kiểm Toán ISO & Sandbox (15%)",
            "Quỹ Dự Phòng Vận Hành (10%)"
        ]
        values = [50, 25, 15, 10]
        colors = [COLOR_BLUE_NAVY, COLOR_BLUE_PRIMARY, COLOR_EMERALD, COLOR_AMBER]

        self.add_donut_chart(
            slide, right_x, top_y, right_w, height,
            categories, values, colors,
            title="PHÂN BỔ NGUỒN VỐN SEED ($500K - $750K USD)"
        )

        self.add_footer(slide, 13, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(13))

    # =========================================================================
    # SLIDE 14: LỘ TRÌNH THỰC THI 4 QUÝ (STEPCHEVRON)
    # =========================================================================
    def build_slide_14_roadmap(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM (TOP 3) • LỘ TRÌNH SẢN PHẨM",
            title_text="LỘ TRÌNH THỰC THI 4 QUÝ: TỪ PHÒNG LAB ĐẾN LIÊN MINH KẾ TOÁN & NGÂN HÀNG",
            subtitle_text="Kế hoạch hành động kỷ luật theo từng mốc quý (Q4/2026 – Q3/2027), bám sát các chỉ số nghiệm thu thực tế.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        steps = [
            (
                "QUÝ 1",
                "Q4/2026: Thử Nghiệm & Hoàn Thiện",
                [
                    ("Hoàn thiện sản phẩm", "Mở rộng bộ giải mã cho 20 ngân hàng lớn; thử nghiệm nội bộ cùng 15 doanh nghiệp thân thiết."),
                    ("Hồ sơ pháp lý", "Nộp hồ sơ đánh giá an ninh thông tin và chuẩn bị tham gia cơ chế thử nghiệm Sandbox."),
                    ("Doanh thu mục tiêu", "ARR $150.000 USD từ các khách hàng tiên phong đầu tiên.")
                ],
                COLOR_BLUE_NAVY
            ),
            (
                "QUÝ 2",
                "Q1/2027: Ra Mắt Top 5 Ngân Hàng",
                [
                    ("Kết nối chuẩn hóa", "Hỗ trợ hoàn chỉnh Vietcombank, Techcombank, BIDV, VietinBank và MBBank."),
                    ("Mở rộng khách hàng", "Đạt 50 khách hàng doanh nghiệp trả phí; triển khai cơ chế phê duyệt hai pha hoàn chỉnh."),
                    ("Doanh thu mục tiêu", "ARR $450.000 USD với dòng tiền vận hành ổn định.")
                ],
                COLOR_BLUE_PRIMARY
            ),
            (
                "QUÝ 3",
                "Q2/2027: Tích Hợp Phần Mềm Kế Toán",
                [
                    ("Kết nối ERP 1 chạm", "Tích hợp tự động vào các phần mềm kế toán phổ biến MISA, FAST và Bravo."),
                    ("Thí điểm ngân hàng", "Thí điểm cùng 2 ngân hàng thương mại tiên phong; vượt mốc 120 doanh nghiệp sử dụng."),
                    ("Doanh thu mục tiêu", "ARR $1.100.000 USD khẳng định tính khả thi của mô hình.")
                ],
                COLOR_EMERALD
            ),
            (
                "QUÝ 4",
                "Q3/2027: Phủ Sóng 35+ Ngân Hàng",
                [
                    ("Phủ sóng toàn diện", "Hỗ trợ toàn bộ 35+ ngân hàng tại Việt Nam; thương mại hóa tháp canh dự báo dòng tiền 48 giờ."),
                    ("Cấp phép bản quyền", "Cấp bản quyền cho 2 ngân hàng thương mại; đạt 220 doanh nghiệp trả phí."),
                    ("Doanh thu mục tiêu", "ARR $2.400.000 USD và chính thức chuẩn bị chạm điểm hòa vốn.")
                ],
                COLOR_AMBER
            )
        ]

        self.add_step_chevron(slide, MARGIN_X, Inches(1.65), CONTENT_WIDTH, Inches(4.90), steps)

        self.add_footer(slide, 14, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(14))

    # =========================================================================
    # SLIDE 15: HIỆU QUẢ ĐƠN VỊ VƯỢT TRỘI (STATBOX & CARDS)
    # =========================================================================
    def build_slide_15_unit_economics(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM • UNIT ECONOMICS & TÀI CHÍNH",
            title_text="HIỆU QUẢ ĐƠN VỊ VƯỢT TRỘI: LTV/CAC 5.2X & ĐIỂM HÒA VỐN THÁNG 11/2027",
            subtitle_text="Mô hình tài chính bền vững nhờ chi phí máy chủ đám mây biên bằng 0 và thời gian hoàn vốn khách hàng dưới 3 tháng.",
            badge_color=COLOR_EMERALD, badge_bg=COLOR_EMERALD_BG, badge_border=COLOR_EMERALD_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)
        card_w = Inches(2.783)
        gap = Inches(0.20)

        econ_boxes = [
            ("5.2x", COLOR_BLUE_NAVY, "TỶ LỆ LTV / CAC", "Giá trị vòng đời khách hàng trên chi phí thu hút", [
                ("Thời gian gắn bó", "Tính trên thời gian sử dụng trung bình 36 tháng của khách hàng doanh nghiệp."),
                ("Tỷ lệ giữ chân cao", "Dự kiến đạt tỷ lệ giữ chân và gia hạn hợp đồng trên 92% hàng năm."),
                ("Hiệu quả vốn", "Mỗi đồng chi phí thu hút mang lại hơn 5.2 đồng doanh thu trọn đời.")
            ]),
            ("4.6 Th", COLOR_BLUE_PRIMARY, "HOÀN VỐN CAC", "Thời gian thu hồi chi phí bán hàng và tiếp thị", [
                ("Kênh đại lý sẵn có", "Tận dụng mạng lưới đại lý phần mềm kế toán giúp rút ngắn thời gian bán hàng."),
                ("Chu kỳ chốt nhanh", "Khách hàng ra quyết định thuê bao trong vòng 2 đến 4 tuần làm việc."),
                ("Dòng tiền quay vòng", "Thu hồi toàn bộ chi phí bán hàng chỉ sau 4.6 tháng thu phí dịch vụ.")
            ]),
            ("> 88%", COLOR_EMERALD, "BIÊN LỢI NHUẬN GỘP", "Biên lợi nhuận gộp phần mềm vững chắc", [
                ("Chi phí máy chủ $0", "Phần mềm chạy trực tiếp trên máy trạm, không tốn chi phí điện toán đám mây."),
                ("Mở rộng không tốn kém", "Thêm khách hàng mới không làm tăng chi phí hạ tầng máy chủ biên."),
                ("Tỷ suất lợi nhuận cao", "Duy trì biên lợi nhuận gộp bền vững trên 88% qua các năm phát triển.")
            ]),
            ("T11/27", COLOR_AMBER, "ĐIỂM HÒA VỐN", "Thời điểm hòa vốn dòng tiền vận hành tự chủ", [
                ("Quy mô hòa vốn", "Đạt điểm hòa vốn vận hành khi chạm mốc 220 khách hàng doanh nghiệp trả phí."),
                ("Tự chủ tài chính", "Dòng tiền kinh doanh đủ bù đắp toàn bộ chi phí hoạt động mà không cần gọi thêm vốn."),
                ("Bảo toàn giá trị", "Bảo vệ tối đa quyền lợi và tránh pha loãng cổ phần cho các nhà đầu tư vòng Seed.")
            ]),
        ]

        for i, (val, col, title, sub, bullets) in enumerate(econ_boxes):
            cx = MARGIN_X + i * (card_w + gap)
            self.add_square_badge_card(
                slide, cx, top_y, card_w, height,
                badge_text=title.split()[0], badge_color=col,
                title=title,
                bullets=bullets,
                stat_val=val, stat_lbl=sub
            )

        self.add_footer(slide, 15, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(15))

    # =========================================================================
    # SLIDE 16: DỰ PHÓNG DOANH THU 3 NĂM (FEATURE CARDS)
    # =========================================================================
    def build_slide_16_financial_projections(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM • DỰ PHÓNG DOANH THU",
            title_text="DỰ PHÓNG DOANH THU 3 NĂM: BASE CASE $3.9M VÀ BULL TARGET $7.2M NĂM 2028",
            subtitle_text="Cơ cấu doanh thu minh bạch giữa thuê bao doanh nghiệp và bản quyền vận hành ngân hàng thương mại.",
            badge_color=COLOR_BLUE_PRIMARY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(5.75)
        top_y = Inches(1.65)
        card_h = Inches(3.70)
        gap = Inches(0.233)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="BASE", badge_color=COLOR_BLUE_NAVY,
            title="Kịch Bản Cơ Sở (Base Case 2028)",
            bullets=[
                ("Doanh thu dự phóng", "$3,900,000 USD (Doanh thu định kỳ ARR: $2,750,000 USD)."),
                ("Quy mô khách hàng", "450 Doanh nghiệp trả phí và 8 Ngân hàng thương mại cấp phép bản quyền."),
                ("Tỷ lệ gia hạn hợp đồng", "Dự kiến đạt > 92% nhờ tính gắn kết nghiệp vụ cao của phần mềm."),
                ("Cơ cấu nguồn thu", "70% từ thuê bao doanh nghiệp và 30% từ bản quyền ngân hàng.")
            ],
            stat_val="$3,900,000", stat_lbl="Doanh thu dự phóng năm 2028"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="BULL", badge_color=COLOR_EMERALD,
            title="Kịch Bản Mục Tiêu Bứt Phá (Bull Target 2028)",
            bullets=[
                ("Doanh thu bứt phá", "$7,200,000 USD (Doanh thu định kỳ ARR: $5,800,000 USD)."),
                ("Quy mô mở rộng", "750 Doanh nghiệp trả phí và 8 Ngân hàng thương mại mua bản quyền nâng cao."),
                ("Biên lợi nhuận ròng", "Dự kiến vượt > 42% nhờ mô hình phần mềm tại chỗ không chịu chi phí server."),
                ("Thị phần mục tiêu", "Chiếm khoảng 4.5% thị phần doanh nghiệp mục tiêu tại Việt Nam.")
            ],
            stat_val="$7,200,000", stat_lbl="Kịch bản mục tiêu bứt phá năm 2028"
        )

        # Summary Growth Bar
        summary_y = top_y + card_h + Inches(0.18)
        summary_h = Inches(1.02)
        self.add_card(slide, MARGIN_X, summary_y, CONTENT_WIDTH, summary_h, bg_color=COLOR_STRIPE_BLUE, border_color=COLOR_STRIPE_BORDER)

        tx_sm = slide.shapes.add_textbox(MARGIN_X + Inches(0.20), summary_y + Inches(0.10), CONTENT_WIDTH - Inches(0.40), summary_h - Inches(0.20))
        tf_sm = tx_sm.text_frame
        tf_sm.word_wrap = True
        tf_sm.margin_left = tf_sm.margin_top = tf_sm.margin_right = tf_sm.margin_bottom = 0

        p_st = tf_sm.paragraphs[0]
        p_st.text = "LỘ TRÌNH TĂNG TRƯỞNG DOANH THU 3 NĂM"
        p_st.font.name = FONT_FAMILY
        p_st.font.size = Pt(8.8)
        p_st.font.bold = True
        p_st.font.color.rgb = COLOR_BLUE_PRIMARY
        p_st.space_after = Pt(2)

        p_sc = tf_sm.add_paragraph()
        p_sc.text = "Năm 2026: $450k (35 DN, 1 NH) ➔ Năm 2027: $2.4M (220 DN, 3 NH, Hòa vốn T11) ➔ Năm 2028: $3.9M (Base) – $7.2M (Bull)"
        p_sc.font.name = FONT_FAMILY
        p_sc.font.size = Pt(11.0)
        p_sc.font.bold = True
        p_sc.font.color.rgb = COLOR_DARK_SLATE

        self.add_footer(slide, 16, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(16))

    # =========================================================================
    # SLIDE 17: QUẢN TRỊ DÒNG TIỀN VẬN HÀNH & CHI PHÍ (DONUTCHART)
    # =========================================================================
    def build_slide_17_cost_structure(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM • CƠ CẤU CHI PHÍ & BURN RATE",
            title_text="QUẢN TRỊ DÒNG TIỀN VẬN HÀNH: CHI PHÍ TINH GỌN & TỶ LỆ ĐỐT TIỀN DƯỚI $25K/THÁNG",
            subtitle_text="Tối ưu hóa tối đa chi phí hoạt động nhờ mô hình phần mềm chạy tại chỗ, bảo đảm runway dài hạn.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)

        # Left Column: Burn Rate & Control Principles
        left_w = Inches(5.2)
        c_h = (height - Inches(0.20)) / 3

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, left_w, c_h,
            badge_text="BURN", badge_color=COLOR_DANGER,
            title="Kiểm Soát Tỷ Lệ Đốt Tiền",
            bullets=[
                ("Mức chi tiêu hàng tháng", "Kiểm soát chặt chẽ ở mức $15.000 – $25.000 USD/tháng trong giai đoạn đầu."),
                ("Nguyên tắc kỷ luật", "Chỉ gia tăng ngân sách khi các chỉ số doanh thu thực tế đạt mốc nghiệm thu.")
            ],
            stat_val="<$25k/Th", stat_lbl="Tỷ lệ đốt tiền hàng tháng"
        )

        self.add_square_badge_card(
            slide, MARGIN_X, top_y + c_h + Inches(0.10), left_w, c_h,
            badge_text="RUNWAY", badge_color=COLOR_EMERALD,
            title="Đường Băng Tài Chính An Toàn",
            bullets=[
                ("Thời gian bảo đảm", "Nguồn vốn Seed $500k – $750k bảo đảm 18 đến 24 tháng vận hành liên tục."),
                ("Vùng đệm an toàn", "Duy trì hoạt động ổn định kể cả khi thị trường có biến động bất lợi.")
            ],
            stat_val="18 - 24 Th", stat_lbl="Runway an toàn tuyệt đối"
        )

        self.add_square_badge_card(
            slide, MARGIN_X, top_y + (c_h + Inches(0.10)) * 2, left_w, c_h,
            badge_text="TỰ CHỦ", badge_color=COLOR_BLUE_NAVY,
            title="Nguyên Tắc Dòng Tiền Thép",
            bullets=[
                ("Lấy ngắn nuôi dài", "Dòng tiền từ thuê bao doanh nghiệp bù đắp chi phí trước khi mở rộng quy mô lớn."),
                ("Minh bạch tài chính", "Báo cáo thu chi chi tiết theo quý gửi hội đồng quản trị và nhà đầu tư.")
            ],
            stat_val="Tự Chủ", stat_lbl="Mô hình tài chính bền vững"
        )

        # Right Column: Donut Chart for Monthly Operating Expenses
        right_x = MARGIN_X + left_w + Inches(0.20)
        right_w = CONTENT_WIDTH - left_w - Inches(0.20)

        categories = [
            "Lương Kỹ Sư & Tài Chính (55%)",
            "Tiếp Thị GTM & Hội Thảo (20%)",
            "Pháp Lý & ISO/Sandbox (15%)",
            "Quản Lý & Hạ Tầng Lab (10%)"
        ]
        values = [55, 20, 15, 10]
        colors = [COLOR_BLUE_NAVY, COLOR_BLUE_PRIMARY, COLOR_EMERALD, COLOR_AMBER]

        self.add_donut_chart(
            slide, right_x, top_y, right_w, height,
            categories, values, colors,
            title="CƠ CẤU CHI PHÍ VẬN HÀNH HÀNG THÁNG"
        )

        self.add_footer(slide, 17, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(17))

    # =========================================================================
    # SLIDE 18: KHUNG PHÁP LÝ & AN NINH DỮ LIỆU (FEATURE CARDS)
    # =========================================================================
    def build_slide_18_regulatory_compliance(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM • PHÁP LÝ & AN TOÀN THÔNG TIN",
            title_text="KHUNG PHÁP LÝ & AN NINH: ĐÁNH GIÁ ISO 27001 & HỒ SƠ SANDBOX NGÂN HÀNG NHÀ NƯỚC",
            subtitle_text="Chủ động hoàn thiện các chứng nhận an toàn thông tin và tham gia cơ chế thử nghiệm có kiểm soát.",
            badge_color=COLOR_EMERALD, badge_bg=COLOR_EMERALD_BG, badge_border=COLOR_EMERALD_BORDER
        )
        card_w = Inches(2.783)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="NĐ13", badge_color=COLOR_DANGER,
            title="Nghị Định 13 & DPIA",
            bullets=[
                ("Đánh giá tác động", "Hoàn thành hồ sơ Đánh giá tác động xử lý dữ liệu cá nhân gửi Cục An ninh mạng A05."),
                ("Zero Data Egress", "Cam kết 100% dữ liệu tài chính không bao giờ rời khỏi thiết bị nội bộ của khách hàng."),
                ("Kiểm soát chặt chẽ", "Bảo đảm quyền riêng tư của chủ thể dữ liệu tuyệt đối theo quy định.")
            ],
            stat_val="DPIA A05", stat_lbl="Tuân thủ Nghị định 13"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="TT09", badge_color=COLOR_BLUE_NAVY,
            title="Thông Tư 09/2020",
            bullets=[
                ("Cấp độ 3 - 5", "Tuân thủ tiêu chuẩn an toàn hệ thống thông tin ngành ngân hàng cấp độ cao."),
                ("Nhật ký kiểm toán", "Lưu trữ nhật ký kiểm toán bất biến phục vụ thanh tra và đối chiếu nghiệp vụ."),
                ("Bảo vệ hệ thống lõi", "Cấm dùng đám mây công cộng cho các tác vụ ngân quỹ trọng yếu.")
            ],
            stat_val="Cấp Độ 3 - 5", stat_lbl="An toàn hệ thống TT"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="ISO", badge_color=COLOR_EMERALD,
            title="Tiêu Chuẩn ISO 27001",
            bullets=[
                ("Kiểm toán độc lập", "Hợp tác cùng tổ chức kiểm toán an ninh độc lập đánh giá quy trình phần mềm."),
                ("Chứng chỉ uy tín", "Xác lập chuẩn mực quản lý an toàn thông tin tạo dựng niềm tin với khối tài chính."),
                ("Khung an toàn quốc tế", "Áp dụng các biện pháp kiểm soát an ninh thông tin theo tiêu chuẩn toàn cầu.")
            ],
            stat_val="ISO 27001", stat_lbl="Quản lý ATTT độc lập"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 3, top_y, card_w, card_h,
            badge_text="SBX", badge_color=COLOR_AMBER,
            title="Sandbox Ngân Hàng Nhà Nước",
            bullets=[
                ("Quyết định 810", "Chuẩn bị sẵn sàng hồ sơ tham gia cơ chế thử nghiệm có kiểm soát giải pháp Fintech."),
                ("Tiên phong chuẩn mực", "Tham gia xây dựng chuẩn mực kết nối tự động hóa ngân quỹ bản địa tại Việt Nam."),
                ("Đồng hành ngân hàng", "Hợp tác chặt chẽ cùng các ngân hàng thương mại lớn trong khuôn khổ thử nghiệm.")
            ],
            stat_val="QĐ 810", stat_lbl="Cơ chế thử nghiệm Sandbox"
        )

        self.add_footer(slide, 18, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(18))

    # =========================================================================
    # SLIDE 19: ĐỘI NGŨ SÁNG LẬP 100% FULL-TIME (FEATURE CARDS)
    # =========================================================================
    def build_slide_19_founding_team(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM • ĐỘI NGŨ SÁNG LẬP",
            title_text="ĐỘI NGŨ SÁNG LẬP 100% FULL-TIME: GẮN KẾT, THẤU HIỂU NGHIỆP VỤ & LÀM CHỦ CÔNG NGHỆ",
            subtitle_text="5 nhà sáng lập làm việc toàn thời gian cùng kế hoạch bổ sung kỹ sư hệ thống hiệu năng cao.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        top_y = Inches(1.65)
        card_h = Inches(3.70)
        card_w = Inches(2.186)
        gap = Inches(0.20)

        founders = [
            ("Nguyễn Anh Dương", "CEO / Co-Founder", "Chiến lược Sản phẩm & Điều hành", "Định hướng chiến lược, kết nối đối tác ngân hàng và quan hệ nhà đầu tư.", "CEO", COLOR_BLUE_NAVY),
            ("Nguyễn Hoàng Hiếu", "CTO / Co-Founder", "Kiến trúc sư Hệ thống & AI", "Làm chủ kiến trúc động cơ xử lý tại chỗ và mô hình ngôn ngữ chuyên sâu.", "CTO", COLOR_BLUE_PRIMARY),
            ("Nguyễn Huy Anh Minh", "COO / Co-Founder", "Vận hành & Pháp lý", "Quản trị tuân thủ Nghị định 13, mở rộng thị trường và quản lý mạng lưới đối tác.", "COO", COLOR_EMERALD),
            ("Nguyễn Minh Hiếu", "Head of Product", "Trải nghiệm Khách hàng & Nghiệp vụ", "Thiết kế quy trình ngân quỹ và tối ưu hóa trải nghiệm kế toán doanh nghiệp.", "CPO", COLOR_AMBER),
            ("Cao Xuân Đại", "Lead Engineer", "Kỹ sư Hệ thống & Tích hợp", "Tối ưu hóa thuật toán đối soát số học xác định và giao diện Workbench.", "TECH", COLOR_DARK_SLATE)
        ]

        for i, (name, role, specialty, desc, badge_lbl, col) in enumerate(founders):
            cx = MARGIN_X + i * (card_w + gap)
            self.add_square_badge_card(
                slide, cx, top_y, card_w, card_h,
                badge_text=badge_lbl, badge_color=col,
                title=name,
                bullets=[
                    ("Vai trò", role),
                    ("Chuyên môn", specialty),
                    ("Nhiệm vụ", desc)
                ]
            )

        # Bottom Recruitment Plan Box
        rec_y = top_y + card_h + Inches(0.18)
        rec_h = Inches(1.02)
        self.add_card(slide, MARGIN_X, rec_y, CONTENT_WIDTH, rec_h, bg_color=COLOR_STRIPE_BLUE, border_color=COLOR_STRIPE_BORDER)

        tx_rec = slide.shapes.add_textbox(MARGIN_X + Inches(0.20), rec_y + Inches(0.10), CONTENT_WIDTH - Inches(0.40), rec_h - Inches(0.20))
        tf_rec = tx_rec.text_frame
        tf_rec.word_wrap = True
        tf_rec.margin_left = tf_rec.margin_top = tf_rec.margin_right = tf_rec.margin_bottom = 0

        p_rt = tf_rec.paragraphs[0]
        p_rt.text = "CAM KẾT DÀI HẠN & KẾ HOẠCH TUYỂN DỤNG SAU VÒNG SEED"
        p_rt.font.name = FONT_FAMILY
        p_rt.font.size = Pt(8.8)
        p_rt.font.bold = True
        p_rt.font.color.rgb = COLOR_BLUE_PRIMARY
        p_rt.space_after = Pt(2)

        p_rc = tf_rec.add_paragraph()
        p_rc.text = "100% Thành viên sáng lập cam kết toàn thời gian tối thiểu 3 năm. Kế hoạch tuyển dụng sau vòng Seed: Bổ sung 4 kỹ sư hệ thống cao cấp và 2 chuyên viên tư vấn tài chính doanh nghiệp để đẩy nhanh tiến độ sản phẩm."
        p_rc.font.name = FONT_FAMILY
        p_rc.font.size = Pt(9.5)
        p_rc.font.color.rgb = COLOR_DARK_SLATE

        self.add_footer(slide, 19, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(19))

    # =========================================================================
    # SLIDE 20: QUẢN TRỊ RỦI RO CHỦ ĐỘNG (FEATURE CARDS)
    # =========================================================================
    def build_slide_20_risk_management(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 2 • KẾ HOẠCH 1 NĂM • QUẢN TRỊ RỦI RO",
            title_text="QUẢN TRỊ RỦI RO CHỦ ĐỘNG: KỊCH BẢN ỨNG PHÓ PHÁP LÝ, KINH DOANH VÀ CÔNG NGHỆ",
            subtitle_text="Nhìn thẳng vào khó khăn và chuẩn bị sẵn sàng các phương án phòng vệ an toàn cho doanh nghiệp.",
            badge_color=COLOR_DANGER, badge_bg=COLOR_RED_BG, badge_border=COLOR_RED_BORDER
        )

        card_w = Inches(3.777)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="BIZ", badge_color=COLOR_BLUE_NAVY,
            title="Chu Kỳ Bán Hàng Ngân Hàng Kéo Dài",
            bullets=[
                ("Thách thức nhận diện", "Quá trình thẩm định kỹ thuật và ký kết với ngân hàng thương mại thường kéo dài 12-18 tháng."),
                ("Kịch bản ứng phó", "Lấy ngắn nuôi dài — Tập trung bán gói thuê bao cho các CFO doanh nghiệp chốt hợp đồng trong 2–4 tuần."),
                ("Kết quả bảo đảm", "Tạo dòng tiền tự chủ nuôi sống bộ máy mà không bị phụ thuộc vào tiến độ ngân hàng.")
            ],
            stat_val="12 - 18 Th", stat_lbl="Chu kỳ bán hàng ngân hàng"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="LAW", badge_color=COLOR_AMBER,
            title="Thủ Tục Thử Nghiệm Sandbox",
            bullets=[
                ("Thách thức nhận diện", "Thời gian phê duyệt đề án Sandbox của Ngân hàng Nhà nước có thể kéo dài từ 6 đến 12 tháng."),
                ("Kịch bản ứng phó", "Triển khai trước gói quản trị và đối soát nội bộ cho doanh nghiệp (không can thiệp lệnh chuyển tiền)."),
                ("Kết quả bảo đảm", "Tạo doanh thu thương mại ngay lập tức trong khi song song hoàn thiện hồ sơ Sandbox.")
            ],
            stat_val="6 - 12 Th", stat_lbl="Thời gian chuẩn bị Sandbox"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="TECH", badge_color=COLOR_DANGER,
            title="Mẫu Sao Kê Thay Đổi Đột Ngột",
            bullets=[
                ("Thách thức nhận diện", "Ngân hàng cập nhật giao diện Internet Banking hoặc thay đổi cấu trúc file Excel sao kê."),
                ("Kịch bản ứng phó", "Kích hoạt cơ chế khóa an toàn chuyển 0.2% sang người duyệt ngay khi phát hiện mẫu chưa nhận diện."),
                ("Kết quả bảo đảm", "Cập nhật từ điển bóc tách tự động trong vòng 24 giờ, bảo đảm không bao giờ lệch số.")
            ],
            stat_val="Khóa 0.2%", stat_lbl="Cơ chế an toàn tức thì"
        )

        self.add_footer(slide, 20, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(20))

    # =========================================================================
    # SLIDE 21: Q&A 01 — TẠI SAO KHÔNG DÙNG EXCEL HAY MACRO VBA?
    # =========================================================================
    def build_slide_21_qa_excel_vs_liva(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • Q&A PHẢN BIỆN CHUYÊN SÂU • DEEP DIVE 01: EXCEL VS LIVA",
            title_text="Q&A 01: \"TẠI SAO KHÔNG DÙNG EXCEL HAY MACRO VBA ĐỂ ĐỐI SOÁT?\"",
            subtitle_text="Phân tích giới hạn chết người của bảng tính thủ công trước khối lượng sao kê ngân hàng đa dạng.",
            badge_color=COLOR_DANGER, badge_bg=COLOR_RED_BG, badge_border=COLOR_RED_BORDER
        )

        left_data = {
            'title': '3 Giới Hạn Chết Người Của Excel & Macro VBA',
            'subtitle': 'Bảng tính thủ công sụp đổ trước sao kê thực tế phức tạp',
            'bg_color': COLOR_RED_BG,
            'border_color': COLOR_RED_BORDER,
            'header_color': COLOR_DANGER,
            'bullet_icon': '❌',
            'items': [
                (
                    'Gãy công thức',
                    'Chỉ cần một ngân hàng chèn thêm 1 cột hoặc đổi tiêu đề là toàn bộ hàm VLOOKUP và Macro báo lỗi, làm tê liệt toàn bộ chuỗi đối soát.'
                ),
                (
                    'Mù chữ viết tắt',
                    'Excel hoàn toàn không thể hiểu các diễn giải tự do như "CK HD 101 bot tien thue" hay tên đối tác viết tắt không dấu.'
                ),
                (
                    'Quá tải dữ liệu',
                    'Khi số lượng giao dịch vượt 20.000 dòng, file Excel bị đơ, treo máy tính và tiềm ẩn rủi ro mất mát dữ liệu kế toán nghiêm trọng.'
                )
            ]
        }

        right_data = {
            'title': 'Ưu Thế Tuyệt Đối Của LIVA Banking Harness',
            'subtitle': 'Tự động hóa thông minh vận hành an toàn và chuẩn xác tại chỗ',
            'bg_color': COLOR_EMERALD_BG,
            'border_color': COLOR_EMERALD_BORDER,
            'header_color': COLOR_EMERALD,
            'bullet_icon': '✅',
            'items': [
                (
                    'Đọc hiểu thông minh',
                    'Tự động nhận diện nội dung ngữ nghĩa tiếng Việt bất kể vị trí cột, ô gộp hay định dạng bảng biểu đặc thù của từng ngân hàng.'
                ),
                (
                    'Tốc độ tức thì',
                    'Xử lý 50.000 dòng trong 19 giây trên máy tính văn phòng thông thường, vận hành mượt mà không bao giờ gây treo máy.'
                ),
                (
                    'Nhật ký kiểm toán',
                    'Lưu trữ lịch sử đối soát bất biến, minh bạch quy trình và không bao giờ bị vô tình sửa xóa công thức như bảng tính Excel.'
                )
            ]
        }

        self.add_compare_grid(
            slide, MARGIN_X, Inches(1.65), CONTENT_WIDTH, Inches(4.90),
            left_data, right_data
        )

        self.add_footer(slide, 21, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(21))

    # =========================================================================
    # SLIDE 22: Q&A 02 — NẾU NGÂN HÀNG TỰ NÂNG CẤP INTERNET BANKING THÌ SAO?
    # =========================================================================
    def build_slide_22_qa_bank_neutrality(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • Q&A PHẢN BIỆN CHUYÊN SÂU • DEEP DIVE 02: TÍNH TRUNG LẬP ĐA NGÂN HÀNG",
            title_text="Q&A 02: \"NẾU NGÂN HÀNG TỰ NÂNG CẤP INTERNET BANKING THÌ LIVA CÓ BỊ THAY THẾ?\"",
            subtitle_text="Giải mã tính trung lập đa ngân hàng và lý do các ngân hàng không bao giờ tích hợp dữ liệu của đối thủ.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(3.777)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="RANH GIỚI", badge_color=COLOR_BLUE_NAVY,
            title="Ranh Giới Lợi Ích Cạnh Tranh",
            bullets=[
                ("Không quản lý đối thủ", "Ngân hàng A (ví dụ Vietcombank) sẽ không bao giờ xây tính năng giúp doanh nghiệp quản lý tiền tại Ngân hàng B (ví dụ Techcombank)."),
                ("Giữ chân tiền gửi CASA", "Mỗi ngân hàng chỉ tập trung bảo vệ nguồn vốn và tối ưu dòng tiền lưu chuyển trong nội bộ hệ thống của riêng mình."),
                ("Rào cản chia sẻ", "Cạnh tranh thị phần khốc liệt khiến việc chia sẻ trực tiếp dữ liệu giữa các ngân hàng thương mại là bất khả thi.")
            ],
            stat_val="1 Ngân Hàng", stat_lbl="Không bao giờ quản lý đối thủ"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="TRỌNG TÀI", badge_color=COLOR_BLUE_PRIMARY,
            title="Vị Thế Trọng Tài Độc Lập",
            bullets=[
                ("Phân tán rủi ro", "Doanh nghiệp vừa và lớn luôn mở từ 5 đến 20 tài khoản ngân hàng để phân tán rủi ro và tận dụng hạn mức tín dụng."),
                ("Bên thứ ba trung lập", "Khách hàng bắt buộc phải có một bên độc lập như LIVA để gom dữ liệu và điều phối dòng tiền tổng thể."),
                ("Bức tranh toàn cảnh", "Cung cấp góc nhìn thanh khoản hợp nhất 360 độ mà không một ngân hàng đơn lẻ nào có thể cung cấp được.")
            ],
            stat_val="5 - 20 TK", stat_lbl="Nhu cầu hợp nhất đa ngân hàng"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="WIN-WIN", badge_color=COLOR_EMERALD,
            title="Quan Hệ Cộng Sinh Thay Vì Đối Đầu",
            bullets=[
                ("Đối tác gia tăng giá trị", "Các ngân hàng thương mại coi LIVA là công cụ giữ chân khách hàng tổ chức và kích hoạt số dư CASA dồi dào."),
                ("Không tốn chi phí phát triển", "Ngân hàng hưởng lợi từ sự gắn kết của doanh nghiệp mà không cần đầu tư hàng triệu USD tự phát triển phần mềm."),
                ("Cơ hội hợp tác bản quyền", "LIVA sẵn sàng cấp phép đóng gói White-label giải pháp cho các ngân hàng thương mại tiên phong.")
            ],
            stat_val="Cộng Sinh", stat_lbl="Gia tăng gắn kết số dư CASA"
        )

        self.add_footer(slide, 22, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(22))

    # =========================================================================
    # SLIDE 23: Q&A 03 — MÔ HÌNH NHỎ (SLM) VS CHATGPT CLOUD
    # =========================================================================
    def build_slide_23_qa_slm_vs_cloud_llm(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • Q&A PHẢN BIỆN CHUYÊN SÂU • DEEP DIVE 03: SLM VS CLOUD LLM",
            title_text="Q&A 03: \"TẠI SAO NGÂN HÀNG BẮT BUỘC DÙNG MÔ HÌNH 3B-8B CỤC BỘ THAY VÌ CHATGPT?\"",
            subtitle_text="Bóc tách bài toán chi phí, độ trễ, quyền riêng tư và sự phù hợp nghiệp vụ của AI tài chính bản địa.",
            badge_color=COLOR_DANGER, badge_bg=COLOR_RED_BG, badge_border=COLOR_RED_BORDER
        )

        left_data = {
            'title': '3 Bẫy Nguy Hiểm Của Cloud LLM Với Ngân Quỹ',
            'subtitle': 'Rủi ro pháp lý và chi phí leo thang của trí tuệ nhân tạo đám mây',
            'bg_color': COLOR_RED_BG,
            'border_color': COLOR_RED_BORDER,
            'header_color': COLOR_DANGER,
            'bullet_icon': '❌',
            'items': [
                (
                    'Vi phạm pháp luật',
                    'Truyền dữ liệu sao kê tài chính ra máy chủ nước ngoài vi phạm trực tiếp Nghị định 13 và Thông tư 09 ngành ngân hàng.'
                ),
                (
                    'Chi phí API đắt đỏ',
                    'Gọi API đám mây xử lý 50.000 dòng sao kê mỗi sáng tốn hàng chục triệu đồng mỗi tháng, chi phí leo thang nhanh theo quy mô.'
                ),
                (
                    'Độ trễ mạng Internet',
                    'Mất 2 đến 5 giây cho mỗi giao dịch qua đường truyền mạng, không đáp ứng được yêu cầu đối soát tức thì đầu giờ sáng.'
                )
            ]
        }

        right_data = {
            'title': '3 Lợi Thế Áp Đảo Của Mô Hình Nhỏ Cục Bộ (SLM)',
            'subtitle': 'Tối ưu hóa chuyên sâu cho dữ liệu ngôn ngữ ngân quỹ Việt Nam',
            'bg_color': COLOR_EMERALD_BG,
            'border_color': COLOR_EMERALD_BORDER,
            'header_color': COLOR_EMERALD,
            'bullet_icon': '✅',
            'items': [
                (
                    'Chuyên biệt nghiệp vụ',
                    'Mô hình 3B-8B tinh chỉnh sâu trên 500.000 mẫu viết tắt tiếng Việt; tập trung bóc tách sao kê thay vì làm thơ hay tán gẫu.'
                ),
                (
                    'Tốc độ xử lý tức thì',
                    'Tốc độ dưới nửa mili-giây (<0.5ms) cho mỗi giao dịch trên máy tính thông thường (tiêu thụ chỉ 2GB RAM), không phụ thuộc mạng.'
                ),
                (
                    'Bảo mật tuyệt đối & Chi phí $0',
                    'Vận hành 100% tại chỗ không rò rỉ dữ liệu, chi phí biến đổi biên bằng $0, mang lại hiệu quả kinh tế lâu dài.'
                )
            ]
        }

        self.add_compare_grid(
            slide, MARGIN_X, Inches(1.65), CONTENT_WIDTH, Inches(4.90),
            left_data, right_data
        )

        self.add_footer(slide, 23, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(23))

    # =========================================================================
    # SLIDE 24: Q&A 04 — XỬ LÝ 0.2% & TRÁCH NHIỆM KHI CÓ SAI SÓT
    # =========================================================================
    def build_slide_24_qa_fail_closed_responsibility(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • Q&A PHẢN BIỆN CHUYÊN SÂU • DEEP DIVE 04: TRÁCH NHIỆM & FAIL-CLOSED",
            title_text="Q&A 04: \"XỬ LÝ SAI SÓT 0.2% NHƯ THẾ NÀO? AI CHỊU TRÁCH NHIỆM KHI MẤT TIỀN?\"",
            subtitle_text="Triết lý khóa an toàn Fail-Closed và cơ chế phân quyền phê duyệt hai pha Maker-Checker.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        # Top Banner: Stat 0.2% Isolation
        banner_h = Inches(1.32)
        top_y = Inches(1.65)
        self.add_card(slide, MARGIN_X, top_y, CONTENT_WIDTH, banner_h, bg_color=COLOR_STRIPE_BLUE, border_color=COLOR_STRIPE_BORDER)

        # Left accent stripe
        b_stripe = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, MARGIN_X, top_y, Inches(0.12), banner_h)
        b_stripe.fill.solid()
        b_stripe.fill.fore_color.rgb = COLOR_DANGER
        b_stripe.line.fill.background()

        # Big Stat Value
        tx_stat = slide.shapes.add_textbox(MARGIN_X + Inches(0.28), top_y + Inches(0.12), Inches(2.6), banner_h - Inches(0.24))
        tf_stat = tx_stat.text_frame
        tf_stat.word_wrap = True
        tf_stat.margin_left = tf_stat.margin_top = tf_stat.margin_right = tf_stat.margin_bottom = 0
        p_val = tf_stat.paragraphs[0]
        p_val.text = "0.2%"
        p_val.font.name = FONT_FAMILY
        p_val.font.size = Pt(36)
        p_val.font.bold = True
        p_val.font.color.rgb = COLOR_DANGER

        p_sub = tf_stat.add_paragraph()
        p_sub.text = "CÁCH LY AN TOÀN"
        p_sub.font.name = FONT_FAMILY
        p_sub.font.size = Pt(9.5)
        p_sub.font.bold = True
        p_sub.font.color.rgb = COLOR_DARK_SLATE

        # Banner Description
        tx_desc = slide.shapes.add_textbox(MARGIN_X + Inches(3.0), top_y + Inches(0.15), CONTENT_WIDTH - Inches(3.2), banner_h - Inches(0.30))
        tf_desc = tx_desc.text_frame
        tf_desc.word_wrap = True
        tf_desc.margin_left = tf_desc.margin_top = tf_desc.margin_right = tf_desc.margin_bottom = 0
        p_dt = tf_desc.paragraphs[0]
        p_dt.text = "CƠ CHẾ CÁCH LY KHÓA AN TOÀN TỨC THÌ (FAIL-CLOSED PROTOCOL)"
        p_dt.font.name = FONT_FAMILY
        p_dt.font.size = Pt(11.5)
        p_dt.font.bold = True
        p_dt.font.color.rgb = COLOR_DARK_SLATE
        p_dt.space_after = Pt(4)

        p_dd = tf_desc.add_paragraph()
        p_dd.text = "Khi gặp giao dịch mơ hồ hoặc sai lệch số học dù chỉ 1 đồng, LIVA lập tức cách ly 0.2% dòng nghi vấn (100 dòng / 50.000 dòng) chuyển Kế toán trưởng xem xét kèm phương án gợi ý. AI tuyệt đối không bao giờ tự ý đoán mò tiền của doanh nghiệp."
        p_dd.font.name = FONT_FAMILY
        p_dd.font.size = Pt(9.5)
        p_dd.font.color.rgb = COLOR_TEXT_BODY

        # 3 Bottom Principles Cards
        card_w = Inches(3.777)
        card_h = Inches(3.38)
        bottom_y = top_y + banner_h + Inches(0.20)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, bottom_y, card_w, card_h,
            badge_text="KHÓA", badge_color=COLOR_DANGER,
            title="Cơ Chế Khóa Fail-Closed",
            bullets=[
                ("Không bao giờ đoán mò", "Khi số liệu chênh lệch dù chỉ 1 đồng hoặc nội dung không rõ ràng, hệ thống kiên quyết dừng lại chờ xác nhận."),
                ("Nguyên tắc thận trọng", "Tuyệt đối không áp đặt giả định toán học lên tài sản và tiền bạc của doanh nghiệp."),
                ("Tự động học hỏi", "Mỗi ca xử lý thủ công thành công đều được ghi nhận để hoàn thiện từ điển nhận diện cho các lần sau.")
            ]
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, bottom_y, card_w, card_h,
            badge_text="2 PHA", badge_color=COLOR_BLUE_NAVY,
            title="Quy Trình Maker-Checker",
            bullets=[
                ("Phân định thẩm quyền", "AI chỉ đóng vai trò người lập đề xuất (Maker), hỗ trợ chuẩn bị sẵn thông tin đối soát và dự thảo bút toán."),
                ("Phê duyệt điện tử", "Kế toán trưởng hoặc CFO (Checker) ký duyệt điện tử thì lệnh mới có giá trị pháp lý và được hạch toán vào sổ cái."),
                ("Con người quyết định", "Quyền quyết định tối cao về dòng tiền luôn nằm trọn trong tay người quản trị tài chính.")
            ]
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, bottom_y, card_w, card_h,
            badge_text="KIỂM TOÁN", badge_color=COLOR_EMERALD,
            title="Nhật Ký Bất Biến & Pháp Lý",
            bullets=[
                ("Lưu vết 100%", "Ghi nhận chi tiết thời điểm, người duyệt, nội dung ban đầu và phương án điều chỉnh trong nhật ký kiểm toán bất biến."),
                ("Trách nhiệm minh bạch", "Phân định rõ ràng trách nhiệm giữa đề xuất của máy và quyết định phê duyệt của con người."),
                ("Sẵn sàng thanh tra", "Cung cấp đầy đủ bằng chứng đối chiếu phục vụ thanh tra thuế và các đơn vị kiểm toán độc lập quốc tế.")
            ]
        )

        self.add_footer(slide, 24, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(24))

    # =========================================================================
    # SLIDE 25: Q&A 05 — CHI TIẾT SỬ DỤNG VỐN GỌI & KẾ HOẠCH DỰ PHÒNG
    # =========================================================================
    def build_slide_25_qa_capital_tranches(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • Q&A PHẢN BIỆN CHUYÊN SÂU • DEEP DIVE 05: PHÂN BỔ NGUỒN VỐN",
            title_text="Q&A 05: \"CHI TIẾT PHÂN BỔ VỐN SEED: 50% R&D, 25% GTM, 15% PHÁP LÝ & 10% DỰ PHÒNG\"",
            subtitle_text="Sử dụng nguồn vốn có kỷ luật, bám sát các mốc nghiệm thu kỹ thuật và chỉ số tăng trưởng.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)

        # Left Column: Donut Chart
        chart_w = Inches(5.0)
        categories = [
            "R&D & Bộ Kết Nối 35+ NH (50%)",
            "Phát Triển GTM & Kênh ERP (25%)",
            "Kiểm Toán ISO & Sandbox (15%)",
            "Quỹ Dự Phòng 18-24 Tháng (10%)"
        ]
        values = [50, 25, 15, 10]
        colors = [COLOR_BLUE_NAVY, COLOR_BLUE_PRIMARY, COLOR_EMERALD, COLOR_AMBER]

        self.add_donut_chart(
            slide, MARGIN_X, top_y, chart_w, height,
            categories, values, colors,
            title="CƠ CẤU PHÂN BỔ VỐN SEED ($500K - $750K USD)"
        )

        # Right Column: 3 Tranche-Gated Disbursement Cards
        right_x = MARGIN_X + chart_w + Inches(0.20)
        right_w = CONTENT_WIDTH - chart_w - Inches(0.20)
        gap = Inches(0.12)
        t_card_h = (height - gap * 2) / 3

        tranches_data = [
            (
                "ĐỢT 1", "40%", COLOR_BLUE_NAVY,
                "Đợt 1 (40% Vốn — Ngay Khi Đóng Vòng)",
                [
                    ("Mục tiêu kỹ thuật", "Kích hoạt hoàn thiện sản phẩm và mở rộng bộ giải mã sao kê 20 ngân hàng thương mại lớn nhất."),
                    ("Mở rộng nhân sự", "Tuyển dụng 4 kỹ sư hệ thống cao cấp và chuyên viên nghiệp vụ đối soát ngân quỹ."),
                    ("Thử nghiệm thực tế", "Hoàn thành kiểm thử nội bộ cùng 15 khách hàng doanh nghiệp thân thiết đầu tiên.")
                ]
            ),
            (
                "ĐỢT 2", "35%", COLOR_BLUE_PRIMARY,
                "Đợt 2 (35% Vốn — Tháng Thứ 06)",
                [
                    ("Mốc nghiệm thu ERP", "Tích hợp 1 chạm vào các phần mềm kế toán phổ biến MISA, FAST và Bravo."),
                    ("Chỉ số thị trường", "Đạt cột mốc 50 khách hàng doanh nghiệp trả phí định kỳ với tỷ lệ hài lòng > 90%."),
                    ("Vận hành an toàn", "Hoàn thiện cơ chế phê duyệt hai pha Maker-Checker và phân quyền chữ ký số.")
                ]
            ),
            (
                "ĐỢT 3", "25%", COLOR_EMERALD,
                "Đợt 3 (25% Vốn — Tháng Thứ 12)",
                [
                    ("Mốc đối tác ngân hàng", "Ký kết thỏa thuận thử nghiệm chính thức cùng 02 ngân hàng thương mại tiên phong."),
                    ("Chứng nhận an ninh", "Hoàn tất kiểm toán an ninh độc lập ISO 27001 và nộp hồ sơ Sandbox Ngân hàng Nhà nước."),
                    ("Tự chủ tài chính", "Chạm mốc 150 doanh nghiệp sử dụng và chuẩn bị đạt điểm hòa vốn vận hành.")
                ]
            )
        ]

        for i, (badge_lbl, badge_pct, color, title, bullets) in enumerate(tranches_data):
            ty = top_y + i * (t_card_h + gap)
            self.add_card(slide, right_x, ty, right_w, t_card_h, bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER)

            # Left Badge Box
            b_box = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, right_x + Inches(0.12), ty + Inches(0.12), Inches(1.35), t_card_h - Inches(0.24))
            b_box.fill.solid()
            b_box.fill.fore_color.rgb = COLOR_STRIPE_BLUE
            b_box.line.color.rgb = color
            b_box.line.width = Pt(1)

            tf_bb = b_box.text_frame
            tf_bb.word_wrap = True
            tf_bb.margin_left = tf_bb.margin_top = tf_bb.margin_right = tf_bb.margin_bottom = 0

            p_pct = tf_bb.paragraphs[0]
            p_pct.text = badge_pct
            p_pct.font.name = FONT_FAMILY
            p_pct.font.size = Pt(22)
            p_pct.font.bold = True
            p_pct.font.color.rgb = color
            p_pct.alignment = PP_ALIGN.CENTER

            p_lbl = tf_bb.add_paragraph()
            p_lbl.text = badge_lbl
            p_lbl.font.name = FONT_FAMILY
            p_lbl.font.size = Pt(8.5)
            p_lbl.font.bold = True
            p_lbl.font.color.rgb = COLOR_DARK_SLATE
            p_lbl.alignment = PP_ALIGN.CENTER

            # Content Text Box
            tx_c = slide.shapes.add_textbox(right_x + Inches(1.60), ty + Inches(0.10), right_w - Inches(1.75), t_card_h - Inches(0.20))
            tf_c = tx_c.text_frame
            tf_c.word_wrap = True
            tf_c.margin_left = tf_c.margin_top = tf_c.margin_right = tf_c.margin_bottom = 0

            p_t = tf_c.paragraphs[0]
            p_t.text = title
            p_t.font.name = FONT_FAMILY
            p_t.font.size = Pt(10.5)
            p_t.font.bold = True
            p_t.font.color.rgb = COLOR_DARK_SLATE
            p_t.space_after = Pt(2)

            for item in bullets:
                p_b = tf_c.add_paragraph()
                p_b.space_after = Pt(1)
                pre, desc = item
                r_pre = p_b.add_run()
                r_pre.text = f"• {pre}: "
                r_pre.font.name = FONT_FAMILY
                r_pre.font.size = Pt(8.8)
                r_pre.font.bold = True
                r_pre.font.color.rgb = COLOR_DARK_SLATE

                r_desc = p_b.add_run()
                r_desc.text = desc
                r_desc.font.name = FONT_FAMILY
                r_desc.font.size = Pt(8.5)
                r_desc.font.color.rgb = COLOR_TEXT_BODY

        self.add_footer(slide, 25, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(25))

    # =========================================================================
    # SLIDE 26: Q&A 06 — CHIẾN LƯỢC BẢO VỆ BẢN QUYỀN TRƯỚC BIG TECH
    # =========================================================================
    def build_slide_26_qa_ip_defense(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • Q&A PHẢN BIỆN CHUYÊN SÂU • DEEP DIVE 06: HÀO LŨY BẢN QUYỀN",
            title_text="Q&A 06: \"CHIẾN LƯỢC BẢO VỆ BẢN QUYỀN & PHÒNG THỦ TRƯỚC CÁC TẬP ĐOÀN BIG TECH\"",
            subtitle_text="Đăng ký sở hữu trí tuệ, làm chủ bộ ngữ liệu tiếng Việt tài chính độc quyền và kiến trúc phần cứng khép kín.",
            badge_color=COLOR_BLUE_NAVY, badge_bg=COLOR_STRIPE_BLUE, badge_border=COLOR_STRIPE_BORDER
        )

        card_w = Inches(3.777)
        card_h = Inches(4.90)
        top_y = Inches(1.65)
        gap = Inches(0.20)

        self.add_square_badge_card(
            slide, MARGIN_X, top_y, card_w, card_h,
            badge_text="IP", badge_color=COLOR_BLUE_NAVY,
            title="Đăng Ký Bản Quyền & SHTT",
            bullets=[
                ("Bảo hộ quyền tác giả", "Hoàn tất thủ tục bảo hộ quyền tác giả cho toàn bộ mã nguồn phần mềm và giải pháp kiến trúc hai tầng."),
                ("Chuyển giao vô điều kiện", "100% thành viên sáng lập ký hợp đồng cam kết chuyển giao toàn bộ quyền sở hữu trí tuệ cho pháp nhân LIVA."),
                ("Bảo mật đa tầng", "Áp dụng thỏa thuận bảo mật nghiêm ngặt (NDA) và phân tách quyền truy cập mã nguồn theo từng module độc lập.")
            ],
            stat_val="100% SHTT", stat_lbl="Thuộc sở hữu pháp nhân LIVA"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + card_w + gap, top_y, card_w, card_h,
            badge_text="DATA", badge_color=COLOR_BLUE_PRIMARY,
            title="Bộ Ngữ Liệu Bản Địa Độc Quyền",
            bullets=[
                ("Kho dữ liệu bản địa", "Sở hữu hơn 500.000 cặp mẫu giao dịch viết tắt tiếng Việt thực tế được chuẩn hóa và gắn nhãn nghiệp vụ."),
                ("Hào lũy không thể sao chép", "Các mô hình AI toàn cầu không thể tiếp cận dữ liệu sao kê ngân quỹ nội địa để huấn luyện đối trọng."),
                ("Độ chính xác vượt trội", "Hiểu sâu sắc cách viết tắt, từ lóng kế toán và thói quen chuyển khoản đặc thù tại thị trường Việt Nam.")
            ],
            stat_val="500.000 Mẫu", stat_lbl="Dữ liệu giao dịch bản địa độc quyền"
        )

        self.add_square_badge_card(
            slide, MARGIN_X + (card_w + gap) * 2, top_y, card_w, card_h,
            badge_text="SEAL", badge_color=COLOR_EMERALD,
            title="Kiến Trúc Đóng Gói Khép Kín",
            bullets=[
                ("Biên dịch mã máy", "Phần mềm được biên dịch khép kín trực tiếp thành mã máy, ngăn chặn triệt để nguy cơ phân tích sao chép ngược."),
                ("Khóa phần cứng an toàn", "Cơ chế kích hoạt bản quyền gắn liền với định danh phần cứng máy tính nội bộ của từng khách hàng doanh nghiệp."),
                ("Đóng hộp độc lập", "Khách hàng sở hữu toàn quyền vận hành mà không phải lo ngại về nguy cơ cửa sau hoặc rò rỉ mã nguồn.")
            ],
            stat_val="Khóa Cứng", stat_lbl="Bảo vệ mã máy chống sao chép"
        )

        self.add_footer(slide, 26, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(26))

    # =========================================================================
    # SLIDE 27: TỔNG KẾT, LIÊN HỆ & TUYÊN NGÔN CHỦ QUYỀN AI TÀI CHÍNH
    # =========================================================================
    def build_slide_27_closing_vision(self):
        slide = self.add_blank_slide()
        self.add_header(
            slide,
            badge_text="VÒNG 3 • TỔNG KẾT & KÊU GỌI HỢP TÁC • CLOSING CALL TO ACTION",
            title_text="LIVA BANKING HARNESS: KHẲNG ĐỊNH CHỦ QUYỀN AI TÀI CHÍNH TẠI VIỆT NAM",
            subtitle_text="Sẵn sàng đồng hành cùng các Quỹ đầu tư và Ngân hàng tiên phong thiết lập chuẩn mực mới cho tự động hóa ngân quỹ.",
            badge_color=COLOR_EMERALD, badge_bg=COLOR_EMERALD_BG, badge_border=COLOR_EMERALD_BORDER
        )

        top_y = Inches(1.65)
        height = Inches(4.90)

        # Left Column: Vision, 3 Commitments & CTA
        left_w = Inches(6.90)

        # Vision Banner
        v_h = Inches(1.10)
        self.add_card(slide, MARGIN_X, top_y, left_w, v_h, bg_color=COLOR_STRIPE_BLUE, border_color=COLOR_STRIPE_BORDER)

        tx_v = slide.shapes.add_textbox(MARGIN_X + Inches(0.20), top_y + Inches(0.12), left_w - Inches(0.40), v_h - Inches(0.24))
        tf_v = tx_v.text_frame
        tf_v.word_wrap = True
        tf_v.margin_left = tf_v.margin_top = tf_v.margin_right = tf_v.margin_bottom = 0

        p_vt = tf_v.paragraphs[0]
        p_vt.text = "TUYÊN NGÔN GIÁ TRỊ CỐT LÕI"
        p_vt.font.name = FONT_FAMILY
        p_vt.font.size = Pt(8.5)
        p_vt.font.bold = True
        p_vt.font.color.rgb = COLOR_BLUE_PRIMARY
        p_vt.space_after = Pt(2)

        p_vc = tf_v.add_paragraph()
        p_vc.text = "“Tự Động Hóa Ngân Quỹ Doanh Nghiệp: Tốc Độ Tức Thì, Chuẩn Xác Từng Đồng & Tuyệt Đối An Toàn Tại Chỗ.”"
        p_vc.font.name = FONT_FAMILY
        p_vc.font.size = Pt(11.5)
        p_vc.font.bold = True
        p_vc.font.color.rgb = COLOR_DARK_SLATE

        # 3 Core Commitments Cards
        comm_y = top_y + v_h + Inches(0.12)
        comm_h = Inches(1.55)
        c_gap = Inches(0.12)
        c_w = (left_w - c_gap * 2) / 3

        commitments = [
            ("100%", "ZERO DATA EGRESS", "Bảo vệ bí mật tài chính tối cao theo Nghị định 13.", COLOR_BLUE_NAVY),
            ("0%", "ẢO GIÁC SỐ HỌC", "Động cơ toán học xác định bảo đảm chuẩn xác từng đồng.", COLOR_EMERALD),
            ("< 3 Th", "HOÀN VỐN ĐẦU TƯ", "Mang lại giá trị kinh tế trực tiếp và tức thì cho doanh nghiệp.", COLOR_BLUE_PRIMARY)
        ]

        for i, (c_val, c_lbl, c_desc, c_col) in enumerate(commitments):
            cx = MARGIN_X + i * (c_w + c_gap)
            self.add_card(slide, cx, comm_y, c_w, comm_h, bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER)

            # Accent top line
            bar = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, cx, comm_y, c_w, Inches(0.06))
            bar.fill.solid()
            bar.fill.fore_color.rgb = c_col
            bar.line.fill.background()

            tx_c = slide.shapes.add_textbox(cx + Inches(0.10), comm_y + Inches(0.12), c_w - Inches(0.20), comm_h - Inches(0.24))
            tf_c = tx_c.text_frame
            tf_c.word_wrap = True
            tf_c.margin_left = tf_c.margin_top = tf_c.margin_right = tf_c.margin_bottom = 0

            p_v = tf_c.paragraphs[0]
            p_v.text = c_val
            p_v.font.name = FONT_FAMILY
            p_v.font.size = Pt(20)
            p_v.font.bold = True
            p_v.font.color.rgb = c_col

            p_l = tf_c.add_paragraph()
            p_l.text = c_lbl
            p_l.font.name = FONT_FAMILY
            p_l.font.size = Pt(8.0)
            p_l.font.bold = True
            p_l.font.color.rgb = COLOR_DARK_SLATE
            p_l.space_after = Pt(2)

            p_d = tf_c.add_paragraph()
            p_d.text = c_desc
            p_d.font.name = FONT_FAMILY
            p_d.font.size = Pt(8.0)
            p_d.font.color.rgb = COLOR_TEXT_BODY

        # Call To Action Box (Bottom)
        cta_y = comm_y + comm_h + Inches(0.12)
        cta_h = height - (v_h + Inches(0.12) + comm_h + Inches(0.12))
        self.add_card(slide, MARGIN_X, cta_y, left_w, cta_h, bg_color=COLOR_CARD_BG, border_color=COLOR_CARD_BORDER)

        tx_cta = slide.shapes.add_textbox(MARGIN_X + Inches(0.18), cta_y + Inches(0.12), left_w - Inches(0.36), cta_h - Inches(0.24))
        tf_cta = tx_cta.text_frame
        tf_cta.word_wrap = True
        tf_cta.margin_left = tf_cta.margin_top = tf_cta.margin_right = tf_cta.margin_bottom = 0

        p_ch = tf_cta.paragraphs[0]
        p_ch.text = "KÊU GỌI ĐẦU TƯ & ĐỐI TÁC CHIẾN LƯỢC"
        p_ch.font.name = FONT_FAMILY
        p_ch.font.size = Pt(10.5)
        p_ch.font.bold = True
        p_ch.font.color.rgb = COLOR_DARK_SLATE
        p_ch.space_after = Pt(3)

        cta_items = [
            ("Vòng gọi vốn Seed", "$500,000 – $750,000 USD (12.0% – 15.0% cổ phần, định giá $4.0M – $5.0M)"),
            ("Đối tác Ngân hàng", "Tìm kiếm 02 Ngân hàng Thương mại tiên phong thử nghiệm giải pháp Sandbox"),
            ("Email & Hotline", "investors@liva-ai.vn | Hotline: +84 (0) 90 123 4567"),
            ("Địa chỉ văn phòng", "Phòng Lab Công nghệ LIVA Banking, Hà Nội / TP. Hồ Chí Minh")
        ]

        for pre, desc in cta_items:
            p_it = tf_cta.add_paragraph()
            p_it.space_after = Pt(2)
            r_pre = p_it.add_run()
            r_pre.text = f"• {pre}: "
            r_pre.font.name = FONT_FAMILY
            r_pre.font.size = Pt(8.8)
            r_pre.font.bold = True
            r_pre.font.color.rgb = COLOR_DARK_SLATE

            r_desc = p_it.add_run()
            r_desc.text = desc
            r_desc.font.name = FONT_FAMILY
            r_desc.font.size = Pt(8.5)
            r_desc.font.color.rgb = COLOR_TEXT_BODY

        # Right Column: Architectural Photography Panel
        photo_x = MARGIN_X + left_w + Inches(0.20)
        photo_w = CONTENT_WIDTH - left_w - Inches(0.20)
        photo_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets", "glass_skyscraper.jpg")
        self.add_photo_panel(
            slide, photo_path, photo_x, top_y, photo_w, height,
            caption="THIẾT LẬP CHUẨN MỰC TỰ ĐỘNG HÓA NGÂN QUỸ TẠI VIỆT NAM"
        )

        self.add_footer(slide, 27, total_slides=27)
        self.set_speaker_notes(slide, self.get_speaker_note(27))

    # =========================================================================
    # MASTER BUILDER ENTRYPOINT
    # =========================================================================
    def build_all(self):
        print("[*] Starting LIVA Banking Harness 27-Slide Presentation Generation...")
        slide_methods = [
            (1, "Title & Vision SplitHero", self.build_slide_01_title),
            (2, "The Problem", self.build_slide_02_problem),
            (3, "Regulatory Wall", self.build_slide_03_regulatory),
            (4, "Ideal Customer Profile", self.build_slide_04_customer),
            (5, "Solution Overview", self.build_slide_05_solution),
            (6, "Before vs After CompareGrid", self.build_slide_06_compare_grid),
            (7, "User Journey StepChevron", self.build_slide_07_user_journey),
            (8, "Two-Tier Architecture", self.build_slide_08_core_tech),
            (9, "Lab Benchmarks StatBox", self.build_slide_09_lab_benchmark),
            (10, "Business Model", self.build_slide_10_business_model),
            (11, "GTM Flywheel StepChevron", self.build_slide_11_gtm),
            (12, "Competitive Moat", self.build_slide_12_competitive_moat),
            (13, "Seed Ask DonutChart", self.build_slide_13_seed_ask),
            (14, "Execution Roadmap StepChevron", self.build_slide_14_roadmap),
            (15, "Unit Economics StatBox", self.build_slide_15_unit_economics),
            (16, "Revenue Projections", self.build_slide_16_financial_projections),
            (17, "Cost Structure DonutChart", self.build_slide_17_cost_structure),
            (18, "Regulatory Compliance", self.build_slide_18_regulatory_compliance),
            (19, "Founding Team", self.build_slide_19_founding_team),
            (20, "Risk Management", self.build_slide_20_risk_management),
            (21, "Q&A 01: Excel vs LIVA CompareGrid", self.build_slide_21_qa_excel_vs_liva),
            (22, "Q&A 02: Multi-Bank Neutrality", self.build_slide_22_qa_bank_neutrality),
            (23, "Q&A 03: SLM vs Cloud LLM CompareGrid", self.build_slide_23_qa_slm_vs_cloud_llm),
            (24, "Q&A 04: Fail-Closed 0.2% & Maker-Checker", self.build_slide_24_qa_fail_closed_responsibility),
            (25, "Q&A 05: Capital Tranches DonutChart", self.build_slide_25_qa_capital_tranches),
            (26, "Q&A 06: IP Defense & Anti-BigTech", self.build_slide_26_qa_ip_defense),
            (27, "Closing Vision SplitHero", self.build_slide_27_closing_vision),
        ]

        for num, desc, method in slide_methods:
            print(f"  [+] Building Slide {num:02d} / 27: {desc}...")
            method()

        os.makedirs(os.path.dirname(os.path.abspath(self.output_path)), exist_ok=True)
        self.prs.save(self.output_path)
        print(f"[✓] Presentation generated successfully: {self.output_path}")
        print(f"[✓] Total Slides: {len(self.prs.slides)}")


def verify_presentation(pptx_path):
    print("\n" + "=" * 60)
    print("VERIFYING GENERATED DECK INTEGRITY...")
    print("=" * 60)
    if not os.path.exists(pptx_path):
        raise FileNotFoundError(f"Deck file not found: {pptx_path}")

    prs = Presentation(pptx_path)
    num_slides = len(prs.slides)
    print(f"[*] Slide count: {num_slides} / 27")
    if num_slides != 27:
        raise ValueError(f"Expected exactly 27 slides, got {num_slides}!")

    banned_jargon = [
        "SIMD", "AVX", "AHash", "Rust", "deserializ", "u64", "Disentangle",
        "DPAPI", "Win32", "ReadDirectoryChanges", "HMAC", "SHA256", "Flatbuffers",
        "zero-copy", "AST"
    ]

    all_jargon_found = []
    for i, slide in enumerate(prs.slides, 1):
        # Verify speaker notes
        notes_text = ""
        if slide.notes_slide and slide.notes_slide.notes_text_frame:
            notes_text = slide.notes_slide.notes_text_frame.text.strip()
        if len(notes_text) < 100:
            raise ValueError(f"Slide {i} speaker notes too short ({len(notes_text)} chars)!")

        # Verify no banned jargon on visible shapes
        for shape in slide.shapes:
            if shape.has_text_frame:
                txt = shape.text_frame.text
                for bj in banned_jargon:
                    if re.search(r'\b' + re.escape(bj) + r'\b', txt, re.IGNORECASE):
                        all_jargon_found.append((i, bj, txt[:60]))

        # Verify Slide 14 specific bug check
        if i == 14:
            for shape in slide.shapes:
                if shape.has_text_frame:
                    if "Chạy trên máy tính nội bộ" in shape.text_frame.text:
                        raise ValueError("Slide 14 contains outdated bullet: 'Chạy trên máy tính nội bộ'")

    if all_jargon_found:
        print("[!] BANNED JARGON VIOLATIONS FOUND:")
        for slide_idx, term, snippet in all_jargon_found:
            print(f"  - Slide {slide_idx}: '{term}' found in: {snippet}")
        raise ValueError(f"Found {len(all_jargon_found)} banned jargon violations on slide shapes!")

    print("[✓] ALL 27 SLIDES VERIFIED CLEAN:")
    print("    - Exactly 27 slides created.")
    print("    - 100% speaker notes present (>100 chars per slide).")
    print("    - 0 banned engineer jargon violations.")
    print("    - Slide 14 verified free of stale roadmap bullets.")
    print("=" * 60 + "\n")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        out_path = sys.argv[1]
    else:
        out_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "liva_innostart_2026.pptx")

    builder = LivaDeckBuilder(out_path)
    builder.build_all()
    verify_presentation(out_path)
