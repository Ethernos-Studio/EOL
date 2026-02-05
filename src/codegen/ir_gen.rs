use std::collections::HashMap;
use crate::ast::*;
use crate::types::Type;
use crate::error::{EolResult, EolError, codegen_error};

pub struct IRGenerator {
    output: String,
    indent: usize,
    label_counter: usize,
    temp_counter: usize,
    global_strings: HashMap<String, String>,
    global_counter: usize,
    current_function: String,
    var_types: HashMap<String, String>,
}

impl IRGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            label_counter: 0,
            temp_counter: 0,
            global_strings: HashMap::new(),
            global_counter: 0,
            current_function: String::new(),
            var_types: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> EolResult<String> {
        self.emit_header();
        
        // 声明外部函数 (printf 和标准C库函数)
        self.emit_line("declare i32 @printf(i8*, ...)");
        self.emit_line("declare i64 @strlen(i8*)");
        self.emit_line("declare i8* @malloc(i64)");
        self.emit_line("declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg)");
        self.emit_line("");
        
        // 空字符串常量（用于 null 安全）
        self.emit_line("@.eol_empty_str = private unnamed_addr constant [1 x i8] c\"\\00\", align 1");
        self.emit_line("");
        
        // EOL 字符串拼接运行时
        // 输入: i8* %a, i8* %b（可为 null）
        // 输出: i8*（新分配内存，需调用者管理；malloc失败返回空字符串）
        self.emit_line("define i8* @__eol_string_concat(i8* %a, i8* %b) {");
        self.emit_line("entry:");
        self.emit_line("  ; 空指针安全检查：null → 空字符串 \"\"");
        self.emit_line("  %a_is_null = icmp eq i8* %a, null");
        self.emit_line("  %a_ptr = select i1 %a_is_null,");
        self.emit_line("    i8* getelementptr ([1 x i8], [1 x i8]* @.eol_empty_str, i64 0, i64 0),");
        self.emit_line("    i8* %a");
        self.emit_line("  ");
        self.emit_line("  %b_is_null = icmp eq i8* %b, null");
        self.emit_line("  %b_ptr = select i1 %b_is_null,");
        self.emit_line("    i8* getelementptr ([1 x i8], [1 x i8]* @.eol_empty_str, i64 0, i64 0),");
        self.emit_line("    i8* %b");
        self.emit_line("  ");
        self.emit_line("  ; 计算长度");
        self.emit_line("  %len_a = call i64 @strlen(i8* %a_ptr)");
        self.emit_line("  %len_b = call i64 @strlen(i8* %b_ptr)");
        self.emit_line("  %total_len = add i64 %len_a, %len_b");
        self.emit_line("  %buf_size = add i64 %total_len, 1  ; +1 for '\\0'");
        self.emit_line("  ");
        self.emit_line("  ; 内存分配");
        self.emit_line("  %result = call i8* @malloc(i64 %buf_size)");
        self.emit_line("  ");
        self.emit_line("  ; malloc 失败保护：返回空字符串而非崩溃");
        self.emit_line("  %is_null = icmp eq i8* %result, null");
        self.emit_line("  br i1 %is_null, label %fail, label %copy");
        self.emit_line("  ");
        self.emit_line("fail:");
        self.emit_line("  ret i8* getelementptr ([1 x i8], [1 x i8]* @.eol_empty_str, i64 0, i64 0)");
        self.emit_line("  ");
        self.emit_line("copy:");
        self.emit_line("  ; 快速内存复制（LLVM 会优化为 SSE/AVX 或 rep movsb）");
        self.emit_line("  call void @llvm.memcpy.p0i8.p0i8.i64(");
        self.emit_line("    i8* %result,");
        self.emit_line("    i8* %a_ptr,");
        self.emit_line("    i64 %len_a,");
        self.emit_line("    i1 false");
        self.emit_line("  )");
        self.emit_line("  ");
        self.emit_line("  ; 复制 b 到 offset = len_a");
        self.emit_line("  %dest_b = getelementptr i8, i8* %result, i64 %len_a");
        self.emit_line("  call void @llvm.memcpy.p0i8.p0i8.i64(");
        self.emit_line("    i8* %dest_b,");
        self.emit_line("    i8* %b_ptr,");
        self.emit_line("    i64 %len_b,");
        self.emit_line("    i1 false");
        self.emit_line("  )");
        self.emit_line("  ");
        self.emit_line("  ; 写入 null terminator");
        self.emit_line("  %end_ptr = getelementptr i8, i8* %result, i64 %total_len");
        self.emit_line("  store i8 0, i8* %end_ptr");
        self.emit_line("  ");
        self.emit_line("  ret i8* %result");
        self.emit_line("}");
        self.emit_line("");
        
        // 找到主类并记录
        let mut main_class = None;
        
        // 生成所有类方法
        for class in &program.classes {
            if class.members.iter().any(|m| {
                if let crate::ast::ClassMember::Method(method) = m {
                    method.name == "main" &&
                    method.modifiers.contains(&crate::ast::Modifier::Public) &&
                    method.modifiers.contains(&crate::ast::Modifier::Static)
                } else {
                    false
                }
            }) {
                main_class = Some(class.name.clone());
            }
            self.generate_class(class)?;
        }
        
        // 生成C入口点main函数
        if let Some(class_name) = main_class {
            self.emit_line("; C entry point");
            self.emit_line("define i32 @main() {");
            self.indent += 1;
            self.emit_line("entry:");
            self.emit_line(&format!("  call void @{}.{:}()", class_name, "main"));
            self.emit_line("  ret i32 0");
            self.indent -= 1;
            self.emit_line("}");
            self.emit_line("");
        }
        
        Ok(self.output.clone())
    }

    fn emit_header(&mut self) {
        self.emit_line("; EOL (Ethernos Object Language) Generated LLVM IR");
        self.emit_line("target triple = \"x86_64-pc-windows-msvc\"");
        self.emit_line("");
    }

    fn emit_line(&mut self, line: &str) {
        if !line.is_empty() {
            self.output.push_str(&"  ".repeat(self.indent));
        }
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}.{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn new_temp(&mut self) -> String {
        let temp = format!("%t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    fn get_or_create_string_constant(&mut self, s: &str) -> String {
        if let Some(name) = self.global_strings.get(s) {
            return name.clone();
        }
        
        let name = format!("@.str.{}", self.global_counter);
        self.global_counter += 1;
        
        // 转义字符串
        let escaped = s.replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\0A")
            .replace("\r", "\\0D")
            .replace("\t", "\\09");
        
        // 声明字符串常量
        let len = s.len() + 1; // +1 for null terminator
        let decl = format!("{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1", 
            name, len, escaped);
        
        // 存储以便稍后输出到全局区
        self.global_strings.insert(s.to_string(), name.clone());
        
        name
    }

    fn generate_class(&mut self, class: &ClassDecl) -> EolResult<()> {
        for member in &class.members {
            match member {
                ClassMember::Method(method) => {
                    if !method.modifiers.contains(&Modifier::Native) {
                        self.generate_method(&class.name, method)?;
                    }
                }
                ClassMember::Field(_) => {
                    // 静态字段暂不实现
                }
            }
        }
        Ok(())
    }

    fn generate_method(&mut self, class_name: &str, method: &MethodDecl) -> EolResult<()> {
        let fn_name = format!("{}.{}", class_name, method.name);
        self.current_function = fn_name.clone();
        
        // 重置临时变量计数器
        self.temp_counter = 0;
        // 清除变量类型映射（每个方法都有自己的作用域）
        self.var_types.clear();
        
        // 函数签名
        let ret_type = self.type_to_llvm(&method.return_type);
        let params: Vec<String> = method.params.iter()
            .map(|p| format!("{} %{}.{}", self.type_to_llvm(&p.param_type), class_name, p.name))
            .collect();
        
        self.emit_line(&format!("define {} @{}({}) {{",
            ret_type, fn_name, params.join(", ")));
        self.indent += 1;
        
        // 入口标签
        self.emit_line("entry:");
        
        // 为参数分配局部变量
        for param in &method.params {
            let param_type = self.type_to_llvm(&param.param_type);
            self.emit_line(&format!("  %{} = alloca {}", param.name, param_type));
            self.emit_line(&format!("  store {} %{}.{}, {}* %{}",
                param_type, class_name, param.name, param_type, param.name));
            // 存储参数类型信息
            self.var_types.insert(param.name.clone(), param_type);
        }
        
        // 生成方法体
        if let Some(body) = method.body.as_ref() {
            self.generate_block(body)?;
        }
        
        // 如果函数没有返回语句且返回void，添加隐式返回
        if method.return_type == Type::Void {
            self.emit_line("  ret void");
        }
        
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        
        Ok(())
    }

    fn generate_block(&mut self, block: &Block) -> EolResult<()> {
        for stmt in &block.statements {
            self.generate_statement(stmt)?;
        }
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Stmt) -> EolResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.generate_expression(expr)?;
            }
            Stmt::VarDecl(var) => {
                let var_type = self.type_to_llvm(&var.var_type);
                self.emit_line(&format!("  %{} = alloca {}", var.name, var_type));
                // 存储变量类型信息
                self.var_types.insert(var.name.clone(), var_type.clone());
                
                if let Some(init) = var.initializer.as_ref() {
                    let value = self.generate_expression(init)?;
                    self.emit_line(&format!("  store {}, {}* %{}",
                        value, var_type, var.name));
                }
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr.as_ref() {
                    let value = self.generate_expression(e)?;
                    self.emit_line(&format!("  ret {}", value));
                } else {
                    self.emit_line("  ret void");
                }
            }
            Stmt::Block(block) => {
                self.generate_block(block)?;
            }
            Stmt::If(if_stmt) => {
                self.generate_if_statement(if_stmt)?;
            }
            Stmt::While(while_stmt) => {
                self.generate_while_statement(while_stmt)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStmt) -> EolResult<()> {
        let then_label = self.new_label("then");
        let else_label = self.new_label("else");
        let merge_label = self.new_label("ifmerge");
        
        let cond = self.generate_expression(&if_stmt.condition)?;
        let (_, cond_val) = self.parse_typed_value(&cond);
        let cond_reg = self.new_temp();
        self.emit_line(&format!("  {} = icmp ne i1 {}, 0", cond_reg, cond_val));
        
        if if_stmt.else_branch.is_some() {
            self.emit_line(&format!("  br i1 {}, label %{}, label %{}",
                cond_reg, then_label, else_label));
        } else {
            self.emit_line(&format!("  br i1 {}, label %{}, label %{}",
                cond_reg, then_label, merge_label));
        }
        
        // then块
        self.emit_line(&format!("{}:", then_label));
        self.generate_statement(&if_stmt.then_branch)?;
        self.emit_line(&format!("  br label %{}", merge_label));
        
        // else块
        if let Some(else_branch) = if_stmt.else_branch.as_ref() {
            self.emit_line(&format!("{}:", else_label));
            self.generate_statement(else_branch)?;
            self.emit_line(&format!("  br label %{}", merge_label));
        }
        
        // merge块
        self.emit_line(&format!("{}:", merge_label));
        
        Ok(())
    }

    fn generate_while_statement(&mut self, while_stmt: &WhileStmt) -> EolResult<()> {
        let cond_label = self.new_label("while.cond");
        let body_label = self.new_label("while.body");
        let end_label = self.new_label("while.end");
        
        self.emit_line(&format!("  br label %{}", cond_label));
        
        // 条件块
        self.emit_line(&format!("{}:", cond_label));
        let cond = self.generate_expression(&while_stmt.condition)?;
        let (_, cond_val) = self.parse_typed_value(&cond);
        let cond_reg = self.new_temp();
        self.emit_line(&format!("  {} = icmp ne i1 {}, 0", cond_reg, cond_val));
        self.emit_line(&format!("  br i1 {}, label %{}, label %{}",
            cond_reg, body_label, end_label));
        
        // 循环体
        self.emit_line(&format!("{}:", body_label));
        self.generate_statement(&while_stmt.body)?;
        self.emit_line(&format!("  br label %{}", cond_label));
        
        // 结束块
        self.emit_line(&format!("{}:", end_label));
        
        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expr) -> EolResult<String> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Identifier(name) => {
                // 加载变量值
                let temp = self.new_temp();
                // 从 var_types 中获取变量类型，默认为 i64（为了向后兼容）
                let var_type = self.var_types.get(name).cloned().unwrap_or_else(|| "i64".to_string());
                self.emit_line(&format!("  {} = load {}, {}* %{}, align 8", temp, var_type, var_type, name));
                Ok(format!("{} {}", var_type, temp))
            }
            Expr::Binary(bin) => self.generate_binary_expression(bin),
            Expr::Unary(unary) => self.generate_unary_expression(unary),
            Expr::Call(call) => self.generate_call_expression(call),
            Expr::Assignment(assign) => self.generate_assignment(assign),
            _ => Err(codegen_error("Unsupported expression".to_string())),
        }
    }

    fn generate_literal(&mut self, lit: &LiteralValue) -> EolResult<String> {
        match lit {
            LiteralValue::Int(val) => Ok(format!("i64 {}", val)),
            LiteralValue::Float(val) => Ok(format!("double {}", val)),
            LiteralValue::Bool(val) => Ok(format!("i1 {}", if *val { 1 } else { 0 })),
            LiteralValue::String(s) => {
                let global_name = self.get_or_create_string_constant(s);
                let temp = self.new_temp();
                let len = s.len() + 1;
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0", 
                    temp, len, len, global_name));
                Ok(format!("i8* {}", temp))
            }
            LiteralValue::Char(c) => Ok(format!("i8 {}", *c as u8)),
            LiteralValue::Null => Ok("i64 0".to_string()),
        }
    }

    fn generate_binary_expression(&mut self, bin: &BinaryExpr) -> EolResult<String> {
        let left = self.generate_expression(&bin.left)?;
        let right = self.generate_expression(&bin.right)?;
        
        // 解析类型和值
        let (left_type, left_val) = self.parse_typed_value(&left);
        let (right_type, right_val) = self.parse_typed_value(&right);
        
        let temp = self.new_temp();
        
        match bin.op {
            BinaryOp::Add => {
                // 字符串拼接处理
                if left_type == "i8*" && right_type == "i8*" {
                    // 调用内建的字符串拼接函数
                    self.emit_line(&format!("  {} = call i8* @__eol_string_concat(i8* {}, i8* {})",
                        temp, left_val, right_val));
                } else if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = add {} {}, {}",
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fadd {} {}, {}",
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Sub => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sub {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fsub {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Mul => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = mul {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fmul {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Div => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sdiv {} {}, {}",
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fdiv {} {}, {}",
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Mod => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = srem {} {}, {}",
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = frem {} {}, {}",
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Eq => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp eq {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp oeq {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Ne => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp ne {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp one {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Lt => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp slt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp olt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Le => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp sle {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp ole {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Gt => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp sgt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp ogt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Ge => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp sge {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp oge {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::And => {
                self.emit_line(&format!("  {} = and {} {}, {}", 
                    temp, left_type, left_val, right_val));
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Or => {
                self.emit_line(&format!("  {} = or {} {}, {}",
                    temp, left_type, left_val, right_val));
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::BitAnd => {
                self.emit_line(&format!("  {} = and {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::BitOr => {
                self.emit_line(&format!("  {} = or {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::BitXor => {
                self.emit_line(&format!("  {} = xor {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::Shl => {
                self.emit_line(&format!("  {} = shl {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::Shr => {
                self.emit_line(&format!("  {} = ashr {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::UnsignedShr => {
                self.emit_line(&format!("  {} = lshr {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            _ => return Err(codegen_error(format!("Unsupported binary operator: {:?}", bin.op))),
        }
        
        Ok(format!("{} {}", left_type, temp))
    }

    fn generate_unary_expression(&mut self, unary: &UnaryExpr) -> EolResult<String> {
        let operand = self.generate_expression(&unary.operand)?;
        let (op_type, op_val) = self.parse_typed_value(&operand);
        let temp = self.new_temp();
        
        match unary.op {
            UnaryOp::Neg => {
                if op_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sub {} 0, {}",
                        temp, op_type, op_val));
                } else {
                    self.emit_line(&format!("  {} = fneg {} {}",
                        temp, op_type, op_val));
                }
            }
            UnaryOp::Not => {
                self.emit_line(&format!("  {} = xor {} {}, 1",
                    temp, op_type, op_val));
                return Ok(format!("i1 {}", temp));
            }
            UnaryOp::BitNot => {
                // 位取反：xor 操作数与 -1
                if op_type.starts_with("i") {
                    self.emit_line(&format!("  {} = xor {} {}, -1",
                        temp, op_type, op_val));
                } else {
                    // 浮点数不支持位取反，但类型系统应该已经阻止了这种情况
                    return Err(codegen_error("Bitwise NOT not supported for floating point".to_string()));
                }
            }
            _ => return Err(codegen_error("Unsupported unary operator".to_string())),
        }
        
        Ok(format!("{} {}", op_type, temp))
    }

    fn generate_call_expression(&mut self, call: &CallExpr) -> EolResult<String> {
        // 处理 print 和 println 函数
        if let Expr::Identifier(name) = call.callee.as_ref() {
            if name == "print" {
                return self.generate_print_call(&call.args, false);
            }
            if name == "println" {
                return self.generate_print_call(&call.args, true);
            }
        }
        
        // 处理普通函数调用
        let fn_name = match call.callee.as_ref() {
            Expr::Identifier(name) => name.clone(),
            Expr::MemberAccess(member) => {
                if let Expr::Identifier(class_name) = member.object.as_ref() {
                    format!("{}.{}", class_name, member.member)
                } else {
                    return Err(codegen_error("Invalid method call".to_string()));
                }
            }
            _ => return Err(codegen_error("Invalid function call".to_string())),
        };
        
        let args: Vec<String> = call.args.iter()
            .map(|arg| self.generate_expression(arg))
            .collect::<EolResult<Vec<_>>>()?;
        
        let temp = self.new_temp();
        self.emit_line(&format!("  {} = call i64 @{}({})", 
            temp, fn_name, args.join(", ")));
        
        Ok(format!("i64 {}", temp))
    }

    fn generate_print_call(&mut self, args: &[Expr], newline: bool) -> EolResult<String> {
        if args.is_empty() {
            return Err(codegen_error("print requires at least one argument".to_string()));
        }
        
        let first_arg = &args[0];
        
        match first_arg {
            Expr::Literal(LiteralValue::String(s)) => {
                let global_name = self.get_or_create_string_constant(s);
                let fmt_str = if newline { "%s\n" } else { "%s" };
                let fmt_name = self.get_or_create_string_constant(fmt_str);
                let len = s.len() + 1;
                let fmt_len = fmt_str.len() + 1; // 加上null终止符
                
                let str_ptr = self.new_temp();
                let fmt_ptr = self.new_temp();
                
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    str_ptr, len, len, global_name));
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));
                
                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i8* {})",
                    fmt_ptr, str_ptr));
            }
            Expr::Literal(LiteralValue::Int(_)) => {
                let value = self.generate_expression(first_arg)?;
                let (_, val) = self.parse_typed_value(&value);
                let fmt_str = if newline { "%ld\n" } else { "%ld" };
                let fmt_name = self.get_or_create_string_constant(fmt_str);
                let fmt_len = fmt_str.len() + 1;
                
                let fmt_ptr = self.new_temp();
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));
                
                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})",
                    fmt_ptr, val));
            }
            _ => {
                // 根据类型决定格式字符串
                let value = self.generate_expression(first_arg)?;
                let (type_str, val) = self.parse_typed_value(&value);
                
                if type_str == "i8*" {
                    // 字符串指针类型
                    let fmt_str = if newline { "%s\n" } else { "%s" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i8* {})",
                        fmt_ptr, val));
                } else if type_str.starts_with("i") && type_str != "i8*" {
                    // 整数类型（排除i8*）
                    let fmt_str = if newline { "%ld\n" } else { "%ld" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})",
                        fmt_ptr, val));
                } else if type_str == "double" || type_str == "float" {
                    // 浮点数类型
                    let fmt_str = if newline { "%f\n" } else { "%f" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, double {})",
                        fmt_ptr, val));
                } else {
                    // 默认作为字符串处理
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

    fn generate_assignment(&mut self, assign: &AssignmentExpr) -> EolResult<String> {
        let value = self.generate_expression(&assign.value)?;
        
        if let Expr::Identifier(name) = assign.target.as_ref() {
            let (_, val) = self.parse_typed_value(&value);
            // 简化处理，假设都是i64类型
            self.emit_line(&format!("  store i64 {}, i64* %{}", val, name));
            Ok(value)
        } else {
            Err(codegen_error("Invalid assignment target".to_string()))
        }
    }

    fn parse_typed_value(&self, typed_val: &str) -> (String, String) {
        let parts: Vec<&str> = typed_val.splitn(2, ' ').collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("i64".to_string(), typed_val.to_string())
        }
    }

    fn type_to_llvm(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Bool => "i1".to_string(),
            Type::String => "i8*".to_string(),
            Type::Char => "i8".to_string(),
            Type::Object(_) => "i8*".to_string(),
            Type::Array(inner) => format!("{}*", self.type_to_llvm(inner)),
            Type::Function(_) => "i8*".to_string(),
        }
    }

    pub fn get_global_strings(&self) -> &HashMap<String, String> {
        &self.global_strings
    }
}

impl Default for IRGenerator {
    fn default() -> Self {
        Self::new()
    }
}
