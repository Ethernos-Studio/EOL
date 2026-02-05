# EOL 编译器

![License](https://img.shields.io/badge/license-GPL3-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)

EOL (Ethernos Object Language) 是一个简单的面向对象编程语言，支持编译为原生 Windows 可执行文件。

EOL是整个Ethernos编程语言工具链中的里程碑，它是Ethernos发布的所有编程语言中，第一个编译型编程语言。

![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)

## 特性

![Features](https://img.shields.io/badge/features-compiler%20%7C%20runtime-success.svg)

- **完整的编译链**: EOL 源代码 -> LLVM IR -> Windows EXE
- **面向对象**: 支持类、方法、静态成员
- **类型系统**: 支持 int、long、String、void 等基础类型
- **控制流**: 支持 if-else、while 循环
- **运算符**: 支持算术、比较、逻辑、位运算符
- **字符串操作**: 支持字符串字面量和字符串拼接
- **MinGW-w64 支持**: 使用开源工具链，无 MSVC 版权依赖

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/dhjs0000/eol.git
cd eol

# 构建编译器
cargo build --release
```

### 编写第一个程序

创建文件 `hello.eol`:

```eol
public class Hello {
    public static void main() {
        println("Hello, World!");
    }
}
```

### 编译运行

```bash
# 使用 eolc 一站式编译
./target/release/eolc hello.eol hello.exe

# 运行
./hello.exe
```

输出:
```
Hello, World!
```

## 工具链

![Tools](https://img.shields.io/badge/tools-3%20binaries-blue.svg)

本项目提供三个可执行文件：

| 工具 | 功能 | 用法 |
|------|------|------|
| `eolc` | EOL -> EXE (一站式) | `eolc source.eol output.exe` |
| `eolll` | EOL -> LLVM IR | `eolll source.eol output.ll` |
| `ir2exe` | LLVM IR -> EXE | `ir2exe input.ll output.exe` |

## 语言语法

### 变量声明

```eol
int a = 10;
long b = 100L;
String s = "Hello";
```

### 算术运算

```eol
int sum = a + b;
int diff = a - b;
int prod = a * b;
int quot = a / b;
int rem = a % b;
```

### 条件语句

```eol
if (a > b) {
    println("a is greater");
} else if (a == b) {
    println("a equals b");
} else {
    println("a is smaller");
}
```

### 循环

```eol
long i = 0;
while (i < 10) {
    println(i);
    i = i + 1;
}
```

### 字符串拼接

```eol
String name = "EOL";
String message = "Hello, " + name + "!";
println(message);
```

## 示例

### 九九乘法表

```eol
public class Multiplication {
    public static void main() {
        long i = 1;
        while (i <= 9) {
            long j = 1;
            while (j <= i) {
                long product = i * j;
                // 构建并输出每个乘法项，例如 "1×2=2  "
                print(i);
                print("x");
                print(j);
                print("=");
                print(product);
                if (product < 10) {
                    print("  "); // 一位数加两个空格
                } else {
                    print(" "); // 两位数加一个空格
                }
                j = j + 1;
            }
            // 每行结束后换行
            println("");
            i = i + 1;
        }
    }
}
```

编译运行:
```bash
./target/release/eolc examples/multiplication.eol mult.exe
./mult.exe
```

## 项目结构

```
eol/
├── src/                    # 源代码
│   ├── bin/               # 可执行文件
│   │   ├── eolc.rs        # 一站式编译器
│   │   ├── eolll.rs       # EOL -> IR 编译器
│   │   └── ir2exe.rs      # IR -> EXE 编译器
│   ├── lexer/             # 词法分析器
│   ├── parser/            # 语法分析器
│   ├── semantic/          # 语义分析器
│   ├── codegen/           # 代码生成器
│   ├── ast.rs             # AST 定义
│   ├── types.rs           # 类型系统
│   └── error.rs           # 错误处理
├── examples/              # 示例程序
├── lib/mingw64/           # MinGW-w64 库
├── llvm-minimal/          # LLVM 工具链
├── mingw-minimal/         # MinGW 链接器
└── Cargo.toml             # Rust 项目配置
```

## 技术栈

![Tech Stack](https://img.shields.io/badge/tech%20stack-Rust%20%7C%20LLVM%20%7C%20MinGW-success.svg)

- **前端**: Rust 实现的词法分析、语法分析、语义分析
- **中端**: LLVM IR 代码生成
- **后端**: MinGW-w64 工具链（lld 链接器）

## 开发状态

![Status](https://img.shields.io/badge/status-active%20development-green.svg)

- [x] 基础类型系统 (int, long, String, void)
- [x] 变量声明和赋值
- [x] 算术运算符 (+, -, *, /, %)
- [x] 比较运算符 (==, !=, <, <=, >, >=)
- [x] 逻辑运算符 (&&, ||)
- [x] 位运算符 (&, |, ^)
- [x] 条件语句 (if-else)
- [x] 循环语句 (while)
- [x] 字符串拼接
- [x] 完整的编译链

## 许可证

![License](https://img.shields.io/badge/license-GPL3-blue.svg)

本项目采用 GPL3 许可证。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交 Issue 和 Pull Request。

## 致谢

- [LLVM Project](https://llvm.org/)
- [MinGW-w64](https://www.mingw-w64.org/)
- [Rust Programming Language](https://www.rust-lang.org/)
