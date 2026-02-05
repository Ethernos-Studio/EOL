# EOL Compiler

![License](https://img.shields.io/badge/license-GPL3-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)

EOL (Ethernos Object Language) is a simple object-oriented programming language that compiles to native Windows executables.

EOL is a milestone in the Ethernos programming language toolchain, being the first compiled programming language released by Ethernos.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)

## Features

![Features](https://img.shields.io/badge/features-compiler%20%7C%20runtime-success.svg)

- **Complete compilation chain**: EOL source code -> LLVM IR -> Windows EXE
- **Object-oriented**: Supports classes, methods, static members
- **Type system**: Supports int, long, String, void and other basic types
- **Control flow**: Supports if-else, while loops
- **Operators**: Supports arithmetic, comparison, logical, and bitwise operators
- **String operations**: Supports string literals and string concatenation
- **MinGW-w64 support**: Uses open-source toolchain, no MSVC copyright dependencies

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/eol.git
cd eol

# Build the compiler
cargo build --release
```

### Write Your First Program

Create a file `hello.eol`:

```eol
public class Hello {
    public static void main() {
        println("Hello, World!");
    }
}
```

### Compile and Run

```bash
# Use eolc for one-step compilation
./target/release/eolc hello.eol hello.exe

# Run
./hello.exe
```

Output:
```
Hello, World!
```

## Toolchain

![Tools](https://img.shields.io/badge/tools-3%20binaries-blue.svg)

This project provides three executables:

| Tool | Function | Usage |
|------|----------|-------|
| `eolc` | EOL -> EXE (one-step) | `eolc source.eol output.exe` |
| `eolll` | EOL -> LLVM IR | `eolll source.eol output.ll` |
| `ir2exe` | LLVM IR -> EXE | `ir2exe input.ll output.exe` |

## Language Syntax

### Variable Declaration

```eol
int a = 10;
long b = 100L;
String s = "Hello";
```

### Arithmetic Operations

```eol
int sum = a + b;
int diff = a - b;
int prod = a * b;
int quot = a / b;
int rem = a % b;
```

### Conditional Statements

```eol
if (a > b) {
    println("a is greater");
} else if (a == b) {
    println("a equals b");
} else {
    println("a is smaller");
}
```

### Loops

```eol
long i = 0;
while (i < 10) {
    println(i);
    i = i + 1;
}
```

### String Concatenation

```eol
String name = "EOL";
String message = "Hello, " + name + "!";
println(message);
```

## Examples

### Multiplication Table

```eol
public class Multiplicatio {
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

Compile and run:
```bash
./target/release/eolc examples/multiplication.eol mult.exe
./mult.exe
```

## Project Structure

```
eol/
├── src/                    # Source code
│   ├── bin/               # Executables
│   │   ├── eolc.rs        # One-step compiler
│   │   ├── eolll.rs       # EOL -> IR compiler
│   │   └── ir2exe.rs      # IR -> EXE compiler
│   ├── lexer/             # Lexical analyzer
│   ├── parser/            # Syntax analyzer
│   ├── semantic/          # Semantic analyzer
│   ├── codegen/           # Code generator
│   ├── ast.rs             # AST definitions
│   ├── types.rs           # Type system
│   └── error.rs           # Error handling
├── examples/              # Example programs
├── lib/mingw64/           # MinGW-w64 libraries
├── llvm-minimal/          # LLVM toolchain
├── mingw-minimal/         # MinGW linker
└── Cargo.toml             # Rust project configuration
```

## Tech Stack

![Tech Stack](https://img.shields.io/badge/tech%20stack-Rust%20%7C%20LLVM%20%7C%20MinGW-success.svg)

- **Frontend**: Rust-based lexical analysis, syntax analysis, semantic analysis
- **Middle-end**: LLVM IR code generation
- **Backend**: MinGW-w64 toolchain (lld linker)

## Development Status

![Status](https://img.shields.io/badge/status-active%20development-green.svg)

- [x] Basic type system (int, long, String, void)
- [x] Variable declaration and assignment
- [x] Arithmetic operators (+, -, *, /, %)
- [x] Comparison operators (==, !=, <, <=, >, >=)
- [x] Logical operators (&&, ||)
- [x] Bitwise operators (&, |, ^)
- [x] Conditional statements (if-else)
- [x] Loop statements (while)
- [x] String concatenation
- [x] Complete compilation chain

## License

![License](https://img.shields.io/badge/license-GPL3-blue.svg)

This project is licensed under the GPL3 License. See the [LICENSE](LICENSE) file for details.

## Contributing

Issues and Pull Requests are welcome.

## Acknowledgments

- [LLVM Project](https://llvm.org/)
- [MinGW-w64](https://www.mingw-w64.org/)
- [Rust Programming Language](https://www.rust-lang.org/)
