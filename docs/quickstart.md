# Cavvy 快速开始指南

欢迎来到 Cavvy 编程语言！本指南将帮助您在 5 分钟内编写并运行第一个 Cavvy 程序。

## 目录

- [安装 Cavvy](#安装-cavvy)
- [Hello World](#hello-world)
- [基础语法概览](#基础语法概览)
- [编译与运行](#编译与运行)
- [下一步](#下一步)

---

## 安装 Cavvy

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/cavvy-lang/cavvy.git
cd cavvy

# 构建编译器（Release 模式）
cargo build --release

# 验证安装
./target/release/cayc --version
```

### 环境变量配置（可选）

将编译器添加到 PATH：

```bash
# Windows PowerShell
$env:PATH += ";$(pwd)\target\release"

# Linux/macOS
export PATH="$PATH:$(pwd)/target/release"
```

---

## Hello World

创建您的第一个 Cavvy 程序：

### 方式一：类内 main 方法（传统 Java 风格）

创建文件 `hello.cay`：

```cay
public class Hello {
    public static void main() {
        println("Hello, World!");
    }
}
```

### 方式二：顶层 main 函数（推荐，0.4.3+）

创建文件 `hello.cay`：

```cay
public int main() {
    println("Hello from top-level main!");
    return 0;
}
```

### 方式三：带命令行参数

```cay
public int main(String[] args) {
    println("Arguments count: " + args.length);
    for (int i = 0; i < args.length; i++) {
        println("Arg " + i + ": " + args[i]);
    }
    return 0;
}
```

---

## 基础语法概览

### 变量声明

```cay
public int main() {
    // 传统方式
    int x = 10;
    String name = "Cavvy";
    
    // 现代方式
    var y = 20;
    let greeting = "Hi";
    
    // 自动类型推断
    auto count = 100;
    auto pi = 3.14159;
    auto message = "Hello";
    
    // 不可变变量
    final int MAX_SIZE = 100;
    final int MIN_VALUE = 0;
    
    println("x = " + x);
    return 0;
}
```

### 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `int` | 32位有符号整数 | `int x = 42;` |
| `long` | 64位有符号整数 | `long l = 100L;` |
| `float` | 32位浮点数 | `float f = 3.14f;` |
| `double` | 64位浮点数 | `double d = 3.14159;` |
| `char` | 16位 Unicode 字符 | `char c = 'A';` |
| `boolean` | 布尔值 | `boolean flag = true;` |
| `String` | 字符串 | `String s = "text";` |
| `void` | 无返回值 | 用于函数返回类型 |

### 控制流

```cay
public int main() {
    // if-else
    int score = 85;
    if (score >= 90) {
        println("A");
    } else if (score >= 80) {
        println("B");
    } else {
        println("C");
    }
    
    // for 循环
    for (int i = 0; i < 5; i++) {
        println("Iteration: " + i);
    }
    
    // while 循环
    int count = 0;
    while (count < 3) {
        println("Count: " + count);
        count++;
    }
    
    // do-while 循环
    int num = 0;
    do {
        println("Number: " + num);
        num++;
    } while (num < 3);
    
    // switch 语句
    int day = 3;
    switch (day) {
        case 1:
            println("Monday");
            break;
        case 2:
            println("Tuesday");
            break;
        case 3:
            println("Wednesday");
            break;
        default:
            println("Other day");
            break;
    }
    
    return 0;
}
```

### 数组

```cay
public int main() {
    // 一维数组
    int[] numbers = new int[5];
    numbers[0] = 10;
    numbers[1] = 20;
    
    // 数组初始化
    int[] values = {1, 2, 3, 4, 5};
    
    // 多维数组
    int[][] matrix = new int[3][3];
    matrix[0][0] = 1;
    matrix[1][1] = 2;
    matrix[2][2] = 3;
    
    // 数组长度
    println("Array length: " + values.length);
    
    // 遍历数组
    for (int i = 0; i < values.length; i++) {
        println("values[" + i + "] = " + values[i]);
    }
    
    return 0;
}
```

### 类和对象

```cay
public class Person {
    private String name;
    private int age;

    // 构造函数
    public Person(String name, int age) {
        this.name = name;
        this.age = age;
    }

    // 方法
    public void greet() {
        println("Hello, I'm " + name + ", " + age + " years old.");
    }

    // Getter
    public String getName() {
        return name;
    }
}

public int main() {
    Person person = new Person("Alice", 25);
    person.greet();
    println("Name: " + person.getName());
    return 0;
}
```

### 继承

```cay
public class Animal {
    protected String name;

    public Animal(String name) {
        this.name = name;
    }

    public void speak() {
        println(name + " makes a sound");
    }
}

public class Dog : Animal {
    public Dog(String name) {
        super(name);
    }

    @Override
    public void speak() {
        println(name + " barks");
    }
}

public int main() {
    Dog dog = new Dog("Buddy");
    dog.speak();  // 输出: Buddy barks
    return 0;
}
```

---

## 编译与运行

### 使用 cayc（一站式编译）

```bash
# 编译为可执行文件
./target/release/cayc hello.cay hello.exe

# 运行
./hello.exe
```

### 分步编译

```bash
# 1. 编译为 LLVM IR
./target/release/cay-ir hello.cay hello.ll

# 2. 编译 IR 为可执行文件
./target/release/ir2exe hello.ll hello.exe

# 运行
./hello.exe
```

### 直接运行（无需生成 EXE）

```bash
./target/release/cay-run hello.cay
```

### 编译选项

```bash
# 优化级别
./target/release/cayc hello.cay hello.exe -O3

# 保留中间文件
./target/release/cayc hello.cay hello.exe --keep-ir
```

---

## 下一步

- **语言文档** - 深入了解 Cavvy 的所有特性：[language-guide.md](language-guide.md)
- **FFI 指南** - 学习如何调用 C 函数：[ffi-guide.md](ffi-guide.md)
- **语法参考** - 完整的语法规范：[syntax-reference.md](syntax-reference.md)
- **示例代码** - 查看 `examples/` 目录下的更多示例

---

## 常见问题

### Q: Cavvy 与 Java 有什么区别？

A: Cavvy 语法类似 Java，但：
- 编译为原生机器码，无 JVM
- 显式内存管理，无 GC
- 支持顶层函数
- 支持 `auto` 类型推断

### Q: 如何调试 Cavvy 程序？

A: 当前版本使用 `println()` 进行简单调试。

### Q: Cavvy 支持哪些平台？

A: 目前支持 Windows 和 Linux。macOS 支持正在开发中。

---

**祝您编程愉快！**
