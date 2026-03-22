## Cavvy v0.5.0.0 Release Notes

### 修改/Change

- **1. feat(rcpl): RCPL 支持预处理器指令** (commit TBD) by @Cavvy

功能实现:

- **上下文扩展**: 在 `Context` 结构体中添加 `preprocessor_directives` 字段，用于存储 `#include`, `#define`, `#ifdef` 等预处理器指令

- **API 增强**: `context.rs` 中新增预处理器指令管理方法
  - `add_preprocessor_directive()`: 添加指令（自动去重）
  - `remove_last_preprocessor_directive()`: 移除最后一条指令（回滚支持）
  - `preprocessor_directives()`: 获取指令列表
  - `show()`: 显示上下文中新增预处理器指令列表

- **输入解析**: `input_parser.rs` 已支持识别 `InputType::Preprocessor` 类型输入

- **上下文更新**: `mod.rs` 中 `update_context()` 方法处理 `Preprocessor` 类型输入，添加指令到上下文并显示提示

- **回滚支持**: `mod.rs` 中 `rollback_context()` 方法支持回滚预处理器指令

- **代码生成**: `code_generator.rs` 在生成代码时将预处理器指令放在文件最开头，确保 `#include` 等指令在其他代码之前处理

特性说明:
- 预处理器指令会自动去重，相同的指令不会重复添加
- `#include <IOPlus.cay>` 等标准库引用现在可以在 RCPL 中正常工作
- `#define` 宏定义和 `#ifdef` 条件编译完全支持
- 配合 `:ctx` 命令可以查看当前所有预处理器指令

- **2. feat(0.5.0.0): 实现内存分配器功能** (commit ead0361) by @dhjs0000

功能实现:

- **AST 扩展**: 添加 `ScopeStmt` (scope 栈作用域语句)、`AllocExpr` / `DeallocExpr` (内存分配/释放表达式)

- **词法分析器**: 新增 `Scope` token 关键字支持

- **语法解析器**: 实现 `parse_scope_statement` 函数解析 scope 语句，语法为 `scope { statements... }`

- **语义分析**: 为 `Alloc` 和 `Dealloc` 表达式添加类型推断，`Alloc` 返回 `long` 类型(指针)，`Dealloc` 返回 `void`

- **代码生成**:
  - `src/codegen/allocator.rs`: 分配器类型定义和 LLVM IR 运行时支持
  - `src/codegen/statements/scope_stmt.rs`: scope 语句代码生成，支持作用域隔离
  - `src/codegen/expressions/allocator.rs`: `__cay_alloc` / `__cay_free` 底层内存操作代码生成

- **语法规范**: 更新 `cavvy.ebnf`，添加 `scope_statement` 语法规则和版本历史记录

- **标准库**: 新增 `caylibs/Allocator.cay`
  - `Allocator` 接口: 定义 `allocate`/`deallocate` 方法契约
  - `GlobalAlloc`: 封装系统 `malloc`/`free` 的全局堆分配器单例
  - `Arena`: 线性分配器，支持批量分配/重置，适用于编译器、游戏帧分配等场景
  - `ScopeAllocator`: 栈作用域分配器，支持 RAII 模式

- **测试**: 新增 `tests/allocator_tests.rs`，包含 6 个分配器功能测试用例
  - `test_scope_statement`: scope 语句基本功能
  - `test_nested_scope`: 嵌套 scope 测试
  - `test_scope_variable_shadowing`: 变量遮蔽测试
  - `test_global_alloc`: GlobalAlloc 全局分配器
  - `test_arena_allocator`: Arena 线性分配器
  - `test_allocator_polymorphism`: 分配器接口多态

- **示例代码**:
  - `examples/test_0_5_0_allocator.cay`: 分配器功能综合测试
  - `examples/test_scope_basic.cay`: scope 语句功能测试

所有单元测试和集成测试通过

- **3. fix(tests): 修复 allocator 测试中的文件权限冲突问题** (commit fde4a61) by @Cavvy

修复:

- 在 `tests/common/mod.rs` 添加 `TEST_LOCK` 全局锁，确保测试串行执行避免文件冲突
- 修复 `test_scope_statement` 和 `test_scope_variable_shadowing` 测试的并发执行问题

## 额外/additional

- **RCPL 预处理器支持**: Cavvy 交互式解释器 (RCPL) 现在完全支持预处理器指令
  - 在 REPL 环境中可直接使用 `#include <Library.cay>` 加载标准库
  - 支持 `#define` 宏定义和条件编译指令
  - 指令自动去重和回滚机制确保错误处理的正确性

- **显式内存管理基础设施**: 为 Cavvy 语言引入零 GC、显式内存管理的基础能力

- **分配器接口设计**: 采用 trait-based 设计，支持自定义分配器实现

- **Arena 分配器**: 提供 O(1) 分配速度的线性分配器，适合高频小对象分配场景

- **scope 语法糖**: 简化栈作用域管理，支持变量遮蔽和自动资源释放

- **类型安全**: `Alloc` 返回 `long` 类型表示裸指针，`Dealloc` 接受 `long` 类型参数

- **LLVM IR 优化**: 分配器运行时函数内联优化，减少函数调用开销

- **向后兼容**: 所有现有代码无需修改即可在新版本编译运行

- 所有测试通过: 293+ 个测试
- 所有示例程序编译运行正常

## 文件/Files

| 文件名 | 适用于 | 额外说明 |
| ------ | ------ | -------- |
| `cavvy-0.5.0.0-win86-64-mingw64-llvm21.zip` | Windows x86_64 | 包含 MinGW 与 LLVM 的完整版本，适用于第一次安装 |
| `cavvy-0.5.0.0-win86-64-only-lib.zip` | Windows x86_64 | 包含 Cavvy 套件和编译期库文件（含新 Allocator.cay），适用于已安装 LLVM 与 MinGW 的环境 |
| `cavvy-0.5.0.0-win86-64-core.zip` | Windows x86_64 | 仅包含 Cavvy 套件的版本，适用于 Cavvy 版本升级 |

---

**兼容性说明**: v0.5.0.0 引入新的 `scope` 关键字，与现有代码无冲突。分配器库 `caylibs/Allocator.cay` 为可选组件，不影响不使用显式内存管理的代码。

**迁移指南**: 如需使用新的内存分配功能，请在代码中包含 `#include <Allocator.cay>`。
