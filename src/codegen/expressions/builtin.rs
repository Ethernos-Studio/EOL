//! 内置函数调用代码生成
//!
//! 处理 print/println/readInt/readFloat/readLine 等内置函数。

use crate::codegen::context::IRGenerator;
use crate::ast::*;
use crate::error::{cayResult, codegen_error};

impl IRGenerator {
    /// 生成 print/println 调用代码
    ///
    /// 支持两种调用方式：
    /// 1. 单参数：print("Hello") 或 println(123)
    /// 2. Format 字符串：print("Value: %d", value) 或 println("Name: %s, Age: %d", name, age)
    ///
    /// 支持的格式说明符：
    /// - %d, %i: 整数 (int/long)
    /// - %f: 浮点数 (float/double)
    /// - %s: 字符串
    /// - %%: 字面量 %
    ///
    /// # Arguments
    /// * `args` - 参数列表
    /// * `newline` - 是否打印换行符
    pub fn generate_print_call(&mut self, args: &[Expr], newline: bool) -> cayResult<String> {
        if args.is_empty() {
            // 无参数，仅打印换行符（如果是 println）或什么都不做（如果是 print）
            if newline {
                let fmt_str = "\n";
                let fmt_name = self.get_or_create_string_constant(fmt_str);
                let fmt_len = fmt_str.len() + 1;
                let fmt_ptr = self.new_temp();
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));
                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {})", fmt_ptr));
            }
            return Ok("void".to_string());
        }

        // 如果只有一个参数，使用原有的简单处理方式
        if args.len() == 1 {
            return self.generate_simple_print(&args[0], newline);
        }

        // 多个参数：第一个参数是 format 字符串
        self.generate_format_print(args, newline)
    }

    /// 生成简单的单参数打印（保持向后兼容）
    fn generate_simple_print(&mut self, arg: &Expr, newline: bool) -> cayResult<String> {
        match arg {
            Expr::Literal(LiteralValue::String(s)) => {
                let global_name = self.get_or_create_string_constant(s);
                let fmt_str = if newline { "%s\n" } else { "%s" };
                let fmt_name = self.get_or_create_string_constant(fmt_str);
                let len = s.len() + 1;
                let fmt_len = fmt_str.len() + 1;

                let str_ptr = self.new_temp();
                let fmt_ptr = self.new_temp();

                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    str_ptr, len, len, global_name));
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));

                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i8* {})",
                    fmt_ptr, str_ptr));
            }
            Expr::Literal(LiteralValue::Int32(_)) | Expr::Literal(LiteralValue::Int64(_)) => {
                let value = self.generate_expression(arg)?;
                let (type_str, val) = self.parse_typed_value(&value);
                let i64_fmt = self.get_i64_format_specifier();
                let fmt_str = if newline { format!("{}\n", i64_fmt) } else { i64_fmt.to_string() };
                let fmt_name = self.get_or_create_string_constant(&fmt_str);
                let fmt_len = fmt_str.len() + 1;

                let fmt_ptr = self.new_temp();
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));

                let final_val = if type_str != "i64" {
                    let ext_temp = self.new_temp();
                    self.emit_line(&format!("  {} = sext {} {} to i64", ext_temp, type_str, val));
                    ext_temp
                } else {
                    val.to_string()
                };

                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})",
                    fmt_ptr, final_val));
            }
            _ => {
                let value = self.generate_expression(arg)?;
                let (type_str, val) = self.parse_typed_value(&value);

                if type_str == "i8*" {
                    let fmt_str = if newline { "%s\n" } else { "%s" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i8* {})",
                        fmt_ptr, val));
                } else if type_str.starts_with("i") && type_str != "i8*" {
                    let i64_fmt = self.get_i64_format_specifier();
                    let fmt_str = if newline { format!("{}\n", i64_fmt) } else { i64_fmt.to_string() };
                    let fmt_name = self.get_or_create_string_constant(&fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));

                    let final_val = if type_str != "i64" {
                        let ext_temp = self.new_temp();
                        self.emit_line(&format!("  {} = sext {} {} to i64", ext_temp, type_str, val));
                        ext_temp
                    } else {
                        val.to_string()
                    };

                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})",
                        fmt_ptr, final_val));
                } else if type_str == "double" || type_str == "float" {
                    let fmt_str = if newline { "%f\n" } else { "%f" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));

                    let final_val = if type_str == "float" {
                        let ext_temp = self.new_temp();
                        self.emit_line(&format!("  {} = fpext float {} to double", ext_temp, val));
                        ext_temp
                    } else {
                        val.to_string()
                    };

                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, double {})",
                        fmt_ptr, final_val));
                } else {
                    let fmt_str = if newline { "%s\n" } else { "%s" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, {})",
                        fmt_ptr, value));
                }
            }
        }

        Ok("i64 0".to_string())
    }

    /// 生成 format 字符串打印（支持多个参数）
    fn generate_format_print(&mut self, args: &[Expr], newline: bool) -> cayResult<String> {
        // 第一个参数必须是 format 字符串
        let format_arg = &args[0];
        let format_str = match format_arg {
            Expr::Literal(LiteralValue::String(s)) => s.clone(),
            _ => {
                // 如果第一个参数不是字符串字面量，回退到简单打印第一个参数
                return self.generate_simple_print(format_arg, newline);
            }
        };

        // 解析 format 字符串，提取格式说明符
        let format_specs = self.parse_format_string(&format_str);

        // 检查参数数量是否匹配
        if format_specs.len() != args.len() - 1 {
            return Err(codegen_error(format!(
                "Format string expects {} arguments, but {} provided",
                format_specs.len(),
                args.len() - 1
            )));
        }

        // 构建最终的 format 字符串（添加换行符如果需要）
        let final_fmt_str = if newline {
            format_str + "\n"
        } else {
            format_str.clone()
        };

        let fmt_name = self.get_or_create_string_constant(&final_fmt_str);
        let fmt_len = final_fmt_str.len() + 1;
        let fmt_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            fmt_ptr, fmt_len, fmt_len, fmt_name));

        // 生成每个参数的值
        let mut arg_values: Vec<(String, String)> = Vec::new(); // (类型, 值)

        for i in 1..args.len() {
            let arg = &args[i];
            let value = self.generate_expression(arg)?;
            let (type_str, val) = self.parse_typed_value(&value);

            // 根据格式说明符进行类型转换
            let spec = &format_specs[i - 1];
            let (final_type, final_val) = self.convert_for_format(&type_str, &val, spec);

            arg_values.push((final_type, final_val));
        }

        // 构建 printf 调用
        let mut call_args = vec![format!("i8* {}", fmt_ptr)];
        for (typ, val) in &arg_values {
            call_args.push(format!("{} {}", typ, val));
        }

        self.emit_line(&format!("  call i32 (i8*, ...) @printf({})",
            call_args.join(", ")));

        Ok("i64 0".to_string())
    }

    /// 解析 format 字符串，提取格式说明符
    /// 返回格式说明符列表，如 ["%d", "%s", "%f"]
    fn parse_format_string(&self, fmt: &str) -> Vec<String> {
        let mut specs = Vec::new();
        let mut chars = fmt.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(&next) = chars.peek() {
                    if next == '%' {
                        // %% - 转义的字面量 %
                        chars.next();
                    } else {
                        // 格式说明符
                        let mut spec = String::from("%");

                        // 收集格式说明符的其余部分（宽度、精度、长度修饰符、转换说明符）
                        while let Some(&ch) = chars.peek() {
                            if ch.is_ascii_alphabetic() || ch == '*' {
                                // 转换说明符（如 d, s, f, x 等）
                                spec.push(ch);
                                chars.next();
                                break;
                            } else {
                                // 宽度、精度、长度修饰符等
                                spec.push(ch);
                                chars.next();
                            }
                        }

                        specs.push(spec);
                    }
                }
            }
        }

        specs
    }

    /// 根据格式说明符转换值类型
    fn convert_for_format(&mut self, type_str: &str, val: &str, spec: &str) -> (String, String) {
        match spec {
            "%d" | "%i" => {
                // 整数格式 - 转换为 i64
                if type_str == "i64" {
                    ("i64".to_string(), val.to_string())
                } else if type_str.starts_with("i") {
                    let ext_temp = self.new_temp();
                    self.emit_line(&format!("  {} = sext {} {} to i64", ext_temp, type_str, val));
                    ("i64".to_string(), ext_temp)
                } else {
                    // 其他类型，尝试作为 i64
                    ("i64".to_string(), val.to_string())
                }
            }
            "%f" | "%e" | "%g" | "%E" | "%G" => {
                // 浮点格式 - 转换为 double
                if type_str == "double" {
                    ("double".to_string(), val.to_string())
                } else if type_str == "float" {
                    let ext_temp = self.new_temp();
                    self.emit_line(&format!("  {} = fpext float {} to double", ext_temp, val));
                    ("double".to_string(), ext_temp)
                } else {
                    // 其他类型，尝试作为 double
                    ("double".to_string(), val.to_string())
                }
            }
            "%s" => {
                // 字符串格式 - 必须是 i8*
                if type_str == "i8*" {
                    ("i8*".to_string(), val.to_string())
                } else {
                    // 尝试转换为字符串（这里简化处理，实际应该调用 toString 方法）
                    (type_str.to_string(), val.to_string())
                }
            }
            "%c" => {
                // 字符格式 - 转换为 i32
                if type_str == "i32" {
                    ("i32".to_string(), val.to_string())
                } else if type_str == "i8" {
                    let ext_temp = self.new_temp();
                    self.emit_line(&format!("  {} = sext i8 {} to i32", ext_temp, val));
                    ("i32".to_string(), ext_temp)
                } else {
                    ("i32".to_string(), val.to_string())
                }
            }
            "%x" | "%X" | "%o" | "%u" => {
                // 无符号整数 - 转换为 i64
                if type_str == "i64" {
                    ("i64".to_string(), val.to_string())
                } else if type_str.starts_with("i") {
                    let ext_temp = self.new_temp();
                    self.emit_line(&format!("  {} = sext {} {} to i64", ext_temp, type_str, val));
                    ("i64".to_string(), ext_temp)
                } else {
                    ("i64".to_string(), val.to_string())
                }
            }
            "%p" => {
                // 指针格式
                (type_str.to_string(), val.to_string())
            }
            _ => {
                // 未知的格式说明符，使用原类型
                (type_str.to_string(), val.to_string())
            }
        }
    }

    /// 生成 readInt 调用代码
    ///
    /// # Arguments
    /// * `args` - 参数列表（应该为空）
    pub fn generate_read_int_call(&mut self, args: &[Expr]) -> cayResult<String> {
        // readInt 应该没有参数
        if !args.is_empty() {
            return Err(codegen_error("readInt() takes no arguments".to_string()));
        }

        // 为输入缓冲区分配空间
        let buffer_size = 32; // 足够存储整数
        let buffer_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca [{} x i8], align 1", buffer_temp, buffer_size));

        // 获取缓冲区指针
        let buffer_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            buffer_ptr, buffer_size, buffer_size, buffer_temp));

        // 调用 scanf 读取整数
        let fmt_str = self.get_i64_format_specifier();
        let fmt_name = self.get_or_create_string_constant(fmt_str);
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            fmt_ptr, fmt_len, fmt_len, fmt_name));

        // 为整数结果分配空间
        let int_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca i64, align 8", int_temp));

        // 调用 scanf
        self.emit_line(&format!("  call i32 (i8*, ...) @scanf(i8* {}, i64* {})",
            fmt_ptr, int_temp));

        // 加载读取的整数值
        let result_temp = self.new_temp();
        self.emit_line(&format!("  {} = load i64, i64* {}, align 8", result_temp, int_temp));

        Ok(format!("i64 {}", result_temp))
    }

    /// 生成 readFloat 调用代码
    ///
    /// # Arguments
    /// * `args` - 参数列表（应该为空）
    pub fn generate_read_float_call(&mut self, args: &[Expr]) -> cayResult<String> {
        // readFloat 应该没有参数
        if !args.is_empty() {
            return Err(codegen_error("readFloat() takes no arguments".to_string()));
        }

        // 为输入缓冲区分配空间
        let buffer_size = 64;
        let buffer_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca [{} x i8], align 1", buffer_temp, buffer_size));

        // 获取缓冲区指针
        let buffer_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            buffer_ptr, buffer_size, buffer_size, buffer_temp));

        // 调用 scanf 读取浮点数
        let fmt_str = "%f";
        let fmt_name = self.get_or_create_string_constant(fmt_str);
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            fmt_ptr, fmt_len, fmt_len, fmt_name));

        // 为浮点数结果分配空间
        let float_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca float, align 4", float_temp));

        // 调用 scanf
        self.emit_line(&format!("  call i32 (i8*, ...) @scanf(i8* {}, float* {})",
            fmt_ptr, float_temp));

        // 加载读取的浮点数值
        let result_temp = self.new_temp();
        self.emit_line(&format!("  {} = load float, float* {}, align 4", result_temp, float_temp));

        Ok(format!("float {}", result_temp))
    }

    /// 生成 readDouble 调用代码
    ///
    /// # Arguments
    /// * `args` - 参数列表（应该为空）
    pub fn generate_read_double_call(&mut self, args: &[Expr]) -> cayResult<String> {
        // readDouble 应该没有参数
        if !args.is_empty() {
            return Err(codegen_error("readDouble() takes no arguments".to_string()));
        }

        // 为输入缓冲区分配空间
        let buffer_size = 64;
        let buffer_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca [{} x i8], align 1", buffer_temp, buffer_size));

        // 获取缓冲区指针
        let buffer_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            buffer_ptr, buffer_size, buffer_size, buffer_temp));

        // 调用 scanf 读取双精度浮点数
        let fmt_str = "%lf";
        let fmt_name = self.get_or_create_string_constant(fmt_str);
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            fmt_ptr, fmt_len, fmt_len, fmt_name));

        // 为双精度浮点数结果分配空间
        let double_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca double, align 8", double_temp));

        // 调用 scanf
        self.emit_line(&format!("  call i32 (i8*, ...) @scanf(i8* {}, double* {})",
            fmt_ptr, double_temp));

        // 加载读取的双精度浮点数值
        let result_temp = self.new_temp();
        self.emit_line(&format!("  {} = load double, double* {}, align 8", result_temp, double_temp));

        Ok(format!("double {}", result_temp))
    }

    /// 生成 readLong 调用代码
    ///
    /// # Arguments
    /// * `args` - 参数列表（应该为空）
    pub fn generate_read_long_call(&mut self, args: &[Expr]) -> cayResult<String> {
        // readLong 与 readInt 相同，都返回 i64
        self.generate_read_int_call(args)
    }

    /// 生成 readChar 调用代码
    ///
    /// # Arguments
    /// * `args` - 参数列表（应该为空）
    pub fn generate_read_char_call(&mut self, args: &[Expr]) -> cayResult<String> {
        // readChar 应该没有参数
        if !args.is_empty() {
            return Err(codegen_error("readChar() takes no arguments".to_string()));
        }

        // 为输入缓冲区分配空间
        let buffer_size = 8;
        let buffer_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca [{} x i8], align 1", buffer_temp, buffer_size));

        // 获取缓冲区指针
        let buffer_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            buffer_ptr, buffer_size, buffer_size, buffer_temp));

        // 调用 scanf 读取字符
        let fmt_str = " %c";  // 空格跳过空白字符
        let fmt_name = self.get_or_create_string_constant(fmt_str);
        let fmt_len = fmt_str.len() + 1;
        let fmt_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            fmt_ptr, fmt_len, fmt_len, fmt_name));

        // 为字符结果分配空间
        let char_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca i8, align 1", char_temp));

        // 调用 scanf
        self.emit_line(&format!("  call i32 (i8*, ...) @scanf(i8* {}, i8* {})",
            fmt_ptr, char_temp));

        // 加载读取的字符值
        let result_temp = self.new_temp();
        self.emit_line(&format!("  {} = load i8, i8* {}, align 1", result_temp, char_temp));

        Ok(format!("i8 {}", result_temp))
    }

    /// 生成 readLine 调用代码
    ///
    /// # Arguments
    /// * `args` - 参数列表（应该为空）
    pub fn generate_read_line_call(&mut self, args: &[Expr]) -> cayResult<String> {
        // readLine 应该没有参数
        if !args.is_empty() {
            return Err(codegen_error("readLine() takes no arguments".to_string()));
        }

        // 分配缓冲区
        let buffer_size = 1024;
        let buffer_temp = self.new_temp();
        self.emit_line(&format!("  {} = alloca [{} x i8], align 1", buffer_temp, buffer_size));

        // 获取缓冲区指针
        let buffer_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
            buffer_ptr, buffer_size, buffer_size, buffer_temp));

        // 获取 stdin
        let stdin_ptr = self.new_temp();
        if self.is_windows_target() {
            // Windows: 使用 __acrt_iob_func(0) 获取 stdin
            self.emit_line(&format!("  {} = call i8* @__acrt_iob_func(i32 0)", stdin_ptr));
        } else {
            // Linux/macOS: stdin 是外部全局变量
            self.emit_line(&format!("  {} = load i8*, i8** @stdin, align 8", stdin_ptr));
        }

        // 调用 fgets
        self.emit_line(&format!("  call i8* @fgets(i8* {}, i32 {}, i8* {})",
            buffer_ptr, buffer_size, stdin_ptr));

        // 返回缓冲区指针
        Ok(format!("i8* {}", buffer_ptr))
    }
}
