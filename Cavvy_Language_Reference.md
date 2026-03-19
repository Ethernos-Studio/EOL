# Cavvy 编程语言完整参考手册

**版本**: 0.4.8.2
**构建号**: 26
**最后更新**: 2026-03-18
**状态**: 活跃开发中

---

## 目录

1. [概述](#1-概述)
2. [快速开始](#2-快速开始)
3. [词法结构](#3-词法结构)
4. [类型系统](#4-类型系统)
5. [变量与常量](#5-变量与常量)
6. [运算符](#6-运算符)
7. [控制流](#7-控制流)
8. [数组](#8-数组)
9. [字符串](#9-字符串)
10. [类与面向对象](#10-类与面向对象)
11. [方法](#11-方法)
12. [Lambda表达式与方法引用](#12-lambda表达式与方法引用)
13. [预处理器](#13-预处理器)
14. [内置函数](#14-内置函数)
15. [编译器工具链](#15-编译器工具链)
16. [EBNF语法规范](#16-ebnf语法规范)
17. [编译器架构](#17-编译器架构)
18. [开发路线图](#18-开发路线图)
19. [附录](#19-附录)

---

## 1. 概述

### 1.1 什么是Cavvy

Cavvy (文件扩展名 `.cay`) 是一个静态类型的面向对象编程语言，由 Ethernos Studio 开发。它是 Ethernos 编程语言工具链中的里程碑，是 Ethernos 发布的所有编程语言中第一个编译型编程语言。

### 1.2 设计哲学

- **编译为原生机器码**: 无运行时依赖，无VM，无GC
- **Java语法风格**: 熟悉的面向对象语法
- **C++级别性能**: 直接编译为LLVM IR，再生成原生可执行文件
- **显式内存管理**: 无垃圾回收，内存控制精确可预测
- **零成本抽象**: 高级特性不牺牲运行时性能

### 1.3 核心特性

| 特性类别 | 支持内容 |
|---------|---------|
| **类型系统** | int, long, float, double, boolean, char, String, void, 数组 |
| **控制流** | if-else, while, for, do-while, switch, break, continue |
| **运算符** | 算术、比较、逻辑、位运算、自增自减、复合赋值 |
| **面向对象** | 类、方法、静态成员、方法重载、可变参数 |
| **字符串** | 字面量、拼接、方法(length, substring, indexOf, replace, charAt) |
| **高级特性** | Lambda表达式、方法引用、类型转换 |
| **编译链** | Cavvy → LLVM IR → Windows EXE |

### 1.4 Hello World

```cay
public class Hello {
    public static void main() {
        println("Hello, World!");
    }
}
```

编译运行:
```bash
cayc hello.cay hello.exe
./hello.exe
```

---

## 2. 快速开始

### 2.1 安装

#### 系统要求
- **操作系统**: Windows 10/11 (目前主要支持Windows)
- **构建工具**: Rust 1.70+ (用于编译编译器)
- **依赖库**: MinGW-w64 (已包含在项目中)

#### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/Ethernos-Studio/Cavvy.git
cd eol

# 构建编译器
cargo build --release
```

构建完成后，可执行文件位于 `target/release/` 目录。

### 2.2 工具链概览

Cavvy提供四个主要工具:

| 工具 | 功能 | 典型用法 |
|------|------|---------|
| `cayc` | 一站式编译器 (Cavvy → EXE) | `cayc source.cay output.exe` |
| `cay-ir` | 生成LLVM IR | `cay-ir source.cay output.ll` |
| `ir2exe` | IR转可执行文件 | `ir2exe input.ll output.exe` |
| `cay-check` | 语法检查 | `cay-check source.cay` |

### 2.3 第一个程序详解

```cay
// 定义一个公共类
public class Hello {
    // main方法是程序入口点
    // public static void 是必需的修饰符
    public static void main() {
        // println是内置函数，输出字符串并换行
        println("Hello, World!");
    }
}
```

**关键点**:
- 每个程序必须有一个包含 `main` 方法的类
- `main` 方法必须是 `public static void` 且不带参数
- 可以使用 `@main` 注解显式指定主类(多类情况下)

---

## 3. 词法结构

### 3.1 源文件编码

Cavvy源文件使用 **UTF-8** 编码，支持Unicode标识符。

### 3.2 空白字符

- 空格 (` `) 和制表符 (`\t`) 用于分隔标记
- 换行符 (`\n` 或 `\r\n`) 用于行号跟踪
- 空白字符在词法分析阶段被忽略(除换行跟踪外)

### 3.3 注释

```cay
// 单行注释 - 从 // 到行尾

/*
 * 多行注释
 * 可以跨越多行
 */

/* 多行注释也可以在一行内 */
```

### 3.4 关键字

Cavvy保留以下关键字，不能用作标识符:

| 类别 | 关键字 |
|------|--------|
| **访问控制** | `public`, `private`, `protected` |
| **修饰符** | `static`, `final`, `abstract`, `native` |
| **类型** | `void`, `int`, `long`, `float`, `double`, `boolean`, `bool`, `char`, `string`, `String` |
| **控制流** | `if`, `else`, `while`, `for`, `do`, `switch`, `case`, `default`, `break`, `continue`, `return` |
| **字面量** | `true`, `false`, `null` |
| **面向对象** | `class`, `this`, `super`, `new` |
| **其他** | `new` |

### 3.5 标识符

标识符命名规则:
- 以字母(A-Z, a-z)或下划线(`_`)开头
- 后续字符可以是字母、数字(0-9)或下划线
- 区分大小写
- 不能是关键字

```cay
// 有效标识符
name
_name
name123
firstName
first_name
MAX_SIZE

// 无效标识符
123name      // 数字开头
class        // 关键字
my-name      // 包含连字符
```

### 3.6 字面量

#### 3.6.1 整数字面量

```cay
// 十进制
int a = 42;
int b = 1_000_000;  // 下划线分隔符

// 十六进制 (0x 或 0X 前缀)
int hex = 0xFF;     // 255
int hex2 = 0x1A_2B; // 6699

// 二进制 (0b 或 0B 前缀)
int bin = 0b1010;   // 10
int bin2 = 0b1111_0000;

// 八进制 (0o 或 0O 前缀)
int oct = 0o77;     // 63

// Long类型后缀
long big = 10000000000L;
long hexLong = 0xFFFFFFFFL;
```

#### 3.6.2 浮点数字面量

```cay
// 双精度 (默认)
double d1 = 3.14159;
double d2 = 2.5e10;     // 科学计数法
double d3 = 1_000.000_001;

// 单精度 (f 或 F 后缀)
float f1 = 3.14f;
float f2 = 1.5e-3f;

// 双精度 (d 或 D 后缀，可选)
double d4 = 3.14d;
```

#### 3.6.3 布尔字面量

```cay
boolean flag = true;
boolean empty = false;
```

#### 3.6.4 字符字面量

```cay
char a = 'A';
char digit = '9';
char newline = '\n';
char tab = '\t';
char backslash = '\\';
char quote = '\'';
```

支持的转义序列:
| 转义序列 | 含义 |
|---------|------|
| `\n` | 换行 |
| `\t` | 制表符 |
| `\r` | 回车 |
| `\\` | 反斜杠 |
| `\'` | 单引号 |
| `\"` | 双引号 |
| `\0` | 空字符 |

#### 3.6.5 字符串字面量

```cay
String s1 = "Hello, World!";
String s2 = "Line 1\nLine 2";
String s3 = "Tab\there";
String empty = "";
```

#### 3.6.6 null字面量

```cay
String str = null;
Object obj = null;
```

### 3.7 运算符和分隔符

```
+  -  *  /  %     // 算术运算符
== != <  <= > >=  // 比较运算符
&& || !           // 逻辑运算符
&  |  ^  ~ << >> >>>  // 位运算符
=  += -= *= /= %= // 赋值运算符
++ --             // 自增自减
() {} []          // 括号
; , . :: -> ...   // 分隔符
? :               // 三元运算符
```

---

## 4. 类型系统

### 4.1 类型概述

Cavvy是静态类型语言，所有变量和表达式都有确定的类型。

### 4.2 基本类型

| 类型 | 大小 | 范围 | 说明 |
|------|------|------|------|
| `int` | 4字节 | -2³¹ ~ 2³¹-1 | 32位有符号整数 |
| `long` | 8字节 | -2⁶³ ~ 2⁶³-1 | 64位有符号整数 |
| `float` | 4字节 | IEEE 754单精度 | 32位浮点数 |
| `double` | 8字节 | IEEE 754双精度 | 64位浮点数 |
| `boolean`/`bool` | 1字节 | true/false | 布尔值 |
| `char` | 1字节 | 0 ~ 255 | ASCII字符 |
| `void` | - | - | 无返回值 |

### 4.3 引用类型

| 类型 | 说明 |
|------|------|
| `String` | 字符串(不可变) |
| 类类型 | 用户定义的类 |
| 数组类型 | 任何类型的数组 |

### 4.4 类型转换

#### 4.4.1 隐式转换(自动)

较小范围类型可自动转换为较大范围类型:

```cay
int i = 100;
long l = i;         // int → long (自动)
float f = i;        // int → float (自动)
double d = f;       // float → double (自动)
```

隐式转换层次:
```
int → long → float → double
int → float → double
```

#### 4.4.2 显式转换(强制)

```cay
double d = 3.14;
int i = (int)d;     // double → int (截断小数)

long l = 1000000;
int small = (int)l; // long → int (可能溢出)

float f = 3.9f;
int truncated = (int)f;  // 结果为 3
```

**注意**: 显式转换可能导致数据丢失。

#### 4.4.3 字面量类型推断

```cay
// 整数默认int
int a = 10;

// 小数默认double
double d = 3.14;

// 后缀指定类型
long l = 100L;
float f = 3.14f;
double d2 = 3.14d;
```

---

## 5. 变量与常量

### 5.1 变量声明

```cay
// 基本语法: 类型 变量名 [= 初始值];
int age = 25;
String name = "Cavvy";
boolean active = true;

// 声明后赋值
int score;
score = 100;

// 同时声明多个变量(需要分别指定类型)
int a = 1;
int b = 2;
int c = 3;
```

### 5.2 final常量

使用 `final` 关键字声明不可变变量:

```cay
// final变量必须在声明时初始化
final int MAX_SIZE = 100;
final String GREETING = "Hello";
final double PI = 3.14159;

// 错误: 不能修改final变量
MAX_SIZE = 200;  // 编译错误!
```

### 5.3 变量作用域

```cay
public class ScopeExample {
    // 类级别变量(字段)
    static int classVar = 10;
    
    public static void main() {
        // 方法级别变量
        int methodVar = 20;
        
        if (methodVar > 10) {
            // 块级别变量
            int blockVar = 30;
            println(blockVar);  // OK
        }
        
        // println(blockVar);  // 错误: blockVar不可见
        println(methodVar);     // OK
        println(classVar);      // OK
    }
}
```

### 5.4 静态变量

```cay
public class Counter {
    // 静态变量 - 所有实例共享
    static int count = 0;
    
    public static void increment() {
        count = count + 1;
    }
    
    public static void main() {
        increment();
        increment();
        println(count);  // 输出: 2
    }
}
```

---

## 6. 运算符

### 6.1 运算符优先级

从高到低排列:

| 优先级 | 运算符 | 结合性 |
|--------|--------|--------|
| 1 | `()` `[]` `.` `::` | 左到右 |
| 2 | `++` `--` (后缀) | 左到右 |
| 3 | `++` `--` (前缀) `+` `-` `!` `~` `(type)` | 右到左 |
| 4 | `*` `/` `%` | 左到右 |
| 5 | `+` `-` | 左到右 |
| 6 | `<<` `>>` `>>>` | 左到右 |
| 7 | `<` `<=` `>` `>=` | 左到右 |
| 8 | `==` `!=` | 左到右 |
| 9 | `&` | 左到右 |
| 10 | `^` | 左到右 |
| 11 | `\|` | 左到右 |
| 12 | `&&` | 左到右 |
| 13 | `\|\|` | 左到右 |
| 14 | `?:` | 右到左 |
| 15 | `=` `+=` `-=` `*=` `/=` `%=` | 右到左 |

### 6.2 算术运算符

```cay
int a = 10, b = 3;

int sum = a + b;        // 13 - 加法
int diff = a - b;       // 7 - 减法
int prod = a * b;       // 30 - 乘法
int quot = a / b;       // 3 - 整数除法(截断)
int rem = a % b;        // 1 - 取模

// 浮点除法
double d = 10.0 / 3.0;  // 3.333...
```

### 6.3 比较运算符

```cay
int a = 10, b = 20;

boolean eq = (a == b);   // false - 等于
boolean ne = (a != b);   // true - 不等于
boolean lt = (a < b);    // true - 小于
boolean le = (a <= b);   // true - 小于等于
boolean gt = (a > b);    // false - 大于
boolean ge = (a >= b);   // false - 大于等于
```

### 6.4 逻辑运算符

```cay
boolean x = true, y = false;

boolean and = x && y;    // false - 逻辑与
boolean or = x || y;     // true - 逻辑或
boolean not = !x;        // false - 逻辑非

// 短路求值
if (x && (10 / 0 > 0)) {  // 不会抛出除零错误，因为x为false时第二部分不执行
    // ...
}
```

### 6.5 位运算符

```cay
int a = 0b1100;  // 12
int b = 0b1010;  // 10

int and = a & b;     // 0b1000 = 8 - 按位与
int or = a | b;      // 0b1110 = 14 - 按位或
int xor = a ^ b;     // 0b0110 = 6 - 按位异或
int not = ~a;        // 按位取反

int left = a << 2;   // 0b110000 = 48 - 左移
int right = a >> 2;  // 0b0011 = 3 - 右移(算术)
```

### 6.6 赋值运算符

```cay
int a = 10;

a += 5;   // a = a + 5;  → 15
a -= 3;   // a = a - 3;  → 12
a *= 2;   // a = a * 2;  → 24
a /= 4;   // a = a / 4;  → 6
a %= 4;   // a = a % 4;  → 2
```

### 6.7 自增自减运算符

```cay
int a = 5;
int b = 5;

// 前缀形式: 先增减，后使用
int x = ++a;  // a = 6, x = 6

// 后缀形式: 先使用，后增减
int y = b++;  // y = 5, b = 6

// 单独使用时效果相同
a++;  // a = 7
++b;  // b = 7
```

### 6.8 三元运算符

```cay
int a = 10, b = 20;
int max = (a > b) ? a : b;  // max = 20

String result = (score >= 60) ? "及格" : "不及格";
```

---

## 7. 控制流

### 7.1 if-else语句

```cay
// 基本if
if (condition) {
    // 条件为true时执行
}

// if-else
if (score >= 90) {
    println("优秀");
} else if (score >= 80) {
    println("良好");
} else if (score >= 60) {
    println("及格");
} else {
    println("不及格");
}

// 嵌套if
if (x > 0) {
    if (y > 0) {
        println("第一象限");
    } else {
        println("第四象限");
    }
}
```

### 7.2 switch语句

```cay
switch (day) {
    case 1:
        println("星期一");
        break;
    case 2:
        println("星期二");
        break;
    case 3:
        println("星期三");
        break;
    case 4:
        println("星期四");
        break;
    case 5:
        println("星期五");
        break;
    default:
        println("周末");
        break;
}

// case穿透(多个case共享代码)
switch (month) {
    case 3:
    case 4:
    case 5:
        println("春季");
        break;
    case 6:
    case 7:
    case 8:
        println("夏季");
        break;
    // ...
}
```

**注意**: Cavvy的switch目前只支持整数类型。

### 7.3 while循环

```cay
// 基本while
int i = 0;
while (i < 10) {
    println(i);
    i = i + 1;
}

// 无限循环
while (true) {
    // 需要内部break退出
    if (condition) {
        break;
    }
}
```

### 7.4 for循环

```cay
// 基本for循环
for (int i = 0; i < 10; i = i + 1) {
    println(i);
}

// 复杂条件
for (int i = 0, j = 10; i < j; i = i + 1, j = j - 1) {
    println(i);
    println(j);
}

// 死循环
for (;;) {
    // 无限循环
    break;
}
```

### 7.5 do-while循环

```cay
// do-while至少执行一次
int i = 0;
do {
    println(i);
    i = i + 1;
} while (i < 10);

// 用户输入验证示例
do {
    println("请输入正数:");
    num = (int)readFloat();
} while (num <= 0);
```

### 7.6 break和continue

```cay
// break: 立即退出循环
for (int i = 0; i < 100; i = i + 1) {
    if (i == 50) {
        break;  // 退出循环，i=50时停止
    }
    println(i);
}

// continue: 跳过当前迭代
for (int i = 0; i < 10; i = i + 1) {
    if (i % 2 == 0) {
        continue;  // 跳过偶数
    }
    println(i);  // 只打印奇数
}
```

---

## 8. 数组

### 8.1 数组声明与创建

```cay
// 声明数组变量
int[] numbers;

// 创建数组
numbers = new int[5];  // 5个元素，初始化为0

// 声明并创建
String[] names = new String[3];

// 获取数组长度
int len = numbers.length;  // 5
```

### 8.2 数组初始化

```cay
// 使用初始化列表
int[] nums = {1, 2, 3, 4, 5};
String[] fruits = {"apple", "banana", "orange"};

// 先声明后初始化(需要new)
int[] arr = new int[]{10, 20, 30};
```

### 8.3 数组访问与修改

```cay
int[] arr = new int[5];

// 赋值
arr[0] = 10;
arr[1] = 20;
arr[2] = 30;

// 访问
int first = arr[0];  // 10
int second = arr[1]; // 20

// 遍历数组
for (int i = 0; i < arr.length; i = i + 1) {
    println(arr[i]);
}
```

### 8.4 多维数组

```cay
// 二维数组声明与创建
int[][] matrix = new int[3][4];  // 3行4列

// 访问二维数组
matrix[0][0] = 1;
matrix[1][2] = 5;
int val = matrix[1][2];  // 5

// 获取维度长度
int rows = matrix.length;      // 3
int cols = matrix[0].length;   // 4

// 遍历二维数组
for (int i = 0; i < matrix.length; i = i + 1) {
    for (int j = 0; j < matrix[i].length; j = j + 1) {
        print(matrix[i][j]);
        print(" ");
    }
    println("");
}
```

### 8.5 数组操作示例

```cay
// 数组求和
public static int sum(int[] arr) {
    int total = 0;
    for (int i = 0; i < arr.length; i = i + 1) {
        total = total + arr[i];
    }
    return total;
}

// 查找最大值
public static int findMax(int[] arr) {
    int max = arr[0];
    for (int i = 1; i < arr.length; i = i + 1) {
        if (arr[i] > max) {
            max = arr[i];
        }
    }
    return max;
}

// 数组反转
public static void reverse(int[] arr) {
    int left = 0;
    int right = arr.length - 1;
    while (left < right) {
        int temp = arr[left];
        arr[left] = arr[right];
        arr[right] = temp;
        left = left + 1;
        right = right - 1;
    }
}
```

---

## 9. 字符串

### 9.1 字符串创建

```cay
// 字符串字面量
String s1 = "Hello";
String s2 = "World";

// 空字符串
String empty = "";
```

### 9.2 字符串拼接

```cay
// 使用 + 运算符
String greeting = "Hello" + ", " + "World!";  // "Hello, World!"

// 与数字拼接
String message = "Count: " + 42;  // "Count: 42"

// 多行拼接
String multi = "Line 1\n" + "Line 2\n" + "Line 3";
```

### 9.3 字符串方法

Cavvy字符串支持以下内置方法:

#### 9.3.1 length()

```cay
String s = "Hello";
int len = s.length();  // 5
```

#### 9.3.2 substring()

```cay
String s = "Hello, World!";

// substring(beginIndex) - 从beginIndex到末尾
String sub1 = s.substring(7);      // "World!"

// substring(beginIndex, endIndex) - [beginIndex, endIndex)
String sub2 = s.substring(0, 5);   // "Hello"
String sub3 = s.substring(7, 12);  // "World"
```

#### 9.3.3 indexOf()

```cay
String s = "The quick brown fox";

int idx1 = s.indexOf("quick");   // 4
int idx2 = s.indexOf("cat");     // -1 (未找到)
```

#### 9.3.4 replace()

```cay
String s = "I love Java, Java is great!";
String replaced = s.replace("Java", "Cavvy");
// "I love Cavvy, Cavvy is great!"
```

#### 9.3.5 charAt()

```cay
String s = "ABCDEF";
char c = s.charAt(0);  // 'A'
char d = s.charAt(3);  // 'D'
```

### 9.4 字符串操作示例

```cay
public static boolean isPalindrome(String s) {
    int left = 0;
    int right = s.length() - 1;
    while (left < right) {
        if (s.charAt(left) != s.charAt(right)) {
            return false;
        }
        left = left + 1;
        right = right - 1;
    }
    return true;
}

public static int countOccurrences(String text, String pattern) {
    int count = 0;
    int index = 0;
    while ((index = text.indexOf(pattern)) != -1) {
        count = count + 1;
        text = text.substring(index + pattern.length());
    }
    return count;
}
```

---

## 10. 类与面向对象

### 10.1 类定义

```cay
// 基本类定义
public class Person {
    // 字段(成员变量)
    String name;
    int age;
    
    // 方法
    public static void greet() {
        println("Hello!");
    }
}
```

### 10.2 访问修饰符

| 修饰符 | 同一类 | 同一包 | 子类 | 任何地方 |
|--------|--------|--------|------|----------|
| `public` | ✓ | ✓ | ✓ | ✓ |
| `protected` | ✓ | ✓ | ✓ | ✗ |
| `private` | ✓ | ✗ | ✗ | ✗ |
| 默认 | ✓ | ✓ | ✗ | ✗ |

```cay
public class AccessExample {
    public int publicVar;       // 公开访问
    private int privateVar;     // 仅类内访问
    protected int protectedVar; // 包内和子类访问
    int defaultVar;             // 包内访问
}
```

### 10.3 静态成员

```cay
public class MathUtils {
    // 静态常量
    public static final double PI = 3.14159;
    
    // 静态变量
    private static int counter = 0;
    
    // 静态方法
    public static int add(int a, int b) {
        return a + b;
    }
    
    public static void main() {
        // 直接通过类名访问
        double pi = MathUtils.PI;
        int sum = MathUtils.add(3, 4);
        println(sum);  // 7
    }
}
```

### 10.4 主类与程序入口

```cay
// 使用@main注解指定主类
@main
public class MainApp {
    public static void main() {
        println("程序启动");
    }
}

// 或使用public class包含main方法
public class Application {
    public static void main() {
        println("Hello");
    }
}
```

---

## 11. 方法

### 11.1 方法定义

```cay
// 基本语法: [修饰符] 返回类型 方法名([参数列表]) { 方法体 }

// 无参数，无返回值
public static void sayHello() {
    println("Hello!");
}

// 有参数，有返回值
public static int add(int a, int b) {
    return a + b;
}

// 多个参数
public static double average(int a, int b, int c) {
    return (a + b + c) / 3.0;
}
```

### 11.2 方法重载

同名方法可以有不同的参数列表:

```cay
public class Calculator {
    // 无参数版本
    public static int add() {
        return 0;
    }
    
    // 一个参数
    public static int add(int a) {
        return a;
    }
    
    // 两个int参数
    public static int add(int a, int b) {
        return a + b;
    }
    
    // 三个int参数
    public static int add(int a, int b, int c) {
        return a + b + c;
    }
    
    // 两个double参数(不同类型)
    public static double add(double a, double b) {
        return a + b;
    }
    
    // 字符串版本
    public static String add(String a, String b) {
        return a + b;
    }
    
    public static void main() {
        println(add());           // 0
        println(add(5));          // 5
        println(add(3, 7));       // 10
        println(add(1, 2, 3));    // 6
        println(add(2.5, 3.5));   // 6.0
        println(add("Hello, ", "World!"));  // "Hello, World!"
    }
}
```

### 11.3 可变参数

```cay
// 可变参数语法: 类型... 参数名

// 纯可变参数
public static int sum(int... numbers) {
    int total = 0;
    for (int i = 0; i < numbers.length; i = i + 1) {
        total = total + numbers[i];
    }
    return total;
}

// 固定参数 + 可变参数
public static int multiplyAndAdd(int multiplier, int... numbers) {
    int total = 0;
    for (int i = 0; i < numbers.length; i = i + 1) {
        total = total + numbers[i];
    }
    return total * multiplier;
}

public static void main() {
    println(sum());                    // 0
    println(sum(1));                   // 1
    println(sum(1, 2, 3, 4, 5));       // 15
    println(multiplyAndAdd(2, 3, 4));  // 14 (3+4)*2
}
```

### 11.4 递归方法

```cay
// 阶乘
public static int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// 斐波那契数列
public static int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// 二分查找
public static int binarySearch(int[] arr, int target, int left, int right) {
    if (left > right) {
        return -1;
    }
    int mid = left + (right - left) / 2;
    if (arr[mid] == target) {
        return mid;
    } else if (arr[mid] < target) {
        return binarySearch(arr, target, mid + 1, right);
    } else {
        return binarySearch(arr, target, left, mid - 1);
    }
}
```

---

## 12. Lambda表达式与方法引用

### 12.1 Lambda表达式

Lambda表达式提供了一种简洁的匿名函数写法:

```cay
// 语法: (参数列表) -> { 方法体 }

// 无参数Lambda
var greet = () -> { println("Hello!"); };
greet();

// 单参数Lambda
var square = (int x) -> { return x * x; };
int result = square(5);  // 25

// 多参数Lambda
var add = (int a, int b) -> { return a + b; };
int sum = add(3, 4);  // 7

// 单表达式Lambda(可省略return和大括号)
var multiply = (int a, int b) -> a * b;
```

### 12.2 方法引用

方法引用提供了一种引用已有方法的简洁方式:

```cay
// 语法: 类名::方法名

// 静态方法引用
var ref = Calculator::add;
int result = ref(3, 4);  // 7
```

---

## 13. 预处理器

Cavvy支持简单的预处理器指令，用于条件编译。

### 13.1 #define

定义宏(无值的宏用于条件编译):

```cay
#define DEBUG
#define VERSION 100
```

### 13.2 #ifdef / #ifndef

条件编译:

```cay
#define DEBUG

public class DebugExample {
    public static void main() {
        int result = 0;
        
        #ifdef DEBUG
        result = result + 1;
        println("Debug mode enabled");
        #endif
        
        #ifndef RELEASE
        result = result + 10;
        println("Not in release mode");
        #endif
        
        println(result);
    }
}
```

### 13.3 #endif

结束条件编译块:

```cay
#define FEATURE_X

#ifdef FEATURE_X
// 这段代码会被编译
println("Feature X is available");
#endif
```

---

## 14. 内置函数

Cavvy提供以下内置函数，无需声明即可使用:

### 14.1 输出函数

```cay
// print - 输出值(不换行)
print("Hello");
print(42);
print(3.14);

// println - 输出值(换行)
println("Hello");
println(42);
println(3.14);

// 支持类型: int, long, float, double, boolean, char, String
```

### 14.2 输入函数

```cay
// readInt() - 读取整数，返回long
long num = readInt();

// readFloat() - 读取浮点数，返回double
double val = readFloat();

// readLine() - 读取一行字符串，返回String
String line = readLine();
```

### 14.3 使用示例

```cay
public class InputOutput {
    public static void main() {
        println("请输入您的名字:");
        String name = readLine();
        
        println("请输入您的年龄:");
        long age = readInt();
        
        println("您好, " + name + "!");
        print("您今年 ");
        print(age);
        println(" 岁。");
    }
}
```

---

## 15. 编译器工具链

### 15.1 cayc - 一站式编译器

```bash
# 基本用法
cayc source.cay output.exe

# 示例
cayc hello.cay hello.exe
./hello.exe
```

### 15.2 cay-ir - IR生成器

```bash
# 生成LLVM IR文件
cay-ir source.cay output.ll

# 查看生成的IR
cay-ir hello.cay hello.ll
cat hello.ll
```

### 15.3 ir2exe - IR转可执行文件

```bash
# 将LLVM IR编译为可执行文件
ir2exe input.ll output.exe

# 示例
ir2exe hello.ll hello.exe
./hello.exe
```

### 15.4 cay-check - 语法检查

```bash
# 检查代码语法，不生成输出
cay-check source.cay

# 示例
cay-check hello.cay
# 输出: Syntax OK 或错误信息
```

---

## 16. EBNF语法规范

Cavvy的完整EBNF语法定义如下:

```ebnf
(* ============================================================================
 * Cavvy 语法规范 - EBNF 表示
 * 版本: 0.3.4
 * ============================================================================ *)

(* ----------------------------------------------------------------------------
 * 预处理器指令
 * ---------------------------------------------------------------------------- *)
preprocessor_directive = define_directive
                       | ifdef_directive
                       | ifndef_directive
                       | endif_directive;

define_directive = "#define", identifier, [ replacement_text ];
replacement_text = { any_character_except_newline };
ifdef_directive = "#ifdef", identifier;
ifndef_directive = "#ifndef", identifier;
endif_directive = "#endif";

(* ----------------------------------------------------------------------------
 * 程序结构
 * ---------------------------------------------------------------------------- *)
program = { preprocessor_directive | class_declaration };

class_declaration = [ annotation ], [ modifiers ], "class", identifier, 
                    [ extends_clause ], [ implements_clause ], "{", { class_member }, "}";

extends_clause = "extends", identifier | ":", identifier;
implements_clause = "implements", identifier, { ",", identifier };
annotation = "@", identifier;
class_member = field_declaration 
             | method_declaration 
             | constructor_declaration 
             | destructor_declaration
             | instance_initializer
             | static_initializer;

(* ----------------------------------------------------------------------------
 * 修饰符
 * ---------------------------------------------------------------------------- *)
modifiers = modifier, { modifier };
modifier = "public" | "private" | "protected" | "static" | "final" | 
           "abstract" | "native";

(* ----------------------------------------------------------------------------
 * 字段声明
 * ---------------------------------------------------------------------------- *)
field_declaration = [ modifiers ], type, identifier, [ "=", expression ], ";";

(* ----------------------------------------------------------------------------
 * 方法声明
 * ---------------------------------------------------------------------------- *)
method_declaration = [ modifiers ], ( type | "void" ), identifier, 
                     "(", [ parameter_list ], ")", ( block | ";" );

(* 构造函数声明 *)
constructor_declaration = [ modifiers ], identifier, 
                          "(", [ parameter_list ], ")", 
                          [ constructor_call ], block;

(* 构造链调用：this() 或 super() *)
constructor_call = ":", ( "this" | "super" ), "(", [ argument_list ], ")";

(* 析构函数声明 *)
destructor_declaration = [ modifiers ], "~", identifier, 
                         "(", ")", block;

(* 实例初始化块 *)
instance_initializer = block;

(* 静态初始化块 *)
static_initializer = "static", block;

parameter_list = parameter, { ",", parameter } | varargs_parameter;
parameter = type, identifier;
varargs_parameter = type, "...", identifier;

(* ----------------------------------------------------------------------------
 * 类型系统
 * ---------------------------------------------------------------------------- *)
type = primitive_type | reference_type;
primitive_type = "int" | "long" | "float" | "double" | "bool" | "string" | "char";
reference_type = identifier, { "[", "]" } | primitive_type, { "[", "]" };

(* ----------------------------------------------------------------------------
 * 语句
 * ---------------------------------------------------------------------------- *)
block = "{", { statement }, "}";

statement = block
          | variable_declaration
          | if_statement
          | while_statement
          | for_statement
          | do_while_statement
          | switch_statement
          | return_statement
          | break_statement
          | continue_statement
          | expression_statement;

variable_declaration = [ "final" ], type, identifier, 
                       [ "=", ( expression | array_initializer ) ], ";";
array_initializer = "{", [ expression, { ",", expression } ], "}";

if_statement = "if", "(", expression, ")", statement, [ "else", statement ];
while_statement = "while", "(", expression, ")", statement;
for_statement = "for", "(", [ statement ], [ expression ], ";", 
                [ expression ], ")", statement;
do_while_statement = "do", statement, "while", "(", expression, ")", ";";

switch_statement = "switch", "(", expression, ")", "{", 
                   { case_clause }, [ default_clause ], "}";
case_clause = "case", integer_literal, ":", { statement };
default_clause = "default", ":", { statement };

return_statement = "return", [ expression ], ";";
break_statement = "break", ";";
continue_statement = "continue", ";";
expression_statement = expression, ";";

(* ----------------------------------------------------------------------------
 * 表达式 (按优先级从低到高)
 * ---------------------------------------------------------------------------- *)
expression = assignment_expression;

assignment_expression = conditional_or_expression, 
                        [ assignment_operator, assignment_expression ];
assignment_operator = "=" | "+=" | "-=" | "*=" | "/=" | "%=";

conditional_or_expression = conditional_and_expression, 
                            { "||", conditional_and_expression };
conditional_and_expression = bitwise_or_expression, 
                             { "&&", bitwise_or_expression };

bitwise_or_expression = bitwise_xor_expression, { "|", bitwise_xor_expression };
bitwise_xor_expression = bitwise_and_expression, { "^", bitwise_and_expression };
bitwise_and_expression = equality_expression, { "&", equality_expression };

equality_expression = relational_expression, { ( "==" | "!=" ), relational_expression };
relational_expression = shift_expression, { ( "<" | "<=" | ">" | ">=" ), shift_expression };
shift_expression = additive_expression, { ( "<<" | ">>" | ">>>" ), additive_expression };

additive_expression = multiplicative_expression, 
                      { ( "+" | "-" ), multiplicative_expression };
multiplicative_expression = unary_expression, { ( "*" | "/" | "%" ), unary_expression };

unary_expression = ( "-" | "!" | "~" | "++" | "--" ), unary_expression
                 | cast_expression
                 | postfix_expression;

cast_expression = "(", type, ")", unary_expression;

postfix_expression = primary_expression, { postfix_operator };
postfix_operator = "(", [ argument_list ], ")"
                 | ".", identifier, [ "(", [ argument_list ], ")" ]
                 | "[", expression, "]"
                 | "++"
                 | "--";

argument_list = expression, { ",", expression };

primary_expression = literal
                   | identifier
                   | "(", expression, ")"
                   | array_creation_expression
                   | object_creation_expression;

array_creation_expression = "new", type, "[", expression, "]", 
                            { "[", expression, "]" }, [ "(", ")" ];
object_creation_expression = "new", identifier, "(", [ argument_list ], ")";

(* ----------------------------------------------------------------------------
 * 字面量
 * ---------------------------------------------------------------------------- *)
literal = integer_literal
        | floating_point_literal
        | string_literal
        | character_literal
        | boolean_literal
        | null_literal;

integer_literal = [ "-" ], ( decimal_literal | hexadecimal_literal | 
                   binary_literal | octal_literal ), [ integer_suffix ];
decimal_literal = digit, { digit | "_" };
hexadecimal_literal = "0x" | "0X", hex_digit, { hex_digit | "_" };
binary_literal = "0b" | "0B", binary_digit, { binary_digit | "_" };
octal_literal = "0o" | "0O", octal_digit, { octal_digit | "_" };
integer_suffix = "L" | "l";

floating_point_literal = [ "-" ], decimal_floating_point_literal, 
                         [ floating_point_suffix ];
decimal_floating_point_literal = digit, { digit | "_" }, ".", { digit | "_" }, [ exponent_part ]
                               | ".", digit, { digit | "_" }, [ exponent_part ]
                               | digit, { digit | "_" }, exponent_part;
exponent_part = ( "e" | "E" ), [ "+" | "-" ], digit, { digit | "_" };
floating_point_suffix = "F" | "f" | "D" | "d";

string_literal = '"', { string_character }, '"';
string_character = any_character_except_double_quote | escape_sequence;

character_literal = "'", ( character | escape_sequence ), "'";
boolean_literal = "true" | "false";
null_literal = "null";

(* ----------------------------------------------------------------------------
 * 基本定义
 * ---------------------------------------------------------------------------- *)
identifier = letter | "_", { letter | digit | "_" };
letter = "a" .. "z" | "A" .. "Z";
digit = "0" .. "9";
hex_digit = digit | "a" .. "f" | "A" .. "F";
binary_digit = "0" | "1";
octal_digit = "0" .. "7";

escape_sequence = "\\", ( "n" | "t" | "r" | "\\" | "'" | '"' );
```

---

## 17. 编译器架构

### 17.1 架构概览

Cavvy编译器采用经典的前端-中端-后端架构:

```
+-----------+    +-----------+    +-----------+    +-----------+
| Cavvy源码 | -> |  词法分析  | -> |  语法分析  | -> |    AST    |
|  (.cay)   |    |  (Lexer)  |    | (Parser)  |    |           |
+-----------+    +-----------+    +-----------+    +-----+-----+
                                                          |
+-----------+    +-----------+    +-----------+          v
| Windows   | <- |  LLVM IR  | <- |  代码生成  | <- |  语义分析  |
|  EXE文件  |    |  生成器   |    | (Codegen) |    | (Semantic)|
+-----------+    +-----------+    +-----------+    +-----------+
```

### 17.2 源码结构

```
src/
├── bin/                    # 可执行文件入口
│   ├── cayc.rs            # 一站式编译器
│   ├── cay-ir.rs          # Cavvy -> IR
│   ├── ir2exe.rs          # IR -> EXE
│   └── cay-check.rs       # 语法检查
├── lexer/                 # 词法分析器
│   └── mod.rs
├── parser/                # 语法分析器
│   ├── mod.rs
│   ├── classes.rs         # 类解析
│   ├── statements.rs      # 语句解析
│   ├── types.rs           # 类型解析
│   ├── utils.rs           # 解析工具
│   └── expressions/       # 表达式解析
│       ├── mod.rs
│       ├── assignment.rs
│       ├── binary.rs
│       ├── lambda.rs
│       ├── postfix.rs
│       ├── primary.rs
│       └── unary.rs
├── semantic/              # 语义分析
│   ├── mod.rs
│   ├── analyzer.rs
│   ├── class_analysis.rs
│   ├── expr_inference.rs
│   ├── symbol_table.rs
│   ├── type_check.rs
│   └── type_utils.rs
├── codegen/               # 代码生成
│   ├── mod.rs
│   ├── generator.rs
│   ├── expressions.rs
│   ├── context.rs
│   ├── types.rs
│   ├── statements/        # 语句代码生成
│   │   ├── mod.rs
│   │   ├── block.rs
│   │   ├── if_stmt.rs
│   │   ├── loops.rs
│   │   ├── return_stmt.rs
│   │   ├── statement.rs
│   │   ├── switch_stmt.rs
│   │   ├── var_decl.rs
│   │   └── jump_stmt.rs
│   └── runtime/           # 运行时支持
│       ├── mod.rs
│       ├── int_to_string.rs
│       ├── float_to_string.rs
│       ├── bool_to_string.rs
│       ├── char_to_string.rs
│       ├── string_concat.rs
│       ├── string_length.rs
│       ├── string_substring.rs
│       ├── string_indexof.rs
│       ├── string_replace.rs
│       └── string_charat.rs
├── preprocessor/          # 预处理器
│   └── mod.rs
├── ast.rs                 # AST定义
├── types.rs               # 类型系统
├── error.rs               # 错误处理
└── lib.rs
```

### 17.3 编译流程

1. **预处理**: 处理 `#define`, `#ifdef`, `#ifndef`, `#endif` 指令
2. **词法分析**: 将源码转换为Token序列
3. **语法分析**: 将Token序列解析为AST
4. **语义分析**: 类型检查、符号解析、方法重载解析
5. **代码生成**: 将AST转换为LLVM IR
6. **编译链接**: 使用LLVM和MinGW生成可执行文件

---

## 18. 开发路线图

### 18.1 版本号规范 (0.B.M.P)

| 位置 | 名称 | 含义 |
|------|------|------|
| 0 | Generation | 架构代际 (0=LLVM后端, 1=自托管, 2=内存安全) |
| B | Big | 功能域里程碑 |
| M | Middle | 特性集群 |
| P | Patch | 每日构建修复 |

### 18.2 当前版本 (0.3.4)

**已完成功能**:
- ✅ 基础类型系统 (int, long, float, double, boolean, char, String, void)
- ✅ 变量声明和赋值
- ✅ 算术、比较、逻辑、位运算符
- ✅ 自增自减、复合赋值运算符
- ✅ 条件语句 (if-else, switch)
- ✅ 循环语句 (while, for, do-while)
- ✅ break/continue 支持
- ✅ 数组 (一维和多维)
- ✅ 字符串拼接和方法
- ✅ 类型转换 (显式和隐式)
- ✅ 方法重载
- ✅ 可变参数
- ✅ Lambda表达式
- ✅ 方法引用
- ✅ 内置函数 (print, println, readInt, readFloat, readLine)

### 18.3 未来规划

#### 阶段二: 面向对象核心 (0.4.x)
- 单继承模型
- 虚函数表(vtable)
- 方法重写与隐藏
- 访问控制
- 构造函数与析构函数

#### 阶段三: 零开销标准库 (0.5.x)
- 分配器接口
- 泛型集合
- 智能指针
- 系统级I/O

#### 阶段四: 错误处理与并发 (0.6.x)
- Result<T, E>错误处理
- 轻量级并发
- 异步I/O基础

#### G1代: 自举与现代化 (1.x)
- 用Cavvy重写编译器
- 类型推断增强
- 函数式编程支持
- 代数数据类型

#### G2代: 内存安全纪元 (2.x)
- 所有权系统
- 借用检查器
- 编译期内存安全

---

## 19. 附录

### 19.1 保留关键字列表

```
abstract    boolean     bool        break       case
char        class       continue    default     do
double      else        false       final       float
for         if          int         long        native
new         null        private     protected   public
return      static      string      String      super
switch      this        true        void        while
```

### 19.2 运算符优先级表

| 优先级 | 运算符 | 描述 |
|--------|--------|------|
| 1 | `() [] . ::` | 括号、数组访问、成员访问、方法引用 |
| 2 | `++ --` (后缀) | 后缀自增自减 |
| 3 | `++ -- + - ! ~ (type)` | 前缀运算符、类型转换 |
| 4 | `* / %` | 乘除模 |
| 5 | `+ -` | 加减 |
| 6 | `<< >> >>>` | 位移 |
| 7 | `< <= > >=` | 比较 |
| 8 | `== !=` | 相等比较 |
| 9 | `&` | 按位与 |
| 10 | `^` | 按位异或 |
| 11 | `\|` | 按位或 |
| 12 | `&&` | 逻辑与 |
| 13 | `\|\|` | 逻辑或 |
| 14 | `?:` | 三元条件 |
| 15 | `= += -= *= /= %=` | 赋值 |

### 19.3 类型大小与范围

| 类型 | 大小 | 最小值 | 最大值 |
|------|------|--------|--------|
| int | 4字节 | -2,147,483,648 | 2,147,483,647 |
| long | 8字节 | -9,223,372,036,854,775,808 | 9,223,372,036,854,775,807 |
| float | 4字节 | ~1.4E-45 | ~3.4E+38 |
| double | 8字节 | ~4.9E-324 | ~1.8E+308 |
| boolean | 1字节 | false | true |
| char | 1字节 | 0 | 255 |

### 19.4 示例程序集锦

#### 九九乘法表
```cay
public class Multiplication {
    public static void main() {
        long i = 1;
        while (i <= 9) {
            long j = 1;
            while (j <= i) {
                print(i);
                print("x");
                print(j);
                print("=");
                print(i * j);
                if (i * j < 10) {
                    print("  ");
                } else {
                    print(" ");
                }
                j = j + 1;
            }
            println("");
            i = i + 1;
        }
    }
}
```

#### 冒泡排序
```cay
public class BubbleSort {
    public static void main() {
        int[] arr = {64, 34, 25, 12, 22, 11, 90};
        
        println("原始数组:");
        printArray(arr);
        
        bubbleSort(arr);
        
        println("排序后数组:");
        printArray(arr);
    }
    
    public static void bubbleSort(int[] arr) {
        int n = arr.length;
        for (int i = 0; i < n - 1; i = i + 1) {
            for (int j = 0; j < n - i - 1; j = j + 1) {
                if (arr[j] > arr[j + 1]) {
                    int temp = arr[j];
                    arr[j] = arr[j + 1];
                    arr[j + 1] = temp;
                }
            }
        }
    }
    
    public static void printArray(int[] arr) {
        for (int i = 0; i < arr.length; i = i + 1) {
            print(arr[i]);
            print(" ");
        }
        println("");
    }
}
```

#### 素数判断
```cay
public class PrimeCheck {
    public static boolean isPrime(int n) {
        if (n <= 1) {
            return false;
        }
        if (n <= 3) {
            return true;
        }
        if (n % 2 == 0 || n % 3 == 0) {
            return false;
        }
        int i = 5;
        while (i * i <= n) {
            if (n % i == 0 || n % (i + 2) == 0) {
                return false;
            }
            i = i + 6;
        }
        return true;
    }
    
    public static void main() {
        println("请输入一个整数:");
        int num = (int)readInt();
        
        if (isPrime(num)) {
            println("是素数");
        } else {
            println("不是素数");
        }
    }
}
```

### 19.5 常见问题

**Q: Cavvy支持哪些平台?**  
A: 目前主要支持Windows。Linux和macOS支持正在开发中。

**Q: Cavvy有垃圾回收吗?**  
A: 没有。Cavvy采用显式内存管理，无GC，确保内存可控。

**Q: 如何调试Cavvy程序?**  
A: 可以使用 `cay-ir` 生成LLVM IR查看中间代码，或使用 `println` 进行简单调试。

**Q: Cavvy与Java有什么区别?**  
A: Cavvy语法类似Java，但编译为原生机器码而非字节码，无JVM，无GC，性能更高。

**Q: 如何报告bug或建议?**  
A: 请访问 https://github.com/Ethernos-Studio/Cavvy 提交Issue。

---

## 许可证

Cavvy编程语言采用 GPL3 许可证。

---

*本文档最后更新于 2026-02-11，对应Cavvy版本 0.3.5*
