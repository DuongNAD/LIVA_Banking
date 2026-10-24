<#
.SYNOPSIS
    Khởi tạo Secure HTTPS/WSS Tunnel kết nối Backend LIVA Native Core (cổng 8002) với Cloudflare Pages.
.DESCRIPTION
    Script tự động kiểm tra ngrok hoặc cloudflared, thiết lập quyền Origin cho WebSocket,
    và mở đường hầm mã hóa TLS cho phép Web Client từ xa truy cập an toàn mà không bị chặn Mixed Content.
.PARAMETER Port
    Cổng backend cục bộ cần mở tunnel (Mặc định: 8002 cho LIVA Native Core).
.PARAMETER Tool
    Công cụ tunnel sử dụng ('ngrok' hoặc 'cloudflared'). Mặc định: 'ngrok'.
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/start-backend-tunnel.ps1
#>

[CmdletBinding()]
param(
    [int]$Port = 8002,
    [ValidateSet("ngrok", "cloudflared", "auto")]
    [string]$Tool = "auto"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   LIVA Banking — Secure Backend Tunnel Launcher" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Cấu hình biến môi trường cho LIVA Core cho phép mọi origin tunnel kết nối
$env:LIVA_WS_ALLOWED_ORIGINS = "*"
Write-Host "[✓] Đã cấu hình LIVA_WS_ALLOWED_ORIGINS = '*'" -ForegroundColor Green

# 2. Kiểm tra công cụ tunnel
$ngrokPath = (Get-Command ngrok -ErrorAction SilentlyContinue)?.Source
$cloudflaredPath = (Get-Command cloudflared -ErrorAction SilentlyContinue)?.Source

if ($Tool -eq "auto") {
    if ($ngrokPath) {
        $Tool = "ngrok"
    } elseif ($cloudflaredPath) {
        $Tool = "cloudflared"
    } else {
        Write-Host "[!] Không tìm thấy ngrok hoặc cloudflared trong PATH." -ForegroundColor Yellow
        Write-Host "    Bạn có thể cài đặt nhanh qua winget:" -ForegroundColor Yellow
        Write-Host "    winget install ngrok.ngrok" -ForegroundColor White
        Write-Host "    HOẶC: winget install Cloudflare.cloudflared" -ForegroundColor White
        exit 1
    }
}

Write-Host "[i] Sử dụng công cụ tunnel: $Tool (Trỏ tới cổng cục bộ: $Port)" -ForegroundColor Cyan
Write-Host ""
Write-Host "----------------------------------------------------------" -ForegroundColor Gray
Write-Host " HƯỚNG DẪN KẾT NỐI VỚI CLOUDFLARE PAGES:" -ForegroundColor Yellow
Write-Host " 1. Khi tunnel khởi động thành công, copy URL HTTPS công khai" -ForegroundColor White
Write-Host "    (ví dụ: https://xxxx.ngrok-free.app hoặc https://xxxx.trycloudflare.com)" -ForegroundColor White
Write-Host " 2. Mở trang web LIVA trên Cloudflare Pages" -ForegroundColor White
Write-Host " 3. Bấm vào nút 'Web Demo' ở góc trên cùng bên phải thanh tiêu đề" -ForegroundColor White
Write-Host " 4. Dán URL vừa copy vào và bấm 'Lưu & Kết nối lại'" -ForegroundColor White
Write-Host "----------------------------------------------------------" -ForegroundColor Gray
Write-Host ""

if ($Tool -eq "ngrok") {
    if (-not $ngrokPath) {
        Write-Error "Không tìm thấy ngrok.exe trong hệ thống."
        exit 1
    }
    Write-Host "[>] Đang khởi động ngrok http $Port..." -ForegroundColor Green
    & $ngrokPath http $Port
} elseif ($Tool -eq "cloudflared") {
    if (-not $cloudflaredPath) {
        Write-Error "Không tìm thấy cloudflared.exe trong hệ thống."
        exit 1
    }
    Write-Host "[>] Đang khởi động Cloudflare Quick Tunnel cho port $Port..." -ForegroundColor Green
    & $cloudflaredPath tunnel --url "http://127.0.0.1:$Port"
}
