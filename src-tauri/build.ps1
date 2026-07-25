# Tauri 构建脚本
# 使用 MSVC 工具链，产出单文件 exe（静态链接 WebView2Loader）
# 前置条件：Visual Studio Build Tools 2022（含 MSVC C++ 工作负载）
# 用法: .\build.ps1 [dev|build|release]

param(
    [Parameter(Position=0)]
    [ValidateSet("dev", "build", "release")]
    [string]$Mode = "build"
)

$ErrorActionPreference = "Stop"

# 1. 确认 MSVC 工具链
$toolchain = rustup show active-toolchain 2>&1
if ($toolchain -notmatch "msvc") {
    Write-Host "⚠ 当前工具链: $toolchain" -ForegroundColor Yellow
    Write-Host "  项目需要 MSVC 工具链（静态链接 WebView2Loader），正在自动切换..." -ForegroundColor Yellow
    rustup default stable-x86_64-pc-windows-msvc 2>&1 | Out-Null
    $toolchain = rustup show active-toolchain 2>&1
}
Write-Host "✓ Rust 工具链: $toolchain" -ForegroundColor Green

# 2. 确认 VS Build Tools 可用
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath 2>&1
    if ($vsPath) {
        Write-Host "✓ VS Build Tools: $vsPath" -ForegroundColor Green
    }
} else {
    Write-Host "⚠ 未找到 vswhere，请确认 Visual Studio Build Tools 已安装" -ForegroundColor Yellow
}

# 3. 执行构建
Set-Location $PSScriptRoot

switch ($Mode) {
    "dev" {
        Write-Host "→ 启动开发模式..." -ForegroundColor Cyan
        npx tauri dev 2>&1
    }
    "build" {
        Write-Host "→ 编译 debug 版本..." -ForegroundColor Cyan
        cargo build 2>&1
    }
    "release" {
        Write-Host "→ 编译 release 版本..." -ForegroundColor Cyan
        npx tauri build 2>&1
    }
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 构建成功!" -ForegroundColor Green
} else {
    Write-Host "✗ 构建失败 (exit $LASTEXITCODE)" -ForegroundColor Red
    exit $LASTEXITCODE
}
