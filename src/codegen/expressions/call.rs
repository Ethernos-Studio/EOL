//! 函数调用表达式代码生成
//!
//! 处理函数调用、内置函数（print/read）、String 方法调用和可变参数。

use crate::codegen::context::IRGenerator;
use crate::ast::*;
use crate::error::{cayResult, codegen_error};

impl IRGenerator {
    /// 生成函数调用表达式代码
    ///
    /// # Arguments
    /// * `call` - 函数调用表达式
    pub fn generate_call_expression(&mut self, call: &CallExpr) -> cayResult<String> {
        // 处理 print 和 println 函数
        if let Expr::Identifier(name) = call.callee.as_ref() {
            match name.as_str() {
                "print" => return self.generate_print_call(&call.args, false),
                "println" => return self.generate_print_call(&call.args, true),
                "readInt" => return self.generate_read_int_call(&call.args),
                "readLong" => return self.generate_read_long_call(&call.args),
                "readFloat" => return self.generate_read_float_call(&call.args),
                "readDouble" => return self.generate_read_double_call(&call.args),
                "readLine" => return self.generate_read_line_call(&call.args),
                "readChar" => return self.generate_read_char_call(&call.args),
                _ => {}
            }
        }

        // 处理 String 方法调用: str.method(args)
        if let Expr::MemberAccess(member) = call.callee.as_ref() {
            // 检查是否是 String 方法调用
            if let Some(method_result) = self.try_generate_string_method_call(member, &call.args)? {
                return Ok(method_result);
            }

            // 处理数组的 length() 方法调用（作为 length 属性的语法糖）
            if member.member == "length" && call.args.is_empty() {
                // 检查对象是否是数组类型
                if let Some(var_type) = self.get_expression_type(&member.object) {
                    if matches!(var_type, crate::types::Type::Array(_)) {
                        // 将 length() 转换为 length 属性访问
                        return self.generate_array_length_access(&member.object);
                    }
                }
            }
        }

        // 处理 extern 函数调用
        if let Expr::Identifier(name) = call.callee.as_ref() {
            let func_name = name.as_ref();
            if self.is_extern_function(func_name) {
                return self.generate_extern_function_call(func_name, &call.args);
            }
        }

        // 处理普通函数调用（支持方法重载和可变参数）
        // 先确定方法信息（类名和方法名）
        // 对于实例方法调用，还需要保存对象表达式以获取 this 指针
        let (class_name, method_name, obj_expr) = match call.callee.as_ref() {
            Expr::Identifier(name) => {
                let name_str = name.as_ref();
                // 检查是否是全局 extern 函数
                if let Some(_extern_func) = self.get_extern_function(name_str) {
                    return self.generate_extern_function_call(name_str, &call.args);
                }
                if !self.current_class.is_empty() {
                    (self.current_class.clone(), name_str.to_string(), None)
                } else {
                    (String::new(), name_str.to_string(), None)
                }
            }
            Expr::MemberAccess(member) => {
                // 检查 object 是否是标识符（类名或变量名）
                match member.object.as_ref() {
                    Expr::Identifier(obj_name) => {
                        let obj_name_str = obj_name.as_ref();
                        // 首先检查是否是已知的类名
                        let class_name = if let Some(ref registry) = self.type_registry {
                            if registry.class_exists(obj_name_str) {
                                obj_name_str.to_string()
                            } else {
                                // 不是类名，尝试从变量映射获取
                                self.var_class_map.get(obj_name_str)
                                    .cloned()
                                    .unwrap_or_else(|| obj_name_str.to_string())
                            }
                        } else {
                            self.var_class_map.get(obj_name_str)
                                .cloned()
                                .unwrap_or_else(|| obj_name_str.to_string())
                        };
                        (class_name, member.member.clone(), Some(member.object.clone()))
                    }
                    _ => {
                        // object 不是标识符，可能是其他表达式
                        return Err(codegen_error("Invalid method call".to_string()));
                    }
                }
            }
            _ => return Err(codegen_error("Invalid function call".to_string())),
        };

        // 检查是否是可变参数方法（根据方法名推断）
        let is_varargs_method = self.is_varargs_method(&class_name, &method_name);

        // 先生成参数以获取参数类型
        let mut arg_results = Vec::new();
        for arg in &call.args {
            arg_results.push(self.generate_expression(arg)?);
        }

        // 处理可变参数：将多余参数打包成数组
        let (processed_args, has_varargs_array) = if is_varargs_method {
            let packed = self.pack_varargs_args(&class_name, &method_name, &arg_results)?;
            // 如果原始参数多于固定参数数量，说明创建了数组
            let (fixed_count, _) = self.get_varargs_info(&class_name, &method_name);
            let has_array = arg_results.len() > fixed_count;
            (packed, has_array)
        } else {
            (arg_results, false)
        };

        // 检查是否是实例方法（需要传递 this）
        let is_instance_method = self.is_instance_method(&class_name, &method_name);
        
        // 为实例方法添加 this 参数
        let mut final_args = Vec::new();
        
        if is_instance_method {
            // 获取 this 指针
            if let Some(obj) = obj_expr {
                // 通过对象表达式获取 this 指针（如 obj1.getId()）
                let obj_result = self.generate_expression(&obj)?;
                let (_, obj_val) = self.parse_typed_value(&obj_result);
                final_args.push(format!("i8* {}", obj_val));
            } else if let Some(this_llvm_name) = self.scope_manager.get_llvm_name("this_ptr") {
                // 通过当前方法的 this_ptr 获取（如在实例方法中调用其他实例方法）
                let this_temp = self.new_temp();
                self.emit_line(&format!("  {} = load i8*, i8** %{}, align 8", 
                    this_temp, this_llvm_name));
                final_args.push(format!("i8* {}", this_temp));
            } else {
                // 在静态方法中调用实例方法且没有对象表达式，使用 null 作为 this
                final_args.push("i8* null".to_string());
            }
        }
        
        // 获取方法的参数类型信息以进行必要的类型转换
        let param_types = self.get_method_param_types(&class_name, &method_name, &processed_args, has_varargs_array);
        
        // 添加其他参数（根据需要进行类型转换）
        for (idx, arg_str) in processed_args.iter().enumerate() {
            let (arg_type, arg_val) = self.parse_typed_value(arg_str);
            
            // 检查是否需要类型转换
            if idx < param_types.len() {
                let param_llvm_type = self.type_to_llvm(&param_types[idx]);
                let converted_arg = self.convert_arg_type(&arg_type, &arg_val, &param_llvm_type);
                final_args.push(converted_arg);
            } else {
                final_args.push(arg_str.clone());
            }
        }

        // 生成函数名 - 使用类型注册表获取方法定义的参数类型
        // 注意：函数名不包含 this 参数，this 只在 IR 调用时传递
        let fn_name = self.generate_function_name(&class_name, &method_name, &processed_args, has_varargs_array);

        // 获取方法的返回类型
        let ret_type = self.get_method_return_type(&class_name, &method_name, &processed_args, has_varargs_array);
        let llvm_ret_type = self.type_to_llvm(&ret_type);
        
        if llvm_ret_type == "void" {
            // void 方法调用不需要命名结果
            self.emit_line(&format!("  call void @{}({})",
                fn_name, final_args.join(", ")));
            Ok("void %dummy".to_string())
        } else {
            let temp = self.new_temp();
            self.emit_line(&format!("  {} = call {} @{}({})",
                temp, llvm_ret_type, fn_name, final_args.join(", ")));
            Ok(format!("{} {}", llvm_ret_type, temp))
        }
    }

    /// 生成函数名 - 优先使用类型注册表中方法定义的参数类型，支持继承
    fn generate_function_name(&self, class_name: &str, method_name: &str, processed_args: &[String], has_varargs_array: bool) -> String {
        // 获取实际参数的类型签名
        let arg_types: Vec<String> = processed_args.iter()
            .enumerate()
            .map(|(idx, r)| {
                let (ty, _) = self.parse_typed_value(r);
                let is_varargs_array = has_varargs_array && idx == processed_args.len() - 1;
                let llvm_type = self.llvm_type_to_signature(&ty);
                if is_varargs_array {
                    "ai".to_string()
                } else {
                    llvm_type
                }
            })
            .collect();
        
        // 尝试从类型注册表获取方法信息（支持继承查找）
        if let Some(ref registry) = self.type_registry {
            // 首先在当前类中查找方法
            let mut current_class_name = class_name.to_string();
            loop {
                if let Some(class_info) = registry.get_class(&current_class_name) {
                    if let Some(methods) = class_info.methods.get(method_name) {
                        let arg_count = processed_args.len();
                        
                        // 首先尝试找到参数类型完全匹配的方法
                        for method in methods {
                            let param_count = method.params.len();
                            let is_varargs = method.params.last().map(|p| p.is_varargs).unwrap_or(false);
                            
                            if is_varargs {
                                // 可变参数方法
                                let fixed_count = param_count.saturating_sub(1);
                                if arg_count >= fixed_count {
                                    // 检查固定参数类型是否匹配
                                    let method_sig = self.build_function_name_from_method(&current_class_name, method_name, &method.params, has_varargs_array);
                                    let expected_sig = format!("{}.__{}_{}", current_class_name, method_name, arg_types.join("_"));
                                    if method_sig == expected_sig {
                                        return method_sig;
                                    }
                                }
                            } else if param_count == arg_count {
                                // 非可变参数方法：检查参数类型是否匹配
                                let method_sig = self.build_function_name_from_method(&current_class_name, method_name, &method.params, has_varargs_array);
                                let expected_sig = format!("{}.__{}_{}", current_class_name, method_name, arg_types.join("_"));
                                if method_sig == expected_sig {
                                    return method_sig;
                                }
                            }
                        }
                        
                        // 如果没有找到类型完全匹配的方法，回退到参数数量匹配
                        for method in methods {
                            let param_count = method.params.len();
                            let is_varargs = method.params.last().map(|p| p.is_varargs).unwrap_or(false);
                            
                            if is_varargs {
                                let fixed_count = param_count.saturating_sub(1);
                                if arg_count >= fixed_count {
                                    return self.build_function_name_from_method(&current_class_name, method_name, &method.params, has_varargs_array);
                                }
                            } else if param_count == arg_count {
                                return self.build_function_name_from_method(&current_class_name, method_name, &method.params, has_varargs_array);
                            }
                        }
                    }
                    
                    // 如果在当前类中没找到，尝试在父类中查找
                    if let Some(ref parent_name) = class_info.parent {
                        current_class_name = parent_name.clone();
                        continue;
                    }
                }
                break;
            }
        }

        // 回退到使用实际参数类型生成函数名
        if arg_types.is_empty() {
            format!("{}.{}", class_name, method_name)
        } else {
            format!("{}.__{}_{}", class_name, method_name, arg_types.join("_"))
        }
    }

    /// 根据方法定义的参数类型构建函数名
    fn build_function_name_from_method(&self, class_name: &str, method_name: &str, params: &[crate::types::ParameterInfo], has_varargs_array: bool) -> String {
        if params.is_empty() {
            return format!("{}.{}", class_name, method_name);
        }

        let param_types: Vec<String> = params.iter()
            .enumerate()
            .map(|(idx, p)| {
                let is_last_varargs = has_varargs_array && idx == params.len() - 1 && p.is_varargs;
                self.param_type_to_signature(&p.param_type, is_last_varargs)
            })
            .collect();

        format!("{}.__{}_{}", class_name, method_name, param_types.join("_"))
    }

    /// 将参数类型转换为签名
    fn param_type_to_signature(&self, ty: &crate::types::Type, is_varargs_array: bool) -> String {
        if is_varargs_array {
            // 可变参数数组：提取元素类型并生成签名
            return self.varargs_element_type_to_signature(ty);
        }

        match ty {
            crate::types::Type::Int32 => "i".to_string(),
            crate::types::Type::Int64 => "l".to_string(),
            crate::types::Type::Float32 => "f".to_string(),
            crate::types::Type::Float64 => "d".to_string(),
            crate::types::Type::Bool => "b".to_string(),
            crate::types::Type::String => "s".to_string(),
            crate::types::Type::Char => "c".to_string(),
            crate::types::Type::Object(name) => format!("o{}", name),
            crate::types::Type::Array(inner) => format!("a{}", self.param_type_to_signature(inner, false)),
            _ => "x".to_string(),
        }
    }

    /// 将可变参数数组的元素类型转换为签名
    /// 可变参数类型是 Array(ElementType)，需要提取元素类型
    fn varargs_element_type_to_signature(&self, ty: &crate::types::Type) -> String {
        use crate::types::Type;
        match ty {
            Type::Array(elem) => {
                match elem.as_ref() {
                    Type::Int32 => "ai".to_string(),
                    Type::Int64 => "al".to_string(),
                    Type::Float32 => "af".to_string(),
                    Type::Float64 => "ad".to_string(),
                    Type::Bool => "ab".to_string(),
                    Type::String => "as".to_string(),
                    Type::Char => "ac".to_string(),
                    Type::Object(name) => format!("ao{}", name),
                    _ => "ax".to_string(),
                }
            }
            _ => self.param_type_to_signature(ty, false), // 如果不是数组类型，回退到普通签名
        }
    }

    /// 获取方法的返回类型
    fn get_method_return_type(&self, class_name: &str, method_name: &str, processed_args: &[String], has_varargs_array: bool) -> crate::types::Type {
        // 获取实际参数的类型签名
        let arg_types: Vec<String> = processed_args.iter()
            .enumerate()
            .map(|(idx, r)| {
                let (ty, _) = self.parse_typed_value(r);
                let is_varargs_array = has_varargs_array && idx == processed_args.len() - 1;
                let llvm_type = self.llvm_type_to_signature(&ty);
                if is_varargs_array {
                    "ai".to_string()
                } else {
                    llvm_type
                }
            })
            .collect();
        
        if let Some(ref registry) = self.type_registry {
            if let Some(class_info) = registry.get_class(class_name) {
                if let Some(methods) = class_info.methods.get(method_name) {
                    let arg_count = processed_args.len();
                    
                    // 首先尝试找到参数类型完全匹配的方法
                    for method in methods {
                        let param_count = method.params.len();
                        let is_varargs = method.params.last().map(|p| p.is_varargs).unwrap_or(false);
                        
                        if is_varargs {
                            let fixed_count = param_count.saturating_sub(1);
                            if arg_count >= fixed_count {
                                let method_sig = self.build_function_name_from_method(class_name, method_name, &method.params, has_varargs_array);
                                let expected_sig = format!("{}.__{}_{}", class_name, method_name, arg_types.join("_"));
                                if method_sig == expected_sig {
                                    return method.return_type.clone();
                                }
                            }
                        } else if param_count == arg_count {
                            let method_sig = self.build_function_name_from_method(class_name, method_name, &method.params, has_varargs_array);
                            let expected_sig = format!("{}.__{}_{}", class_name, method_name, arg_types.join("_"));
                            if method_sig == expected_sig {
                                return method.return_type.clone();
                            }
                        }
                    }
                    
                    // 如果没有找到类型完全匹配的方法，回退到参数数量匹配
                    for method in methods {
                        let param_count = method.params.len();
                        let is_varargs = method.params.last().map(|p| p.is_varargs).unwrap_or(false);
                        
                        if is_varargs {
                            let fixed_count = param_count.saturating_sub(1);
                            if arg_count >= fixed_count {
                                return method.return_type.clone();
                            }
                        } else if param_count == arg_count {
                            return method.return_type.clone();
                        }
                    }
                }
            }
        }
        
        // 默认返回 i64 类型
        crate::types::Type::Int64
    }

    /// 检查方法是否是可变参数方法
    /// 查询类型注册表来确定方法是否真的是可变参数方法
    fn is_varargs_method(&self, class_name: &str, method_name: &str) -> bool {
        // 查询类型注册表
        if let Some(ref registry) = self.type_registry {
            if let Some(class_info) = registry.get_class(class_name) {
                if let Some(methods) = class_info.methods.get(method_name) {
                    // 检查是否有任何方法是可变参数的
                    for method in methods {
                        if method.params.last().map(|p| p.is_varargs).unwrap_or(false) {
                            return true;
                        }
                    }
                }
            }
        }
        // 默认返回false，避免将普通方法误认为可变参数方法
        false
    }

    /// 检查方法是否是实例方法（非静态方法）
    fn is_instance_method(&self, class_name: &str, method_name: &str) -> bool {
        // 查询类型注册表
        if let Some(ref registry) = self.type_registry {
            if let Some(class_info) = registry.get_class(class_name) {
                if let Some(methods) = class_info.methods.get(method_name) {
                    // 检查是否有任何方法是实例方法（非静态）
                    for method in methods {
                        if !method.is_static {
                            return true;
                        }
                    }
                }
            }
        }
        // 默认返回false
        false
    }

    /// 将可变参数打包成数组
    /// fixed_param_count: 固定参数的数量
    fn pack_varargs_args(&mut self, class_name: &str, method_name: &str, arg_results: &[String]) -> cayResult<Vec<String>> {
        // 从类型注册表获取固定参数数量和可变参数元素类型
        let (fixed_param_count, varargs_elem_type) = self.get_varargs_info(class_name, method_name);

        if arg_results.len() <= fixed_param_count {
            // 参数数量不足或刚好，不需要打包
            return Ok(arg_results.to_vec());
        }

        // 分割固定参数和可变参数
        let fixed_args = &arg_results[..fixed_param_count];
        let varargs = &arg_results[fixed_param_count..];

        // 检查是否只有一个参数且是数组类型（直接传递数组给可变参数）
        if varargs.len() == 1 {
            let (arg_type, arg_val) = self.parse_typed_value(&varargs[0]);
            // 检查参数类型是否是数组指针（以*结尾但不是i8*）
            if arg_type.ends_with("*") && arg_type != "i8*" {
                // 直接将数组指针作为可变参数传递
                let mut result = fixed_args.to_vec();
                result.push(format!("i8* {}", arg_val));
                return Ok(result);
            }
        }

        // 创建数组来存储可变参数
        let array_size = varargs.len();
        let raw_ptr = self.new_temp();
        let array_ptr = self.new_temp();

        // 根据元素类型确定 LLVM 类型和大小
        let (llvm_elem_type, elem_size) = match varargs_elem_type {
            crate::types::Type::Int32 => ("i32", 4),
            crate::types::Type::Int64 => ("i64", 8),
            crate::types::Type::Float32 => ("float", 4),
            crate::types::Type::Float64 => ("double", 8),
            crate::types::Type::String => ("i8", 8), // String 是指针类型
            crate::types::Type::Char => ("i8", 1),
            crate::types::Type::Bool => ("i8", 1),
            _ => ("i32", 4), // 默认使用 i32
        };

        // 分配数组内存：8字节（长度+padding）+ 元素数据
        let header_size = 8;
        let data_size = array_size * elem_size;
        let total_size = header_size + data_size;
        self.emit_line(&format!("  {} = call i8* @calloc(i64 1, i64 {})", raw_ptr, total_size));

        // 存储长度信息到前4字节
        let len_ptr_i8 = self.new_temp();
        let len_ptr = self.new_temp();
        self.emit_line(&format!("  {} = getelementptr i8, i8* {}, i64 0", len_ptr_i8, raw_ptr));
        self.emit_line(&format!("  {} = bitcast i8* {} to i32*", len_ptr, len_ptr_i8));
        self.emit_line(&format!("  store i32 {}, i32* {}, align 4", array_size, len_ptr));

        // 计算数组元素起始地址（跳过8字节头部）
        self.emit_line(&format!("  {} = getelementptr i8, i8* {}, i64 {}", array_ptr, raw_ptr, header_size));

        // 将可变参数存入数组
        for (i, arg_str) in varargs.iter().enumerate() {
            let (arg_type, arg_val) = self.parse_typed_value(arg_str);
            let elem_ptr_i8 = self.new_temp();
            let offset = i * elem_size;

            // 计算元素地址 (i8*)
            self.emit_line(&format!("  {} = getelementptr i8, i8* {}, i64 {}", elem_ptr_i8, array_ptr, offset));

            // 根据元素类型进行存储
            self.store_vararg_element(&elem_ptr_i8, &arg_type, &arg_val, llvm_elem_type);
        }

        // 构建结果：固定参数 + 数组指针（指向元素0，与正常数组布局一致）
        let mut result = fixed_args.to_vec();
        result.push(format!("i8* {}", array_ptr));

        Ok(result)
    }

    /// 获取可变参数方法的固定参数数量和元素类型
    fn get_varargs_info(&self, class_name: &str, method_name: &str) -> (usize, crate::types::Type) {
        if let Some(ref registry) = self.type_registry {
            if let Some(class_info) = registry.get_class(class_name) {
                if let Some(methods) = class_info.methods.get(method_name) {
                    for method in methods {
                        if let Some(last_param) = method.params.last() {
                            if last_param.is_varargs {
                                // 可变参数数量 = 总参数数量 - 1（最后一个可变参数）
                                let fixed_count = method.params.len() - 1;
                                // 获取可变参数的元素类型
                                let elem_type = match &last_param.param_type {
                                    crate::types::Type::Array(elem) => elem.as_ref().clone(),
                                    _ => last_param.param_type.clone(),
                                };
                                return (fixed_count, elem_type);
                            }
                        }
                    }
                }
            }
        }
        // 默认值：没有固定参数，元素类型为 Int32
        (0, crate::types::Type::Int32)
    }

    /// 存储可变参数元素到数组
    fn store_vararg_element(&mut self, elem_ptr_i8: &str, arg_type: &str, arg_val: &str, llvm_elem_type: &str) {
        match llvm_elem_type {
            "i32" => {
                let elem_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to i32*", elem_ptr, elem_ptr_i8));
                if arg_type == "i64" {
                    let truncated = self.new_temp();
                    self.emit_line(&format!("  {} = trunc i64 {} to i32", truncated, arg_val));
                    self.emit_line(&format!("  store i32 {}, i32* {}, align 4", truncated, elem_ptr));
                } else if arg_type == "i32" {
                    self.emit_line(&format!("  store i32 {}, i32* {}, align 4", arg_val, elem_ptr));
                }
            }
            "i64" => {
                let elem_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to i64*", elem_ptr, elem_ptr_i8));
                if arg_type == "i32" {
                    let extended = self.new_temp();
                    self.emit_line(&format!("  {} = sext i32 {} to i64", extended, arg_val));
                    self.emit_line(&format!("  store i64 {}, i64* {}, align 8", extended, elem_ptr));
                } else {
                    self.emit_line(&format!("  store i64 {}, i64* {}, align 8", arg_val, elem_ptr));
                }
            }
            "float" => {
                let elem_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to float*", elem_ptr, elem_ptr_i8));
                // 如果参数是 double 类型，需要转换为 float
                if arg_type == "double" {
                    let converted = self.new_temp();
                    self.emit_line(&format!("  {} = fptrunc double {} to float", converted, arg_val));
                    self.emit_line(&format!("  store float {}, float* {}, align 4", converted, elem_ptr));
                } else {
                    self.emit_line(&format!("  store float {}, float* {}, align 4", arg_val, elem_ptr));
                }
            }
            "double" => {
                let elem_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to double*", elem_ptr, elem_ptr_i8));
                self.emit_line(&format!("  store double {}, double* {}, align 8", arg_val, elem_ptr));
            }
            "i8" => {
                // 用于 String (i8*), char, bool
                let elem_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to i8**", elem_ptr, elem_ptr_i8));
                self.emit_line(&format!("  store i8* {}, i8** {}, align 8", arg_val, elem_ptr));
            }
            _ => {
                // 默认处理为 i32
                let elem_ptr = self.new_temp();
                self.emit_line(&format!("  {} = bitcast i8* {} to i32*", elem_ptr, elem_ptr_i8));
                self.emit_line(&format!("  store i32 {}, i32* {}, align 4", arg_val, elem_ptr));
            }
        }
    }

    /// 获取方法的参数类型列表
    fn get_method_param_types(&self, class_name: &str, method_name: &str, processed_args: &[String], has_varargs_array: bool) -> Vec<crate::types::Type> {
        if let Some(ref registry) = self.type_registry {
            // 在类及其父类中查找方法
            let mut current_class_name = class_name.to_string();
            loop {
                if let Some(class_info) = registry.get_class(&current_class_name) {
                    if let Some(methods) = class_info.methods.get(method_name) {
                        let arg_count = processed_args.len();
                        
                        for method in methods {
                            let param_count = method.params.len();
                            let is_varargs = method.params.last().map(|p| p.is_varargs).unwrap_or(false);
                            
                            if is_varargs {
                                let fixed_count = param_count.saturating_sub(1);
                                if arg_count >= fixed_count {
                                    // 返回固定参数类型（不包括可变参数数组）
                                    return method.params.iter()
                                        .take(fixed_count)
                                        .map(|p| p.param_type.clone())
                                        .collect();
                                }
                            } else if param_count == arg_count {
                                return method.params.iter()
                                    .map(|p| p.param_type.clone())
                                    .collect();
                            }
                        }
                    }
                    
                    // 在父类中查找
                    if let Some(parent) = &class_info.parent {
                        current_class_name = parent.clone();
                        continue;
                    }
                }
                break;
            }
        }
        Vec::new()
    }

    /// 转换参数类型以匹配形参类型
    fn convert_arg_type(&mut self, arg_type: &str, arg_val: &str, param_llvm_type: &str) -> String {
        // 如果类型已经匹配，直接返回
        if arg_type == param_llvm_type {
            return format!("{} {}", arg_type, arg_val);
        }
        
        // double -> float 转换
        if arg_type == "double" && param_llvm_type == "float" {
            let converted = self.new_temp();
            self.emit_line(&format!("  {} = fptrunc double {} to float", converted, arg_val));
            return format!("float {}", converted);
        }
        
        // float -> double 转换
        if arg_type == "float" && param_llvm_type == "double" {
            let converted = self.new_temp();
            self.emit_line(&format!("  {} = fpext float {} to double", converted, arg_val));
            return format!("double {}", converted);
        }
        
        // i32 -> i64 转换
        if arg_type == "i32" && param_llvm_type == "i64" {
            let converted = self.new_temp();
            self.emit_line(&format!("  {} = sext i32 {} to i64", converted, arg_val));
            return format!("i64 {}", converted);
        }
        
        // i64 -> i32 截断
        if arg_type == "i64" && param_llvm_type == "i32" {
            let converted = self.new_temp();
            self.emit_line(&format!("  {} = trunc i64 {} to i32", converted, arg_val));
            return format!("i32 {}", converted);
        }
        
        // 默认：不进行转换
        format!("{} {}", arg_type, arg_val)
    }

    /// 生成 extern 函数调用
    ///
    /// # Arguments
    /// * `func_name` - extern 函数名称
    /// * `args` - 函数参数
    fn generate_extern_function_call(&mut self, func_name: &str, args: &[Expr]) -> cayResult<String> {
        // 获取 extern 函数信息（克隆以避免借用问题）
        let extern_func = {
            let found = self.get_extern_function(func_name);
            match found {
                Some(f) => f.clone(),
                None => return Err(codegen_error(format!("Extern function '{}' not found", func_name))),
            }
        };

        // 生成参数
        let mut arg_results = Vec::new();
        for arg in args {
            arg_results.push(self.generate_expression(arg)?);
        }

        // 获取参数类型和值
        let mut processed_args = Vec::new();
        for (idx, arg_str) in arg_results.iter().enumerate() {
            let (arg_type, arg_val) = self.parse_typed_value(arg_str);
            
            // 获取参数的期望类型（从 extern 函数声明中）
            if idx < extern_func.params.len() {
                let param_type = &extern_func.params[idx].param_type;
                let llvm_param_type = self.type_to_llvm(param_type);
                
                // 进行类型转换
                let converted_arg = self.convert_arg_type(&arg_type, &arg_val, &llvm_param_type);
                processed_args.push(converted_arg);
            } else {
                // 如果参数数量超过声明中的数量，直接传递
                processed_args.push(arg_str.clone());
            }
        }

        // 获取返回类型
        let llvm_ret_type = self.type_to_llvm(&extern_func.return_type);

        // 直接调用 extern 函数（不创建包装函数）
        if llvm_ret_type == "void" {
            self.emit_line(&format!("  call void @{}({})",
                func_name, processed_args.join(", ")));
            Ok("void %dummy".to_string())
        } else {
            let temp = self.new_temp();
            self.emit_line(&format!("  {} = call {} @{}({})",
                temp, llvm_ret_type, func_name, processed_args.join(", ")));
            Ok(format!("{} {}", llvm_ret_type, temp))
        }
    }
}
