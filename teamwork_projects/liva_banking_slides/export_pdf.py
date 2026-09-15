#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
export_pdf.py
Exports index.html to a pristine 27-page 16:9 vector PDF (liva_innostart_2026.pdf)
using Playwright Chromium with native @media print vector rendering.
Validates 27 pages, 16x9 dimensions, 27 unique page headers, and zero blank pages.
"""

import os
import sys
import pypdf
from playwright.sync_api import sync_playwright

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')
if hasattr(sys.stderr, 'reconfigure'):
    sys.stderr.reconfigure(encoding='utf-8')

def export_deck():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    html_file = os.path.join(script_dir, 'index.html')
    output_pdf = os.path.join(script_dir, 'liva_innostart_2026.pdf')

    if not os.path.exists(html_file):
        raise FileNotFoundError(f"Source HTML file not found: {html_file}")

    file_url = f"file:///{html_file.replace(os.sep, '/')}"
    print(f"[*] Starting 27-slide PDF export from {file_url} ...")

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={'width': 1920, 'height': 1080})
        page.goto(file_url, wait_until='networkidle')

        # Ensure all web components, vector SVGs, and web fonts are fully rendered
        page.wait_for_timeout(1500)

        # Vector export of all 27 slides via @media print stylesheet
        page.pdf(
            path=output_pdf,
            width='16in',
            height='9in',
            print_background=True,
            prefer_css_page_size=True
        )
        browser.close()

    # Deep pypdf verification
    print(f"[*] Analyzing generated PDF: {output_pdf}")
    reader = pypdf.PdfReader(output_pdf)
    total_pages = len(reader.pages)
    file_size_kb = os.path.getsize(output_pdf) / 1024
    print(f"[*] Export complete! Total pages: {total_pages}, file size: {file_size_kb:.1f} KB")

    # 1. Slide Count Verification
    assert total_pages == 27, f"Expected exactly 27 pages, got {total_pages}"
    print(f"[✓] Page count check passed: exactly {total_pages} pages.")

    # 2. Dimensions and Zero-Blank Verification for all 27 pages
    slide_titles = []
    print("\n--- PER-PAGE VERIFICATION MATRIX ---")
    for i, p in enumerate(reader.pages):
        page_num = i + 1
        w = float(p.mediabox.width) / 72.0
        h = float(p.mediabox.height) / 72.0
        txt = p.extract_text().strip()
        lines = [line.strip() for line in txt.split('\n') if line.strip()]

        # Dimension assertion
        assert abs(w - 16.0) < 0.1 and abs(h - 9.0) < 0.1, (
            f"Page {page_num} dimensions {w:.2f}in x {h:.2f}in do not match 16.0in x 9.0in"
        )

        # Zero blank or clipping assertion
        assert len(txt) >= 200, f"Page {page_num} text too short ({len(txt)} chars), possible blank or clipping!"
        assert len(lines) >= 5, f"Page {page_num} has only {len(lines)} lines, possible clipping!"

        # Extract Action Title (skip badge prefixes)
        action_title = ""
        for line in lines:
            if len(line) > 15 and not any(line.startswith(pfx) for pfx in ["VÒNG 1", "VÒNG 2", "VÒNG 3", "INNOSTART", "TRANG", "ROUND"]):
                action_title = line
                break
        if not action_title and lines:
            action_title = lines[0]

        slide_titles.append(action_title)
        print(f"  [Page {page_num:02d} OK] {w:.1f}\"x{h:.1f}\" | {len(txt):4d} chars | {len(lines):2d} lines | Title: {action_title[:55]}...")

    # 3. Unique Headers Verification
    unique_titles = set(slide_titles)
    print(f"\n[*] Unique Action Title headers: {len(unique_titles)} / 27")
    assert len(unique_titles) == 27, f"Expected 27 distinct slide headers, got {len(unique_titles)}: {unique_titles}"
    print("[✓] All 27 pages have 100% unique, non-duplicate Action Titles!")

    print("\n============================================================")
    print("[✓] PDF EXPORT & VERIFICATION PASSED WITH 100% EXCELLENCE!")
    print(f"    - Output Path: {output_pdf}")
    print(f"    - Pages: {total_pages} / 27 (16:9 Landscape 16.0\" x 9.0\")")
    print(f"    - Unique Headers: {len(unique_titles)} / 27")
    print(f"    - Zero Blank Pages: Verified (all pages >200 chars)")
    print("============================================================\n")
    return output_pdf

if __name__ == '__main__':
    export_deck()

