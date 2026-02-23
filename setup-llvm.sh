#!/bin/bash

set -euo pipefail

URL="https://github.com/cavvy-lang/Cavvy-src-Assets/releases/download/llvm-minimal/bin-linux/bin-linux.zip"
TARGET_DIR="./llvm-minimal/bin/bin-linux"
TEMP_ZIP=".temp-bin-linux-$$.zip"
TEMP_DIR=".temp-extract-$$"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔧 Setting up LLVM minimal binaries...${NC}"
echo "   URL: $URL"
echo "   Target: $TARGET_DIR"

# 检查依赖
check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}❌ Error: $1 is required but not installed.${NC}"
        if [ "$1" == "unzip" ]; then
            echo "   Install: sudo apt-get install unzip  (Debian/Ubuntu)"
            echo "            sudo yum install unzip      (RHEL/CentOS)"
            echo "            brew install unzip          (macOS)"
        fi
        exit 1
    fi
}

check_dependency curl
check_dependency unzip

# 创建父目录
PARENT_DIR=$(dirname "$TARGET_DIR")
mkdir -p "$PARENT_DIR"

# 如果目标已存在，询问是否覆盖
if [ -d "$TARGET_DIR" ]; then
    echo -e "${YELLOW}⚠️  Target directory already exists. Overwrite? (y/N)${NC}"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Aborted.${NC}"
        exit 1
    fi
    rm -rf "$TARGET_DIR"
fi

# 清理函数（在退出时调用）
cleanup() {
    if [ -f "$TEMP_ZIP" ]; then
        rm -f "$TEMP_ZIP"
        echo -e "🧹 Cleaned up temporary zip"
    fi
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
        echo -e "🧹 Cleaned up temporary directory"
    fi
}
trap cleanup EXIT

# 下载
echo -e "⬇️  Downloading..."
if ! curl -fsSL --progress-bar -o "$TEMP_ZIP" "$URL"; then
    echo -e "${RED}❌ Download failed${NC}"
    exit 1
fi

# 验证文件
if [ ! -f "$TEMP_ZIP" ] || [ ! -s "$TEMP_ZIP" ]; then
    echo -e "${RED}❌ Downloaded file is empty or missing${NC}"
    exit 1
fi

FILESIZE=$(du -h "$TEMP_ZIP" | cut -f1)
echo -e "✅ Downloaded: $FILESIZE"

# 解压
echo -e "📦 Extracting..."
mkdir -p "$TEMP_DIR"

if ! unzip -q "$TEMP_ZIP" -d "$TEMP_DIR"; then
    echo -e "${RED}❌ Extraction failed${NC}"
    exit 1
fi

# 处理解压后的内容：
# 如果zip内部有bin-linux文件夹，直接使用；否则将解压内容视为bin-linux
if [ -d "$TEMP_DIR/bin-linux" ] && [ "$(ls -A "$TEMP_DIR" | wc -l)" -eq 1 ]; then
    # zip内已有bin-linux文件夹
    mv "$TEMP_DIR/bin-linux" "$TARGET_DIR"
else
    # zip内是散文件，创建bin-linux文件夹并移入
    mkdir -p "$TARGET_DIR"
    mv "$TEMP_DIR"/* "$TARGET_DIR/"
fi

echo -e "${GREEN}✅ Successfully installed to: $(realpath "$TARGET_DIR")${NC}"

# 列出内容
echo -e "${CYAN}📂 Contents:${NC}"
ls -la "$TARGET_DIR"