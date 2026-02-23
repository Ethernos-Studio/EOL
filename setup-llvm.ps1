#!/usr/bin/env pwsh
#Requires -Version 5.0

param(
    [string]$Url = "https://github.com/cavvy-lang/Cavvy-src-Assets/releases/download/llvm-minimal/bin-linux/bin-linux.zip",
    [string]$TargetDir = "./llvm-minimal/bin/bin-linux",
    [string]$TempZip = "./.temp-bin-linux.zip"
)

$ErrorActionPreference = "Stop"

Write-Host "🔧 Setting up LLVM minimal binaries..." -ForegroundColor Cyan
Write-Host "   URL: $Url"
Write-Host "   Target: $TargetDir"

# 检查并创建目标目录的父目录
$parentDir = Split-Path -Parent $TargetDir
if (!(Test-Path $parentDir)) {
    Write-Host "📁 Creating directory: $parentDir"
    New-Item -ItemType Directory -Force -Path $parentDir | Out-Null
}

# 如果目标已存在，询问是否覆盖
if (Test-Path $TargetDir) {
    $response = Read-Host "⚠️  Target directory already exists. Overwrite? (y/N)"
    if ($response -ne 'y' -and $response -ne 'Y') {
        Write-Host "❌ Aborted." -ForegroundColor Red
        exit 1
    }
    Remove-Item -Recurse -Force $TargetDir
}

try {
    # 下载文件（禁用进度条以提高速度，然后恢复）
    Write-Host "⬇️  Downloading..."
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $Url -OutFile $TempZip -UseBasicParsing
    $ProgressPreference = 'Continue'

    if (!(Test-Path $TempZip)) {
        throw "Download failed: File not created"
    }

    # 获取文件大小
    $fileSize = (Get-Item $TempZip).Length / 1MB
    Write-Host "✅ Downloaded: $([math]::Round($fileSize, 2)) MB"

    # 解压到临时目录，然后移动（避免zip内路径问题）
    $tempExtractDir = "./.temp-extract-$(Get-Random)"
    Write-Host "📦 Extracting..."
    Expand-Archive -Path $TempZip -DestinationPath $tempExtractDir -Force

    # 处理解压后的内容：
    # 如果zip内部有bin-linux文件夹，直接使用；否则将解压内容视为bin-linux
    $extractedContent = Get-ChildItem $tempExtractDir
    if ($extractedContent.Count -eq 1 -and $extractedContent[0].PSIsContainer -and $extractedContent[0].Name -eq "bin-linux") {
        # zip内已有bin-linux文件夹，移动到目标位置
        Move-Item $extractedContent[0].FullName $TargetDir
    } else {
        # zip内是散文件，创建bin-linux文件夹并移入
        New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
        Move-Item "$tempExtractDir\*" $TargetDir
    }

    Write-Host "✅ Successfully installed to: $(Resolve-Path $TargetDir)" -ForegroundColor Green

} catch {
    Write-Host "❌ Error: $_" -ForegroundColor Red
    exit 1
} finally {
    # 清理临时文件
    if (Test-Path $TempZip) {
        Remove-Item -Force $TempZip
        Write-Host "🧹 Cleaned up temporary files"
    }
    if (Test-Path $tempExtractDir) {
        Remove-Item -Recurse -Force $tempExtractDir -ErrorAction SilentlyContinue
    }
}