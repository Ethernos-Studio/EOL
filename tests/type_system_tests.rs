//! 类型系统全面测试
//!
//! 测试类型别名、函数指针类型、类型解析等

mod common;
use common::compile_and_run_eol;

/// 测试基本类型别名
#[test]
fn test_basic_type_alias() {
    let code = r#"
alias MyInt = int;
alias MyLong = long;

extern {
    c_int printf(c_string fmt, ...);
}

public int main() {
    // 使用类型别名
    MyInt a = 10;
    MyLong b = 1000000;
    
    printf("MyInt: %d\n", a);
    printf("MyLong: %ld\n", b);
    printf("Basic type alias test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_type_basic_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("MyInt: 10"), 
                "MyInt alias should work, got: {}", output);
            assert!(output.contains("Basic type alias test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试C类型别名
#[test]
fn test_c_type_alias() {
    let code = r#"
alias CInt = c_int;
alias CLong = c_long;
alias CSize = size_t;
alias VoidPtr = ptr;

extern {
    c_int printf(c_string fmt, ...);
    VoidPtr malloc(CSize size);
    void free(VoidPtr p);
}

public int main() {
    CInt i = 42;
    CLong l = 123456789;
    CSize sz = 64;
    
    printf("CInt: %d\n", i);
    printf("CLong: %ld\n", l);
    printf("CSize: %zu\n", sz);
    
    VoidPtr p = malloc(sz);
    if (p != null) {
        printf("Allocated with C type aliases\n");
        free(p);
    }
    
    printf("C type alias test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_c_type_alias_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("CInt: 42"), 
                "CInt alias should work, got: {}", output);
            assert!(output.contains("C type alias test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试函数指针类型别名
#[test]
fn test_function_pointer_type() {
    let code = r#"
// 定义各种函数指针类型
alias BinaryOp = fn(int, int) -> int;
alias UnaryOp = fn(int) -> int;
alias VoidFn = fn() -> void;
alias Predicate = fn(int) -> bool;

extern {
    c_int printf(c_string fmt, ...);
}

fn add(a: int, b: int) -> int {
    return a + b;
}

fn negate(a: int) -> int {
    return -a;
}

fn always_true(a: int) -> bool {
    return true;
}

fn do_nothing() -> void {
    printf("Doing nothing\n");
}

public int main() {
    // 使用函数指针类型
    BinaryOp op = add;
    UnaryOp neg = negate;
    Predicate pred = always_true;
    VoidFn noop = do_nothing;
    
    int result1 = op(5, 3);
    int result2 = neg(10);
    bool result3 = pred(0);
    
    printf("add(5,3) = %d\n", result1);
    printf("negate(10) = %d\n", result2);
    printf("always_true(0) = %s\n", result3 ? "true" : "false");
    
    noop();
    
    printf("Function pointer type test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_fn_type_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("add(5,3) = 8"), 
                "BinaryOp should work, got: {}", output);
            assert!(output.contains("negate(10) = -10"), 
                "UnaryOp should work, got: {}", output);
            assert!(output.contains("Function pointer type test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试类型别名链
#[test]
fn test_type_alias_chain() {
    let code = r#"
alias IntAlias1 = int;
alias IntAlias2 = IntAlias1;
alias IntAlias3 = IntAlias2;

alias PtrAlias1 = ptr;
alias PtrAlias2 = PtrAlias1;

extern {
    c_int printf(c_string fmt, ...);
    PtrAlias2 malloc(size_t size);
    void free(PtrAlias3 p);
}

public int main() {
    // 使用多级类型别名
    IntAlias3 x = 100;
    printf("IntAlias3: %d\n", x);
    
    PtrAlias2 p = malloc(32);
    if (p != null) {
        printf("Allocated with PtrAlias2\n");
        free(p);
    }
    
    printf("Type alias chain test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_type_chain_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("IntAlias3: 100"), 
                "Type alias chain should work, got: {}", output);
            assert!(output.contains("Type alias chain test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试复杂函数指针类型
#[test]
fn test_complex_function_pointer() {
    let code = r#"
// 复杂函数指针类型
alias Callback = fn(int, ptr) -> bool;
alias Handler = fn(c_string, c_int) -> void;
alias Factory = fn() -> ptr;
alias Comparator = fn(ptr, ptr) -> c_int;

extern {
    c_int printf(c_string fmt, ...);
    ptr malloc(size_t size);
    void free(ptr p);
}

fn my_callback(code: int, data: ptr) -> bool {
    printf("Callback called with code %d\n", code);
    return code > 0;
}

fn my_handler(msg: c_string, level: c_int) -> void {
    printf("[%d] %s\n", level, msg);
}

fn my_factory() -> ptr {
    return malloc(16);
}

fn my_comparator(a: ptr, b: ptr) -> c_int {
    return 0;  // 简化：总是相等
}

public int main() {
    Callback cb = my_callback;
    Handler h = my_handler;
    Factory f = my_factory;
    Comparator cmp = my_comparator;
    
    bool result = cb(42, null);
    h("Test message", 1);
    ptr data = f();
    c_int cmp_result = cmp(data, data);
    
    printf("Callback result: %s\n", result ? "true" : "false");
    printf("Comparator result: %d\n", cmp_result);
    
    if (data != null) {
        free(data);
    }
    
    printf("Complex function pointer test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_complex_fn_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Callback called with code 42"), 
                "Callback should be called, got: {}", output);
            assert!(output.contains("Complex function pointer test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试函数指针作为结构体字段（如果支持）
#[test]
fn test_function_pointer_in_class() {
    let code = r#"
// 定义操作函数指针类型
alias Operation = fn(int, int) -> int;

extern {
    c_int printf(c_string fmt, ...);
}

// 包含函数指针的类
class Calculator {
    Operation op;
    
    public Calculator(Operation operation) {
        this.op = operation;
    }
    
    public int calculate(int a, int b) {
        return this.op(a, b);
    }
}

fn multiply(a: int, b: int) -> int {
    return a * b;
}

fn divide(a: int, b: int) -> int {
    return a / b;
}

public int main() {
    Calculator calc1 = new Calculator(multiply);
    Calculator calc2 = new Calculator(divide);
    
    int result1 = calc1.calculate(10, 5);
    int result2 = calc2.calculate(10, 5);
    
    printf("multiply(10,5) = %d\n", result1);
    printf("divide(10,5) = %d\n", result2);
    
    if (result1 == 50 && result2 == 2) {
        printf("Function pointer in class test passed!\n");
        return 0;
    }
    
    return 1;
}
"#;
    
    let temp_path = format!("tests/temp_fn_in_class_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("multiply(10,5) = 50"), 
                "Multiply should work, got: {}", output);
            assert!(output.contains("divide(10,5) = 2"), 
                "Divide should work, got: {}", output);
            assert!(output.contains("Function pointer in class test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试类型别名与数组
#[test]
fn test_type_alias_with_array() {
    let code = r#"
alias IntArray = int[];
alias Byte = c_char;

extern {
    c_int printf(c_string fmt, ...);
}

public int main() {
    // 使用类型别名创建数组
    IntArray arr = new int[5];
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    arr[3] = 40;
    arr[4] = 50;
    
    int sum = 0;
    for (int i = 0; i < 5; i = i + 1) {
        sum = sum + arr[i];
    }
    
    printf("Sum: %d\n", sum);
    
    if (sum == 150) {
        printf("Type alias with array test passed!\n");
        return 0;
    }
    
    return 1;
}
"#;
    
    let temp_path = format!("tests/temp_type_array_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Sum: 150"), 
                "Array sum should be 150, got: {}", output);
            assert!(output.contains("Type alias with array test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试类型别名作用域
#[test]
fn test_type_alias_scope() {
    let code = r#"
alias GlobalInt = int;

extern {
    c_int printf(c_string fmt, ...);
}

class MyClass {
    // 类级别的类型别名（如果支持）
    alias ClassInt = int;
    
    public ClassInt value;
    
    public MyClass(ClassInt v) {
        this.value = v;
    }
    
    public ClassInt getValue() {
        return this.value;
    }
}

fn test_function() -> void {
    // 函数级别的类型使用
    GlobalInt local = 42;
    printf("Local value: %d\n", local);
}

public int main() {
    GlobalInt g = 100;
    printf("GlobalInt: %d\n", g);
    
    MyClass obj = new MyClass(200);
    printf("ClassInt value: %d\n", obj.getValue());
    
    test_function();
    
    printf("Type alias scope test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_type_scope_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("GlobalInt: 100"), 
                "GlobalInt should work, got: {}", output);
            assert!(output.contains("Type alias scope test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试递归类型别名（如果支持）
#[test]
fn test_recursive_type_pattern() {
    let code = r#"
// 链表节点的类型别名模式
alias NodeData = ptr;
alias NodeNext = ptr;

extern {
    c_int printf(c_string fmt, ...);
    ptr malloc(size_t size);
    void free(ptr p);
}

// 简化的链表操作
fn create_node(data: int) -> ptr {
    ptr node = malloc(16);  // 简化：固定大小
    printf("Created node with data %d at %p\n", data, node);
    return node;
}

public int main() {
    NodeData node1 = create_node(10);
    NodeData node2 = create_node(20);
    NodeData node3 = create_node(30);
    
    if (node1 != null) free(node1);
    if (node2 != null) free(node2);
    if (node3 != null) free(node3);
    
    printf("Recursive type pattern test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_recursive_type_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Created node with data 10"), 
                "Node 1 should be created, got: {}", output);
            assert!(output.contains("Recursive type pattern test passed"), 
                "Test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}
