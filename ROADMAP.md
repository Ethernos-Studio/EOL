# EOL 语言开发路线图 (Roadmap)

## 项目概述
EOL (Ethernos Object Language) 是一个编译为 Windows 可执行文件的静态类型编程语言，语法设计目标与 Java 高度兼容。

**当前版本：0.2.0.x**

---

## 版本号规范 (0.B.M.P)

| 位置 | 名称 | 含义 | 示例 |
|------|------|------|------|
| **0** | Generation | 架构代际 | 0=LLVM后端, 1=自托管, 2=内存安全 |
| **B** | Big | 功能域里程碑 | 0.1=原型, 0.2=当前, 0.3=控制流完善... |
| **M** | Middle | 特性集群 | 0.3.1.x=循环家族 |
| **P** | Patch | 每日构建修复 | 0.3.1.0→0.3.1.1 |

---

## 已完成功能 (0.1.x - 0.2.x)

### 0.1.x 原型阶段
- [x] 基础词法/语法分析器
- [x] LLVM IR 代码生成
- [x] Windows EXE 输出
- [x] 基础类型（int, String, void）
- [x] 类和方法定义
- [x] if/else 和 while

### 0.2.x 当前阶段
- [x] 版本号集成（0.2.0）
- [x] 编译优化选项（LTO, PGO, SIMD）
- [x] IR 阶段优化（--opt-ir）
- [x] 完整的编译器驱动（eolc/eolll/ir2exe）

---

## 阶段一：控制流完善 (0.3.x.x)

### 0.3.1.x 循环家族
- [ ] **for 循环** - Java 风格 `for (int i = 0; i < n; i++)`
- [ ] **增强 for 循环** - `for (Type item : collection)` 遍历集合
- [ ] **do-while 循环** - `do { ... } while (condition);`
- [ ] **switch 语句** - Java 风格，支持 `case` 穿透和 `break`
- [ ] **break/continue 标签** - 嵌套循环控制 `outer: for (...) ... break outer;`

### 0.3.2.x 类型系统扩展
- [ ] **浮点类型** - `float`, `double` 支持
- [ ] **字符类型** - `char` 类型和字符字面量 `'A'`
- [ ] **布尔类型** - 原生 `boolean` 类型（true/false）
- [ ] **long 类型** - 64位有符号整数
- [ ] **类型转换** - 显式强制转换 `(int)value`

### 0.3.3.x 数组完备
- [ ] **多维数组** - `int[][] matrix = new int[3][3];`
- [ ] **数组初始化** - `int[] arr = {1, 2, 3};`
- [ ] **数组长度** - `arr.length` 属性
- [ ] **数组边界检查** - 运行时安全检查

### 0.3.4.x 字符串与方法
- [ ] **字符串增强** - `String` 类方法（substring, indexOf, replace等）
- [ ] **方法重载** - 同名不同参数列表
- [ ] **可变参数** - `void method(String fmt, Object... args)`
- [ ] **方法引用** - 静态/实例方法引用 `ClassName::methodName`
- [ ] **Lambda 表达式** - `(params) -> { body }`

---

## 阶段二：面向对象特性 (0.4.x.x)

### 0.4.1.x 继承与多态
- [ ] **继承** - `class Child extends Parent`
- [ ] **方法重写** - `@Override` 注解支持
- [ ] **多态** - 父类引用指向子类对象
- [ ] **抽象类** - `abstract class` 定义
- [ ] **接口** - `interface` 多实现 `implements`
- [ ] **访问修饰符** - `public/protected/private/default` 完整支持

### 0.4.2.x 构造与初始化
- [ ] **构造函数重载** - 多构造函数支持
- [ ] **构造函数链** - `this(...)` 和 `super(...)` 调用
- [ ] **初始化块** - 实例初始化块 `{ ... }`
- [ ] **静态初始化** - `static { ... }` 类级别初始化

### 0.4.3.x 核心类特性
- [ ] **final 类/方法** - 不可继承/重写
- [ ] **static 导入** - `import static ...`
- [ ] **内部类** - 成员内部类、静态内部类
- [ ] **匿名类** - `new Interface() { ... }`

### 0.4.4.x 泛型编程
- [ ] **泛型类** - `class Container<T>`
- [ ] **泛型方法** - `<T> T max(T a, T b)`
- [ ] **类型边界** - `<T extends Number>`
- [ ] **通配符** - `?`, `? extends T`, `? super T`
- [ ] **泛型擦除** - 编译时类型处理

---

## 阶段三：标准库建设 (0.5.x.x)

### 0.5.1.x 核心库 (java.lang 等效)
- [ ] **System 类** - `System.out.println()`, `System.currentTimeMillis()`
- [ ] **Math 类** - `Math.sin()`, `Math.sqrt()`, `Math.pow()`
- [ ] **Object 类** - 所有类的根类，`toString()`, `equals()`, `hashCode()`
- [ ] **包装类** - `Integer`, `Double`, `Boolean` 等
- [ ] **String 类** - 不可变字符串，完整方法集
- [ ] **StringBuilder/StringBuffer** - 可变字符串

### 0.5.2.x 集合框架 (java.util 等效)
- [ ] **List 接口** - `ArrayList<T>`, `LinkedList<T>`
- [ ] **Set 接口** - `HashSet<T>`, `TreeSet<T>`
- [ ] **Map 接口** - `HashMap<K,V>`, `TreeMap<K,V>`
- [ ] **Queue/Deque** - `ArrayDeque<T>`, `PriorityQueue<T>`
- [ ] **Iterator** - `iterator()`, `hasNext()`, `next()`
- [ ] **Collections 工具** - `sort()`, `binarySearch()`, `shuffle()`

### 0.5.3.x 实用工具
- [ ] **Arrays 类** - `Arrays.sort()`, `Arrays.toString()`
- [ ] **Random 类** - 随机数生成
- [ ] **Date/Time API** - `LocalDate`, `LocalTime`, `LocalDateTime`
- [ ] **Formatter** - `String.format()`, `printf()`
- [ ] **Scanner** - 控制台输入解析
- [ ] **正则表达式** - `Pattern`, `Matcher`

### 0.5.4.x IO 与 NIO
- [ ] **File 类** - 文件/目录操作
- [ ] **Stream** - `InputStream`, `OutputStream`, `Reader`, `Writer`
- [ ] **Buffered IO** - `BufferedReader`, `BufferedWriter`
- [ ] **File IO** - `FileInputStream`, `FileOutputStream`
- [ ] **NIO.2** - `Path`, `Files`, `Paths`

---

## 阶段四：高级特性 (0.6.x.x)

### 0.6.1.x 异常处理
- [ ] **异常类层次** - `Throwable` > `Exception` > `RuntimeException`
- [ ] **try-catch-finally** - 完整异常处理
- [ ] **多重 catch** - `catch (A | B e)`
- [ ] **try-with-resources** - 自动资源管理
- [ ] **throw/throws** - 异常抛出声明
- [ ] **自定义异常** - 继承 `Exception` 或 `RuntimeException`

### 0.6.2.x 注解与反射
- [ ] **注解定义** - `@interface`
- [ ] **元注解** - `@Retention`, `@Target`
- [ ] **常用注解** - `@Override`, `@Deprecated`, `@SuppressWarnings`
- [ ] **反射 API** - `Class<?>`, `Method`, `Field`, `Constructor`

### 0.6.3.x 枚举与记录
- [ ] **枚举类型** - `enum Status { ACTIVE, INACTIVE }`
- [ ] **枚举方法** - 构造函数、字段、方法
- [ ] **记录类** - `record Point(int x, int y)`

### 0.6.4.x 并发编程 (java.util.concurrent 等效)
- [ ] **Thread 类** - 线程创建和启动
- [ ] **Runnable/Callable** - 任务接口
- [ ] **同步机制** - `synchronized`, `Lock`, `ReentrantLock`
- [ ] **线程池** - `ExecutorService`, `ThreadPoolExecutor`
- [ ] **并发集合** - `ConcurrentHashMap`, `CopyOnWriteArrayList`
- [ ] **原子类** - `AtomicInteger`, `AtomicBoolean`
- [ ] **CompletableFuture** - 异步编程

---

## 阶段五：模块系统与生态 (0.7.x.x)

### 0.7.1.x 包管理
- [ ] **包声明** - `package com.example.project;`
- [ ] **导入语句** - `import`, `import static`
- [ ] **访问控制** - 包级私有 (default)
- [ ] **包管理器** - 类似 Maven/Gradle 的依赖管理

### 0.7.2.x 模块系统 (Java 9+ 风格)
- [ ] **module-info.java** - 模块声明
- [ ] **exports** - 导出包
- [ ] **requires** - 依赖声明
- [ ] **服务提供** - `provides ... with ...`

### 0.7.3.x 开发工具
- [ ] **LSP 支持** - 语言服务器协议
- [ ] **VSCode 插件** - 语法高亮、跳转、补全、调试
- [ ] **代码格式化** - 类似 Eclipse/IDEA 格式化规则
- [ ] **静态分析** - 代码检查工具
- [ ] **单元测试** - JUnit 风格测试框架

### 0.7.4.x 跨平台支持
- [ ] **Linux 后端** - ELF 可执行文件
- [ ] **macOS 支持** - Mach-O 格式
- [ ] **JVM 后端** - 可选编译为 JVM 字节码

---

## 阶段六：性能优化 (0.8.x.x)

### 0.8.1.x 编译器优化
- [ ] **逃逸分析** - 栈上分配对象
- [ ] **内联优化** - 方法内联展开
- [ ] **常量折叠** - 编译期常量计算
- [ ] **死代码消除** - 移除未使用代码
- [ ] **SIMD 向量化** - 自动使用 SIMD 指令

### 0.8.2.x 运行时优化
- [ ] **JIT 编译** - 热点代码即时编译
- [ ] **GC 可选** - 垃圾回收器（可选启用）
- [ ] **AOT 编译** - 预编译为原生代码

---

## 第二级：EOL 特色语法 (0.9.x.x ~ 1.x.x.x)

在保持 Java 兼容性的基础上，引入 EOL 独特的语法糖。

### 7.1 现代 Lambda 与函数式编程
- [ ] **箭头函数语法** - `(para1, para2) -> { body }` 风格匿名函数
- [ ] **函数类型** - `Function<Int, Int> add = (a, b) -> a + b;`
- [ ] **闭包支持** - 完整闭包，捕获外部变量
- [ ] **高阶函数** - 函数作为参数和返回值
- [ ] **函数组合** - `f.andThen(g)`, `f.compose(g)`
- [ ] **柯里化** - `add(1)(2)(3)` 自动柯里化支持
- [ ] **管道操作符** - `data |> transform |> filter |> collect`

### 7.2 面向对象增强
- [ ] **结构体 (struct)** - 值类型数据结构 `struct Point { int x, y; }`
- [ ] **自定类型 (typedef/type)** - `type ID = String;` 类型别名增强
- [ ] **扩展方法** - `extend ClassName { newMethod() {} }` 为现有类添加方法
- [ ] **属性访问器** - `get/set` 自动属性 `property String name;`
- [ ] **数据类** - `@Data` 自动生成 equals/hashCode/toString
- [ ] **密封类** - `sealed class Shape permits Circle, Square`
- [ ] **模式匹配 (类)** - `if (obj instanceof Point(int x, int y))`

### 7.3 运算符重载与中缀函数
- [ ] **中缀函数 (expr)** - `expr fun add(a: Int, b: Int) = a + b` 然后 `1 add 2`
- [ ] **运算符重载** - `operator fun plus(other: Vector) = Vector(...)`
- [ ] **自定义运算符** - 定义新的运算符符号和优先级
- [ ] **范围运算符** - `1..10`, `'a'..'z'` 闭区间
- [ ] **安全调用** - `obj?.method()` 空安全调用
- [ ] **Elvis 运算符** - `name ?: "default"` 空值合并
- [ ] **非空断言** - `name!!` 强制非空

### 7.4 解构与模式匹配
- [ ] **解构声明** - `val (x, y) = point;`
- [ ] **数组解构** - `val [a, b, ...rest] = arr;`
- [ ] **when 表达式** - 增强 switch，支持模式匹配
  ```eol
  when (obj) {
      is Point(int x, int y) -> println("$x, $y");
      is String s && s.length > 5 -> println("long string");
      else -> println("other");
  }
  ```
- [ ] **守卫子句** - `case n if n > 0:` 带条件的 case
- [ ] **类型模式** - `case String s:` 自动类型转换
- [ ] **列表模式** - `case [1, 2, 3]:` 匹配列表内容

### 7.5 异步与并发语法糖
- [ ] **async/await** - `async fun foo()` 和 `await result`
- [ ] **异步流** - `async Stream<T>` 和 `yield` 生成器
- [ ] **结构化并发** - `async { ... }` 块，自动取消子任务
- [ ] **协程** - `suspend fun` 轻量级线程
- [ ] **选择表达式** - `select { case chan1.recv() -> ... }`

### 7.6 元编程与宏
- [ ] **编译期常量** - `const val MAX = 100;`
- [ ] **宏系统** - `macro!()` 编译时代码生成
- [ ] **代码注入** - `#[derive(Debug)]` 自动派生 trait
- [ ] **条件编译** - `#if DEBUG` 编译期条件
- [ ] **编译期反射** - 在编译时获取类型信息

### 7.7 内存与安全
- [ ] **所有权系统 (可选)** - 编译期内存安全（可选启用）
- [ ] **借用检查** - `&T`, `&mut T` 借用语义
- [ ] **智能指针** - `Box<T>`, `Rc<T>`, `Arc<T>`
- [ ] **生命周期** - 显式生命周期标注
- [ ] **unsafe 块** - `unsafe { ... }` 不安全代码隔离

### 7.8 集合与流式处理
- [ ] **集合字面量** - `#[1, 2, 3]`, `#{"a": 1, "b": 2}`
- [ ] **序列推导式** - `[x * 2 for x in list if x > 0]`
- [ ] **流式 API** - `list.stream().filter(...).map(...).collect()`
- [ ] **并行流** - `list.parallelStream()` 自动并行化
- [ ] **不可变集合** - `ImmutableList`, `ImmutableMap`

### 7.9 字符串与格式化
- [ ] **原始字符串** - `r"C:\Users\name"` 不转义
- [ ] **多行字符串** - `"""..."""` 保留格式
- [ ] **字符串模板** - `"Hello, $name!"` 和 `"Sum: ${a + b}"`
- [ ] **内插表达式** - `"Result: ${method()}"`
- [ ] **格式化字面量** - `f"{value:.2f}"` 格式控制

### 7.10 其他语法糖
- [ ] **尾随逗号** - 函数参数、数组末尾允许逗号
- [ ] **命名参数** - `drawPoint(x: 10, y: 20)`
- [ ] **默认参数** - `fun greet(name, greeting = "Hello")`
- [ ] **参数展开** - `call(*args, **kwargs)`
- [ ] **链式调用** - `obj.method1().method2().method3()`
- [ ] **空合并链** - `a ?? b ?? c ?? default`
- [ ] **提前返回** - `return if condition;` 守卫语句

### 7.11 与其他语言互操作
- [ ] **FFI 外部函数** - `extern "C"` 调用 C 库
- [ ] **JNI 兼容** - 与 Java 代码互操作
- [ ] **WebAssembly** - 编译为 WASM 在浏览器运行
- [ ] **Python 绑定** - 调用 Python 库

---

## 代际演进

| 代际 | 版本 | 目标 |
|------|------|------|
| **G0** | 0.x.x.x | LLVM 后端 + Java 兼容（当前） |
| **G1** | 1.x.x.x | 自托管编译器（用 EOL 写 EOL） |
| **G2** | 2.x.x.x | 所有权系统（内存安全纪元） |

---

## 开发优先级

| 优先级 | 特性类别 |
|--------|----------|
| P0 | Java 兼容性（0.3.x - 0.8.x） |
| P1 | 语法糖（中缀函数、解构、字符串模板） |
| P2 | 函数式（Lambda 增强、管道、高阶函数） |
| P3 | 异步（async/await、协程） |
| P4 | 元编程（宏、编译期计算） |
| P5 | 内存安全（所有权系统，实验性） |

---

**注意：** 本路线图会根据实际开发情况和社区反馈进行调整。
