#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
sync_drive.py
Automated Synchronization & Verification Pipeline for LIVA Banking Harness INNOSTART 2026 Deliverables.

Synchronizes final deliverables from local repository:
  e:/Project/01_AI_Agents/LIVA_Banking/teamwork_projects/liva_banking_slides/
to Google Drive target:
  G:\My Drive\01_Du_An_Khoi_Nghiep\LIVA_InnoStart_2026\

Key Tasks:
1. Core Deliverables:
   - index.html
   - liva_innostart_2026.pptx
   - liva_innostart_2026.pdf
   - assets/ directory (photographic assets)
2. Aliased 27-Slide Deliverables (official naming convention):
   - LIVA_INNOSTART_2026_PitchDeck_27Slides.html
   - LIVA_INNOSTART_2026_PitchDeck_27Slides.pptx
   - LIVA_INNOSTART_2026_PitchDeck_27Slides.pdf
3. Synchronize both to:
   - Target root: G:\My Drive\01_Du_An_Khoi_Nghiep\LIVA_InnoStart_2026\
   - Subfolder:   G:\My Drive\01_Du_An_Khoi_Nghiep\LIVA_InnoStart_2026\Slide_Demo_Day_INNOSTART_2026\
4. Robust Integrity:
   - File size comparison
   - SHA256 checksum verification
   - Timestamp reporting
"""

import os
import sys
import shutil
import hashlib
import datetime
import argparse
import json
from pathlib import Path

# Enforce UTF-8 on Windows stdout/stderr
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')
if hasattr(sys.stderr, 'reconfigure'):
    sys.stderr.reconfigure(encoding='utf-8')

DEFAULT_SOURCE_DIR = os.path.dirname(os.path.abspath(__file__))
DEFAULT_TARGET_BASE = r"G:\My Drive\01_Du_An_Khoi_Nghiep\LIVA_InnoStart_2026"
DEMO_DAY_SUBFOLDER = "Slide_Demo_Day_INNOSTART_2026"

def compute_sha256(filepath, chunk_size=65536):
    """Compute SHA256 hex digest of a file in binary chunks."""
    h = hashlib.sha256()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(chunk_size), b''):
            h.update(chunk)
    return h.hexdigest()

def format_size(size_bytes):
    """Format byte size into human readable string."""
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 * 1024:
        return f"{size_bytes / 1024:.1f} KB"
    else:
        return f"{size_bytes / (1024 * 1024):.2f} MB"

def get_iso_timestamp(filepath):
    """Get file modification time in ISO 8601 format."""
    mtime = os.path.getmtime(filepath)
    return datetime.datetime.fromtimestamp(mtime).isoformat()

def sync_single_file(src_path, dst_path, dry_run=False, verify_only=False):
    """
    Safely copy src_path to dst_path and verify SHA256 and byte size.
    Returns a dict with verification details.
    """
    if not os.path.isfile(src_path):
        raise FileNotFoundError(f"Source file not found: {src_path}")

    src_size = os.path.getsize(src_path)
    src_sha = compute_sha256(src_path)
    src_mtime = get_iso_timestamp(src_path)

    dst_dir = os.path.dirname(dst_path)
    if not dry_run and not verify_only:
        os.makedirs(dst_dir, exist_ok=True)

    copied = False
    need_copy = True

    if os.path.exists(dst_path):
        try:
            dst_size = os.path.getsize(dst_path)
            dst_sha = compute_sha256(dst_path)
            if dst_size == src_size and dst_sha == src_sha:
                need_copy = False
        except Exception:
            need_copy = True

    if verify_only:
        if not os.path.exists(dst_path):
            raise FileNotFoundError(f"Destination file missing: {dst_path}")
        dst_size = os.path.getsize(dst_path)
        dst_sha = compute_sha256(dst_path)
        dst_mtime = get_iso_timestamp(dst_path)
        if dst_size != src_size:
            raise ValueError(f"Size mismatch: src={src_size} vs dst={dst_size}")
        if dst_sha != src_sha:
            raise ValueError(f"SHA256 mismatch: src={src_sha} vs dst={dst_sha}")
        return {
            "src_path": src_path,
            "dst_path": dst_path,
            "size": dst_size,
            "sha256": dst_sha,
            "mtime": dst_mtime,
            "copied": False,
            "status": "VERIFIED_EXISTING"
        }

    if need_copy:
        if not dry_run:
            shutil.copy2(src_path, dst_path)
            copied = True
        else:
            copied = True

    if not dry_run:
        if not os.path.exists(dst_path):
            raise RuntimeError(f"Destination file missing after copy: {dst_path}")
        dst_size = os.path.getsize(dst_path)
        dst_sha = compute_sha256(dst_path)
        dst_mtime = get_iso_timestamp(dst_path)

        if dst_size != src_size:
            raise ValueError(f"Size mismatch for {dst_path}: src={src_size} vs dst={dst_size}")
        if dst_sha != src_sha:
            raise ValueError(f"SHA256 mismatch for {dst_path}: src={src_sha} vs dst={dst_sha}")
        status = "VERIFIED_MATCH"
    else:
        dst_size = src_size
        dst_sha = src_sha
        dst_mtime = get_iso_timestamp(dst_path) if os.path.exists(dst_path) else "DRY_RUN"
        status = "DRY_RUN_PLAN"

    return {
        "src_path": src_path,
        "dst_path": dst_path,
        "size": dst_size,
        "sha256": dst_sha,
        "mtime": dst_mtime,
        "copied": copied,
        "status": status
    }

def run_synchronization(source_dir=DEFAULT_SOURCE_DIR, target_base=DEFAULT_TARGET_BASE, dry_run=False, verify_only=False):
    """
    Execute full synchronization to Google Drive target.
    """
    print("==========================================================================================")
    print(" LIVA BANKING HARNESS — GOOGLE DRIVE DELIVERABLE SYNCHRONIZATION PIPELINE")
    print("==========================================================================================")
    print(f" Source Directory: {source_dir}")
    print(f" Target Base:      {target_base}")
    print(f" Mode:             {'VERIFY_ONLY' if verify_only else ('DRY_RUN' if dry_run else 'ACTIVE_SYNC')}")
    print("------------------------------------------------------------------------------------------\n")

    if not os.path.exists(source_dir):
        print(f"[!] FATAL: Source directory does not exist: {source_dir}")
        sys.exit(1)

    if not os.path.exists(target_base):
        print(f"[!] FATAL: Target Google Drive directory does not exist or drive not mounted: {target_base}")
        sys.exit(1)

    # 1. Define files to sync
    core_files = [
        "index.html",
        "liva_innostart_2026.pptx",
        "liva_innostart_2026.pdf"
    ]

    # Verify that core files exist in source
    missing_source = [f for f in core_files if not os.path.exists(os.path.join(source_dir, f))]
    if missing_source:
        print(f"[!] FATAL: Missing core deliverable(s) in source: {missing_source}")
        sys.exit(1)

    # Assets
    assets_dir = os.path.join(source_dir, "assets")
    asset_files = []
    if os.path.exists(assets_dir):
        for f in sorted(os.listdir(assets_dir)):
            if os.path.isfile(os.path.join(assets_dir, f)):
                asset_files.append(os.path.join("assets", f))

    # Aliased 27-slide files
    aliased_mappings = [
        ("index.html", "LIVA_INNOSTART_2026_PitchDeck_27Slides.html"),
        ("liva_innostart_2026.pptx", "LIVA_INNOSTART_2026_PitchDeck_27Slides.pptx"),
        ("liva_innostart_2026.pdf", "LIVA_INNOSTART_2026_PitchDeck_27Slides.pdf"),
    ]

    # Scripts to sync to Slide_Demo_Day_INNOSTART_2026/
    helper_scripts = [
        "build_pptx.py",
        "export_pdf.py",
        "verify_deck.py",
        "check_alignment.py",
        "sync_drive.py"
    ]

    # Define targets
    demo_day_dir = os.path.join(target_base, DEMO_DAY_SUBFOLDER)
    destinations = [
        ("Root Folder", target_base, False),
        ("Demo Day Subfolder", demo_day_dir, True)
    ]

    sync_results = []
    total_bytes = 0
    errors = []

    print("[*] Planning synchronization actions...\n")

    for dest_name, dest_dir, is_demo_day in destinations:
        print(f"--- Synchronizing to: {dest_name} ({dest_dir}) ---")

        # A. Core deliverables
        for fname in core_files:
            src = os.path.join(source_dir, fname)
            dst = os.path.join(dest_dir, fname)
            try:
                res = sync_single_file(src, dst, dry_run=dry_run, verify_only=verify_only)
                sync_results.append(res)
                total_bytes += res["size"]
                print(f"  [✓] {fname:<30} -> {format_size(res['size']):<9} | SHA256: {res['sha256'][:12]}... | {res['status']}")
            except Exception as e:
                errors.append((dst, str(e)))
                print(f"  [✗] {fname:<30} -> ERROR: {e}")

        # B. Aliased 27-slide files
        for src_name, alias_name in aliased_mappings:
            src = os.path.join(source_dir, src_name)
            dst = os.path.join(dest_dir, alias_name)
            try:
                res = sync_single_file(src, dst, dry_run=dry_run, verify_only=verify_only)
                sync_results.append(res)
                total_bytes += res["size"]
                print(f"  [✓] {alias_name:<30} -> {format_size(res['size']):<9} | SHA256: {res['sha256'][:12]}... | {res['status']}")
            except Exception as e:
                errors.append((dst, str(e)))
                print(f"  [✗] {alias_name:<30} -> ERROR: {e}")

        # C. Photographic assets
        for rel_asset in asset_files:
            src = os.path.join(source_dir, rel_asset)
            dst = os.path.join(dest_dir, rel_asset)
            try:
                res = sync_single_file(src, dst, dry_run=dry_run, verify_only=verify_only)
                sync_results.append(res)
                total_bytes += res["size"]
                print(f"  [✓] {rel_asset:<30} -> {format_size(res['size']):<9} | SHA256: {res['sha256'][:12]}... | {res['status']}")
            except Exception as e:
                errors.append((dst, str(e)))
                print(f"  [✗] {rel_asset:<30} -> ERROR: {e}")

        # D. Synchronize tool scripts to demo_day folder if applicable
        if is_demo_day:
            for sname in helper_scripts:
                src = os.path.join(source_dir, sname)
                dst = os.path.join(dest_dir, sname)
                if os.path.exists(src):
                    try:
                        res = sync_single_file(src, dst, dry_run=dry_run, verify_only=verify_only)
                        sync_results.append(res)
                        total_bytes += res["size"]
                        print(f"  [✓] {sname:<30} -> {format_size(res['size']):<9} | SHA256: {res['sha256'][:12]}... | {res['status']}")
                    except Exception as e:
                        errors.append((dst, str(e)))
                        print(f"  [✗] {sname:<30} -> ERROR: {e}")

        print()

    # -------------------------------------------------------------------------
    # SUMMARY & FINAL VERIFICATION TABLE
    # -------------------------------------------------------------------------
    print("==========================================================================================")
    print(" GOOGLE DRIVE SYNCHRONIZATION AUDIT REPORT")
    print("==========================================================================================")
    print(f"{'Target Path':<65} | {'Size':<10} | {'Modified (ISO)':<20} | Status")
    print("-" * 108)

    for item in sync_results:
        display_path = item["dst_path"]
        # Shorten path prefix if possible for display
        if display_path.startswith(target_base):
            rel = display_path[len(target_base):].lstrip("\\/")
            display_path = f"[Drive]: {rel}"
        if len(display_path) > 63:
            display_path = "..." + display_path[-60:]

        mtime_str = item["mtime"][:19] if len(item["mtime"]) >= 19 else item["mtime"]
        print(f"{display_path:<65} | {format_size(item['size']):<10} | {mtime_str:<20} | {item['status']}")

    print("=" * 108)
    print(f"Total Synchronized / Verified Targets: {len(sync_results)}")
    print(f"Total Aggregated Byte Volume:          {format_size(total_bytes)} ({total_bytes:,} bytes)")
    print(f"Errors Encountered:                    {len(errors)}")

    if errors:
        print("\n[!] Synchronization completed with errors:")
        for target, err in errors:
            print(f"    - {target}: {err}")
        return False

    print("\n[✓] ALL DELIVERABLES SUCCESSFULLY SYNCHRONIZED AND BYTE-VERIFIED TO GOOGLE DRIVE!")
    return True

def main():
    parser = argparse.ArgumentParser(description="Synchronize LIVA Banking Harness deliverables to Google Drive.")
    parser.add_argument("--source-dir", default=DEFAULT_SOURCE_DIR, help="Local source directory of slides")
    parser.add_argument("--target-dir", default=DEFAULT_TARGET_BASE, help="Google Drive destination directory")
    parser.add_argument("--dry-run", action="store_true", help="Simulate synchronization without copying")
    parser.add_argument("--verify-only", action="store_true", help="Verify destination files against source without copying")

    args = parser.parse_args()

    success = run_synchronization(
        source_dir=args.source_dir,
        target_base=args.target_dir,
        dry_run=args.dry_run,
        verify_only=args.verify_only
    )

    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
