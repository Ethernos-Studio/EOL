# Cavvy 快速入门指南

## 1. 简介

Cavvy (Cay) 是一个静态类型的面向对象编程语言，编译为原生机器码，无运行时依赖，无 VM，无 GC。

**核心特性**：
- 🚀 原生性能
- 🛡️ 内存安全
- ☕ Java 风格语法
- 🔧 完整工具链
- 🌉 FFI 支持
- 📦 字节码系统

## 2. 安装

### 2.1 从源码构建

```bash
# 克隆仓库
git clone https://github.com/Ethernos-Studio/Cavvy.git
cd Cavvy

# 构建编译器（Release 模式）
cargo build --release

# 运行测试
cargo test --release
```

### 2.2 系统要求

- **Windows**：Windows 10/11 x64
- **Linux**：x86_64 Linux 发行版
- **依赖**：LLVM 17.0+, MinGW-w64 13.2+ (Windows)

## 3. 第一个程序

### 3.1 创建源文件

创建一个名为 `hello.cay` 的文件，内容如下：

```cay
public class Hello {
    public static void main() {
        println("Hello, World!");
    }
}
```

或者使用顶层 main 函数（0.4.3+）：

```cay
public int main() {
    println("Hello from top-level main!");
    return 0;
}
```

### 3.2 编译运行

#### Windows

```bash
# 编译
./target/release/cayc hello.cay hello.exe

# 运行
./hello.exe
```

#### Linux

```bash
# 编译
./target/release/cayc hello.cay hello

# 运行
./hello
```

## 4. 基本语法

### 4.1 变量声明

```cay
// 基本类型
int a = 10;
long b = 100L;
float f = 3.14f;
double d = 3.14159;
boolean flag = true;
char c = 'A';
String s = "Hello, Cavvy!";

// 自动类型推断
auto x = 42;        // int
auto pi = 3.14;     // double
auto msg = "hi";    // String
```

### 4.2 控制流

```cay
// if-else
if (a > b) {
    println("a is greater");
} else if (a == b) {
    println("a equals b");
} else {
    println("a is smaller");
}

// 循环
for (int i = 0; i < 10; i++) {
    println(i);
}

int j = 0;
while (j < 10) {
    println(j);
    j++;
}

// do-while
int k = 0;
do {
    println(k);
    k++;
} while (k < 5);
```

### 4.3 数组

```cay
// 一维数组
int[] arr = new int[5];
int[] initArr = {1, 2, 3, 4, 5};

// 数组长度
int len = arr.length;

// 数组访问
arr[0] = 100;
int val = arr[0];
```

## 5. 面向对象编程

### 5.1 类定义

```cay
public class Animal {
    protected String name;
    
    public Animal(String name) {
        this.name = name;
    }
    
    public void speak() {
        println("Some sound");
    }
}

public class Dog extends Animal {
    public Dog(String name) {
        super(name);
    }
    
    @Override
    public void speak() {
        println(name + " says: Woof!");
    }
}
```

### 5.2 抽象类和接口

```cay
public abstract class Shape {
    public abstract double area();
}

public interface Drawable {
    void draw();
}

public class Circle extends Shape implements Drawable {
    private double radius;
    
    public Circle(double radius) {
        this.radius = radius;
    }
    
    @Override
    public double area() {
        return 3.14159 * radius * radius;
    }
    
    @Override
    public void draw() {
        println("Drawing circle with radius " + radius);
    }
}
```

## 6. 方法重载和可变参数

```cay
public class Calculator {
    // 方法重载
    public static int add(int a, int b) {
        return a + b;
    }
    
    public static double add(double a, double b) {
        return a + b;
    }
    
    // 可变参数
    public static int sum(int... numbers) {
        int total = 0;
        for (int i = 0; i < numbers.length; i++) {
            total = total + numbers[i];
        }
        return total;
    }
}
```

## 7. Lambda 表达式

```cay
// Lambda 表达式
var add = (int a, int b) -> { return a + b; };
int result = add(3, 4);

// 简写形式
var multiply = (int a, int b) -> a * b;

// 方法引用
var ref = Calculator::add;
```

## 8. FFI - 调用 C 函数

```cay
// 声明外部 C 函数
extern {
    int abs(int x);
    double sqrt(double x);
    int strlen(String s);
}

public class MathExample {
    public static void main() {
        int result = abs(-42);
        double root = sqrt(2.0);
        println("Abs: " + result);
        println("Sqrt: " + root);
    }
}
```

## 9. 编译选项

### 9.1 优化级别

```bash
# 最高优化
cayc -O3 hello.cay hello.exe

# 无优化（调试）
cayc -O0 hello.cay hello.exe
```

### 9.2 字节码混淆

```bash
# 深度混淆
cayc --obfuscate --obfuscate-level deep hello.cay hello.exe
```

### 9.3 链接库

```bash
# 链接数学库
cayc -lm hello.cay hello.exe
```

## 10. 工具链

Cavvy 提供六个可执行文件：

| 工具 | 功能 | 用法 |
|------|------|------|
| `cayc` | 一站式编译器 | `cayc source.cay output.exe` |
| `cay-ir` | 生成 LLVM IR | `cay-ir source.cay output.ll` |
| `ir2exe` | IR 转可执行文件 | `ir2exe input.ll output.exe` |
| `cay-check` | 语法检查 | `cay-check source.cay` |
| `cay-run` | 直接运行 | `cay-run source.cay` |
| `cay-bcgen` | 生成字节码 | `cay-bcgen source.cay output.caybc` |

## 11. 常见问题

### 11.1 编译错误

**问题**：编译失败，提示语法错误
**解决方案**：检查代码语法，确保所有括号、分号等符号正确配对

**问题**：链接错误，找不到符号
**解决方案**：确保正确链接必要的库，如 `-lm` 用于数学函数

### 11.2 运行时错误

**问题**：程序崩溃或行为异常
**解决方案**：检查内存管理，确保对象正确初始化和释放

**问题**：输出乱码
**解决方案**：Windows 下可使用 `-features console_utf8` 选项

## 12. 示例程序

### 12.1 九九乘法表

```cay
public class Multiplication {
    public static void main() {
        for (int i = 1; i <= 9; i++) {
            for (int j = 1; j <= i; j++) {
                print(j + "x" + i + "=" + (i*j) + "\t");
            }
            println("");
        }
    }
}
```

### 12.2 斐波那契数列

```cay
public class Fibonacci {
    // 迭代实现
    public static long fibIterative(int n) {
        if (n <= 1) return n;
        long a = 0, b = 1;
        for (int i = 2; i <= n; i++) {
            long temp = a + b;
            a = b;
            b = temp;
        }
        return b;
    }
    
    public static void main() {
        for (int i = 0; i < 20; i++) {
            println("fib(" + i + ") = " + fibIterative(i));
        }
    }
}
```

### 12.3 冒泡排序

```cay
public class Sorting {
    public static void bubbleSort(int[] arr) {
        int n = arr.length;
        for (int i = 0; i < n - 1; i++) {
            for (int j = 0; j < n - i - 1; j++) {
                if (arr[j] > arr[j + 1]) {
                    int temp = arr[j];
                    arr[j] = arr[j + 1];
                    arr[j + 1] = temp;
                }
            }
        }
    }
    
    public static void main() {
        int[] numbers = {64, 34, 25, 12, 22, 11, 90};
        bubbleSort(numbers);
        
        print("Sorted: ");
        for (int i = 0; i < numbers.length; i++) {
            print(numbers[i] + " ");
        }
        println("");
    }
}
```

## 13. 下一步

- 查看 [Cavvy 语言参考](file:///workspace/Cavvy_Language_Reference.md) 了解完整语法
- 探索 [examples](file:///workspace/examples) 目录中的示例程序
- 阅读 [CODE_WIKI.md](file:///workspace/CODE_WIKI.md) 了解编译器内部工作原理
- 查看 [ROADMAP.md](file:///workspace/ROADMAP.md) 了解未来开发计划

---

**Cavvy - 编译未来**