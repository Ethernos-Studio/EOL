use crate::types::{Type, ParameterInfo, ClassInfo, MethodInfo};
use crate::error::SourceLocation;

#[derive(Debug, Clone)]
pub struct Program {
    pub classes: Vec<ClassDecl>,
    pub interfaces: Vec<InterfaceDecl>,
    pub top_level_functions: Vec<TopLevelFunction>,
}

/// 顶层函数声明（类外函数）
#[derive(Debug, Clone)]
pub struct TopLevelFunction {
    pub name: String,
    pub modifiers: Vec<Modifier>,
    pub return_type: Type,
    pub params: Vec<ParameterInfo>,
    pub body: Block,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub modifiers: Vec<Modifier>,
    pub methods: Vec<MethodDecl>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub modifiers: Vec<Modifier>,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,  // 实现的接口列表
    pub members: Vec<ClassMember>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Method(MethodDecl),
    Field(FieldDecl),
    Constructor(ConstructorDecl),
    Destructor(DestructorDecl),
    InstanceInitializer(Block),  // 实例初始化块 { ... }
    StaticInitializer(Block),    // 静态初始化块 static { ... }
}

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name: String,
    pub modifiers: Vec<Modifier>,
    pub return_type: Type,
    pub params: Vec<ParameterInfo>,
    pub body: Option<Block>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub field_type: Type,
    pub modifiers: Vec<Modifier>,
    pub initializer: Option<Expr>,
    pub loc: SourceLocation,
}

/// 构造函数声明
#[derive(Debug, Clone)]
pub struct ConstructorDecl {
    pub modifiers: Vec<Modifier>,
    pub params: Vec<crate::types::ParameterInfo>,
    pub body: Block,
    pub constructor_call: Option<ConstructorCall>, // this() 或 super() 调用
    pub loc: SourceLocation,
}

/// 构造函数调用（this() 或 super()）
#[derive(Debug, Clone)]
pub enum ConstructorCall {
    This(Vec<Expr>),   // this(args)
    Super(Vec<Expr>),  // super(args)
}

/// 析构函数声明
#[derive(Debug, Clone)]
pub struct DestructorDecl {
    pub modifiers: Vec<Modifier>,
    pub body: Block,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Public,
    Private,
    Protected,
    Static,
    Final,
    Abstract,
    Native,
    Main,      // 标记主类，用于解决多main冲突
    Override,  // @Override 注解，标记方法重写
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    VarDecl(VarDecl),
    Return(Option<Expr>),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    DoWhile(DoWhileStmt),
    Switch(SwitchStmt),
    Block(Block),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub var_type: Type,
    pub initializer: Option<Expr>,
    pub is_final: bool,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub init: Option<Box<Stmt>>,
    pub condition: Option<Expr>,
    pub update: Option<Expr>,
    pub body: Box<Stmt>,
    pub loc: SourceLocation,
}

/// do-while 循环语句
#[derive(Debug, Clone)]
pub struct DoWhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
    pub loc: SourceLocation,
}

/// switch case 分支
#[derive(Debug, Clone)]
pub struct Case {
    pub value: i64,
    pub body: Vec<Stmt>,
}

/// switch 语句
#[derive(Debug, Clone)]
pub struct SwitchStmt {
    pub expr: Expr,
    pub cases: Vec<Case>,
    pub default: Option<Vec<Stmt>>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralValue),
    Identifier(String),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MemberAccess(MemberAccessExpr),
    New(NewExpr),
    Assignment(AssignmentExpr),
    Cast(CastExpr),
    ArrayCreation(ArrayCreationExpr),
    ArrayAccess(ArrayAccessExpr),
    ArrayInit(ArrayInitExpr),  // 数组初始化: {1, 2, 3}
    MethodRef(MethodRefExpr),  // 方法引用: ClassName::methodName
    Lambda(LambdaExpr),        // Lambda 表达式: (params) -> { body }
    Ternary(TernaryExpr),      // 三元运算符: condition ? true_expr : false_expr
    InstanceOf(InstanceOfExpr), // instanceof 运算符: obj instanceof Type
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bool(bool),
    Char(char),
    Null,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    UnsignedShr,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct MemberAccessExpr {
    pub object: Box<Expr>,
    pub member: String,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct NewExpr {
    pub class_name: String,
    pub args: Vec<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub op: AssignOp,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}

#[derive(Debug, Clone)]
pub struct CastExpr {
    pub expr: Box<Expr>,
    pub target_type: Type,
    pub loc: SourceLocation,
}

/// 数组创建表达式: new Type[size] 或 new Type[size1][size2]... 或 new Type[size]()
#[derive(Debug, Clone)]
pub struct ArrayCreationExpr {
    pub element_type: Type,
    pub sizes: Vec<Expr>,  // 支持多维数组，每个维度的大小
    pub zero_init: bool,   // 是否零初始化 new Type[size]()
    pub loc: SourceLocation,
}

/// 数组初始化表达式: {1, 2, 3}
#[derive(Debug, Clone)]
pub struct ArrayInitExpr {
    pub elements: Vec<Expr>,
    pub loc: SourceLocation,
}

/// 数组访问表达式: arr[index]
#[derive(Debug, Clone)]
pub struct ArrayAccessExpr {
    pub array: Box<Expr>,
    pub index: Box<Expr>,
    pub loc: SourceLocation,
}

/// 方法引用表达式: ClassName::methodName 或 obj::methodName
#[derive(Debug, Clone)]
pub struct MethodRefExpr {
    pub class_name: Option<String>,  // 类名（静态方法引用）
    pub object: Option<Box<Expr>>,   // 对象表达式（实例方法引用）
    pub method_name: String,
    pub loc: SourceLocation,
}

/// Lambda 表达式: (params) -> { body }
#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<LambdaParam>,
    pub body: LambdaBody,
    pub loc: SourceLocation,
}

/// Lambda 参数
#[derive(Debug, Clone)]
pub struct LambdaParam {
    pub name: String,
    pub param_type: Option<Type>,  // 可选的类型注解
}

/// Lambda 体（可以是表达式或语句块）
#[derive(Debug, Clone)]
pub enum LambdaBody {
    Expr(Box<Expr>),      // 单表达式: (x) -> x * 2
    Block(Block),         // 语句块: (x) -> { return x * 2; }
}

/// 三元运算符表达式: condition ? true_expr : false_expr
#[derive(Debug, Clone)]
pub struct TernaryExpr {
    pub condition: Box<Expr>,
    pub true_branch: Box<Expr>,
    pub false_branch: Box<Expr>,
    pub loc: SourceLocation,
}

/// instanceof 表达式: obj instanceof Type
#[derive(Debug, Clone)]
pub struct InstanceOfExpr {
    pub expr: Box<Expr>,
    pub target_type: crate::types::Type,
    pub loc: SourceLocation,
}

impl Program {
    pub fn find_main_class(&self) -> Option<&ClassDecl> {
        self.classes.iter().find(|c| {
            c.members.iter().any(|m| {
                if let ClassMember::Method(method) = m {
                    method.name == "main"
                        && method.modifiers.contains(&Modifier::Public)
                        && method.modifiers.contains(&Modifier::Static)
                        && method.params.is_empty()
                        && method.return_type == Type::Void
                } else {
                    false
                }
            })
        })
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            classes: Vec::new(),
            interfaces: Vec::new(),
            top_level_functions: Vec::new(),
        }
    }
}
