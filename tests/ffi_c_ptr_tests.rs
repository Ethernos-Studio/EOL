//! C指针和函数指针FFI测试
//!
//! 测试C指针、函数指针类型别名、void*等FFI功能

mod common;
use common::compile_and_run_eol;

/// 测试C指针基本操作
#[test]
fn test_c_pointer_basic() {
    let code = r#"
extern {
    // C标准库函数
    c_int printf(c_string fmt, ...);
    ptr malloc(size_t size);
    void free(ptr p);
}

public int main() {
    // 分配内存
    ptr p = malloc(16);
    if (p == null) {
        printf("Failed to allocate memory\n");
        return 1;
    }
    
    printf("Memory allocated successfully\n");
    free(p);
    printf("Memory freed successfully\n");
    
    return 0;
}
"#;
    
    // 写入临时文件
    let temp_path = format!("tests/temp_c_ptr_basic_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Memory allocated successfully"), 
                "Should allocate memory, got: {}", output);
            assert!(output.contains("Memory freed successfully"), 
                "Should free memory, got: {}", output);
        }
        Err(e) => {
            // 如果编译失败，可能是语法还不完全支持，记录错误
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试多级C指针
#[test]
fn test_c_pointer_multi_level() {
    let code = r#"
extern {
    c_int printf(c_string fmt, ...);
    ptr malloc(size_t size);
    void free(ptr p);
}

public int main() {
    // 分配指针数组 (ptr* 即 ptr*)
    ptr arr = malloc(8 * 3);  // 3个指针的空间
    if (arr == null) {
        printf("Failed to allocate array\n");
        return 1;
    }
    
    printf("Pointer array allocated\n");
    free(arr);
    printf("Pointer array freed\n");
    
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_c_ptr_multi_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Pointer array allocated"), 
                "Should allocate pointer array, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试void*指针
#[test]
fn test_c_void_pointer() {
    let code = r#"
extern {
    c_int printf(c_string fmt, ...);
    ptr malloc(size_t size);
    void free(ptr p);
    ptr memset(ptr s, c_int c, size_t n);
}

public int main() {
    // 使用void*进行内存操作
    ptr p = malloc(16);
    if (p == null) {
        printf("Failed to allocate\n");
        return 1;
    }
    
    // 使用memset初始化内存
    memset(p, 0, 16);
    printf("Memory initialized with memset\n");
    
    free(p);
    printf("Memory freed\n");
    
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_c_void_ptr_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Memory initialized with memset"), 
                "Should use memset with void*, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试函数指针类型别名
#[test]
fn test_function_pointer_type_alias() {
    let code = r#"
// 定义比较函数指针类型
alias CompareFn = fn(c_int, c_int) -> c_int;

extern {
    c_int printf(c_string fmt, ...);
}

// 比较函数实现
fn compare_asc(a: c_int, b: c_int) -> c_int {
    return a - b;
}

fn compare_desc(a: c_int, b: c_int) -> c_int {
    return b - a;
}

public int main() {
    // 使用类型别名声明函数指针
    CompareFn cmp_asc = compare_asc;
    CompareFn cmp_desc = compare_desc;
    
    c_int result1 = cmp_asc(5, 3);
    c_int result2 = cmp_desc(5, 3);
    
    printf("asc(5,3) = %d\n", result1);
    printf("desc(5,3) = %d\n", result2);
    
    if (result1 > 0 && result2 < 0) {
        printf("Function pointer type alias works!\n");
        return 0;
    }
    
    return 1;
}
"#;
    
    let temp_path = format!("tests/temp_fn_ptr_alias_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Function pointer type alias works"), 
                "Function pointer type alias should work, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试extern函数使用函数指针参数
#[test]
fn test_extern_function_pointer_param() {
    let code = r#"
// 定义回调函数类型
alias CallbackFn = fn(c_int) -> void;

extern {
    c_int printf(c_string fmt, ...);
}

// 模拟回调处理
fn process_with_callback(value: c_int, callback: CallbackFn) -> void {
    callback(value);
}

fn print_value(val: c_int) -> void {
    printf("Value: %d\n", val);
}

public int main() {
    process_with_callback(42, print_value);
    printf("Callback test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_extern_fn_ptr_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Value: 42"), 
                "Callback should print value, got: {}", output);
            assert!(output.contains("Callback test passed"), 
                "Callback test should pass, got: {}", output);
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
alias IntPtr = ptr;
alias IntPtrAlias = IntPtr;

extern {
    c_int printf(c_string fmt, ...);
    ptr malloc(size_t size);
    void free(ptr p);
}

public int main() {
    // 使用类型别名链
    IntPtrAlias p = malloc(8);
    if (p == null) {
        printf("Allocation failed\n");
        return 1;
    }
    
    printf("Type alias chain works!\n");
    free(p);
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_type_alias_chain_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Type alias chain works"), 
                "Type alias chain should work, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试qsort风格的函数指针使用
#[test]
fn test_qsort_style_callback() {
    let code = r#"
// 定义比较函数指针类型（类似C的qsort）
alias CompareFn = fn(ptr a, ptr b) -> c_int;

extern {
    c_int printf(c_string fmt, ...);
}

// 简单的整数比较函数
fn int_compare(a: ptr, b: ptr) -> c_int {
    // 注意：这里简化处理，实际应该解引用指针
    // 由于当前不支持直接解引用，我们只测试函数指针调用
    return 0;
}

// 模拟排序函数
fn my_sort(base: ptr, nmemb: size_t, size: size_t, cmp: CompareFn) -> void {
    printf("Sorting %zu elements of size %zu\n", nmemb, size);
    // 简化：只调用一次比较函数测试
    c_int result = cmp(base, base);
    printf("Compare result: %d\n", result);
}

public int main() {
    c_int arr[5] = {5, 2, 8, 1, 9};
    
    // 使用函数指针调用排序
    my_sort(arr, 5, 4, int_compare);
    
    printf("Qsort-style callback test passed!\n");
    return 0;
}
"#;
    
    let temp_path = format!("tests/temp_qsort_style_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("Sorting 5 elements"), 
                "Should show sorting info, got: {}", output);
            assert!(output.contains("Qsort-style callback test passed"), 
                "Qsort test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}

/// 测试函数指针作为返回值
#[test]
fn test_function_pointer_return() {
    let code = r#"
alias BinaryOp = fn(c_int, c_int) -> c_int;

extern {
    c_int printf(c_string fmt, ...);
}

fn add(a: c_int, b: c_int) -> c_int {
    return a + b;
}

fn subtract(a: c_int, b: c_int) -> c_int {
    return a - b;
}

// 返回函数指针的函数
fn get_operation(is_add: bool) -> BinaryOp {
    if (is_add) {
        return add;
    } else {
        return subtract;
    }
}

public int main() {
    BinaryOp op1 = get_operation(true);
    BinaryOp op2 = get_operation(false);
    
    c_int result1 = op1(10, 5);
    c_int result2 = op2(10, 5);
    
    printf("add(10,5) = %d\n", result1);
    printf("subtract(10,5) = %d\n", result2);
    
    if (result1 == 15 && result2 == 5) {
        printf("Function pointer return test passed!\n");
        return 0;
    }
    
    return 1;
}
"#;
    
    let temp_path = format!("tests/temp_fn_ptr_return_{}.cay", std::process::id());
    std::fs::write(&temp_path, code).expect("Failed to write temp file");
    
    let result = compile_and_run_eol(&temp_path);
    let _ = std::fs::remove_file(&temp_path);
    
    match result {
        Ok(output) => {
            assert!(output.contains("add(10,5) = 15"), 
                "Should show add result, got: {}", output);
            assert!(output.contains("subtract(10,5) = 5"), 
                "Should show subtract result, got: {}", output);
            assert!(output.contains("Function pointer return test passed"), 
                "Function pointer return test should pass, got: {}", output);
        }
        Err(e) => {
            panic!("Test failed with error: {}", e);
        }
    }
}
