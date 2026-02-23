#!/bin/bash
# 测试脚本：验证多平台适配IR代码功能
# 对应 ROADMAP.md#L180-185 中的多平台适配IR代码目标

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}=== 测试 ROADMAP.md#L180-185: 多平台适配IR代码 ===${NC}"
echo
echo -e "${CYAN}测试目标：${NC}"
echo "1. 实现EOL程序在不同平台上的可移植性，无需修改代码"
echo "2. 生成的IR代码在不同平台上的兼容性，避免依赖特定平台的指令集"
echo "3. 可选生成参数：-f:XX/--feature:XX开启，-No:XX关闭，-D:XX定义宏，-U:XX取消定义宏"
echo "4. 支持Windows、Linux、macOS等主要操作系统"
echo "5. 支持混淆IR代码，防止被反编译和修改"
echo

# 检查编译器是否存在
CAY_IR_PATH="./target/release/cay-ir"
if [ ! -f "$CAY_IR_PATH" ]; then
    echo -e "${RED}Error: cay-ir not found at $CAY_IR_PATH${NC}"
    echo "Please build the compiler first: cargo build --release"
    exit 1
fi

# 创建输出目录
mkdir -p test_output

# 测试用例
passed=0
failed=0

# 测试函数
run_test() {
    local test_name="$1"
    local options="$2"
    local expected_patterns="$3"
    local not_expected_patterns="$4"
    local description="$5"
    
    echo -e "${CYAN}=== 测试: $test_name ===${NC}"
    echo -e "${YELLOW}描述: $description${NC}"
    echo -e "${YELLOW}命令: $CAY_IR_PATH examples/hello.cay test_output/hello_${test_name}.ll $options${NC}"
    
    # 运行编译器
    if ! $CAY_IR_PATH examples/hello.cay test_output/hello_${test_name}.ll $options >/dev/null 2>&1; then
        echo -e "${RED}❌ 失败: 编译器执行失败${NC}"
        ((failed++))
        return 1
    fi
    
    # 检查生成的文件
    if [ ! -f "test_output/hello_${test_name}.ll" ]; then
        echo -e "${RED}❌ 失败: IR文件未生成${NC}"
        ((failed++))
        return 1
    fi
    
    # 检查预期模式
    local success=true
    if [ -n "$expected_patterns" ]; then
        IFS=',' read -ra patterns <<< "$expected_patterns"
        for pattern in "${patterns[@]}"; do
            # 使用更健壮的方式搜索，忽略 null 字节，使用扩展正则表达式
            if ! tr -d '\0' < test_output/hello_${test_name}.ll | grep -q -E "$pattern"; then
                echo -e "${RED}❌ 失败: 未找到预期模式 '$pattern'${NC}"
                # 显示文件内容的相关部分，帮助调试
                echo -e "${YELLOW}文件内容预览:${NC}"
                head -30 test_output/hello_${test_name}.ll
                success=false
            else
                echo -e "${GREEN}✓ 找到预期模式 '$pattern'${NC}"
            fi
        done
    fi
    
    # 检查不应存在的模式
    if [ "$success" = true ] && [ -n "$not_expected_patterns" ]; then
        IFS=',' read -ra patterns <<< "$not_expected_patterns"
        for pattern in "${patterns[@]}"; do
            # 使用更健壮的方式搜索，忽略 null 字节
            if tr -d '\0' < test_output/hello_${test_name}.ll | grep -q "$pattern"; then
                echo -e "${RED}❌ 失败: 找到不应存在的模式 '$pattern'${NC}"
                success=false
            else
                echo -e "${GREEN}✓ 未找到不应存在的模式 '$pattern'${NC}"
            fi
        done
    fi
    
    if [ "$success" = true ]; then
        echo -e "${GREEN}✅ 成功${NC}"
        ((passed++))
    else
        ((failed++))
    fi
    
    echo
}

# 测试1: Linux平台基础测试
run_test "linux_basic" "--target linux" "target triple = \"x86_64-unknown-linux-gnu\"" "SetConsoleOutputCP" "测试Linux平台基础IR生成"

# 测试2: Windows平台基础测试
run_test "windows_basic" "--target windows" "target triple = \"x86_64-w64-mingw32\"" "setlocale" "测试Windows平台基础IR生成"

# 测试3: macOS平台基础测试
run_test "macos_basic" "--target macos" "target triple = \"x86_64-apple-darwin\"" "SetConsoleOutputCP" "测试macOS平台基础IR生成"

# 测试4: Linux平台启用console_utf8特性
run_test "linux_console_utf8" "--target linux -f:console_utf8" "setlocale,@.str.locale" "" "测试Linux平台启用console_utf8特性"

# 测试5: Windows平台启用console_utf8特性
run_test "windows_console_utf8" "--target windows -f:console_utf8" "declare dllimport void @SetConsoleOutputCP" "" "测试Windows平台启用console_utf8特性"

# 测试6: 禁用console_utf8特性
run_test "feature_disable" "--target linux -No:console_utf8" "" "setlocale" "测试禁用console_utf8特性"

# 测试7: 定义宏
run_test "macro_define" "--target linux -D:TEST_MACRO -D:VERSION=123" "TEST_MACRO,VERSION=123" "" "测试定义宏"

# 测试8: 取消定义宏
run_test "macro_undef" "--target linux -D:TEST_MACRO -U:TEST_MACRO" "" "TEST_MACRO" "测试取消定义宏"

# 测试9: IR代码混淆
run_test "obfuscate" "--target linux --obfuscate" "__obf_" "" "测试IR代码混淆"

# 测试10: 组合选项测试
run_test "combined_options" "--target linux -f:console_utf8 -D:DEBUG -D:RELEASE=1" "setlocale,DEBUG,RELEASE=1" "" "测试组合选项"

# 输出测试结果
echo -e "${CYAN}=== 测试结果汇总 ===${NC}"
echo -e "通过: ${GREEN}$passed${NC}"
echo -e "失败: ${RED}$failed${NC}"
echo -e "总计: $((passed + failed))"

if [ "$failed" -eq 0 ]; then
    echo -e "${GREEN}🎉 所有测试通过！多平台适配IR代码功能正常工作。${NC}"
else
    echo -e "${RED}❌ 部分测试失败，请检查以上输出。${NC}"
    exit 1
fi

# 清理测试文件（可选）
# rm -rf test_output

echo -e "${CYAN}=== 测试完成 ===${NC}"