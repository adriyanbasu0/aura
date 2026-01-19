#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Const(ConstDecl),
    Var(VarDecl),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Box<Type>,
    pub body: Vec<Stmt>,
    pub attrs: Vec<FunctionAttribute>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionAttribute {
    Noreturn,
    CCall,
    StdCall,
    Inline,
    Entry(Option<String>),
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct Union {
    pub name: String,
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone)]
pub struct UnionVariant {
    pub name: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub ty: Option<Box<Type>>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub ty: Option<Box<Type>>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Const(ConstStmt),
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Block(Vec<Stmt>),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Asm(AsmStmt),
    Defer(Box<Stmt>),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: String,
    pub ty: Option<Box<Type>>,
    pub value: Box<Expr>,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct ConstStmt {
    pub name: String,
    pub ty: Option<Box<Type>>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Box<Expr>,
    pub then_branch: Vec<Stmt>,
    pub else_branch: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Box<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub init: Box<Stmt>,
    pub condition: Box<Expr>,
    pub update: Box<Stmt>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct AsmStmt {
    pub template: String,
    pub inputs: Vec<AsmOperand>,
    pub outputs: Vec<AsmOperand>,
    pub clobbers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AsmOperand {
    pub constraint: String,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Syscall(String, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    PtrField(Box<Expr>, String),
    Cast(Box<Expr>, Type),
    Sizeof(Type),
    Alignof(Type),
    Offsetof(Type, String),
    Assign(Box<Expr>, Box<Expr>),
    AddrOf(Box<Expr>),
    Deref(Box<Expr>),
    Block(Vec<Stmt>, Option<Box<Expr>>),
    If(Box<IfExpr>),
    Alloc(Box<Type>, Box<Expr>),
    Free(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    AddrOf,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LShift,
    RShift,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    Neq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64, IntSuffix),
    Float(f64, FloatSuffix),
    Bool(bool),
    String(Vec<u8>),
    Char(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntSuffix {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,
    Isize,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatSuffix {
    F32,
    F64,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Usize,
    Isize,
    BitInt(u8, bool),
    Ptr(Box<Type>),
    MutPtr(Box<Type>),
    ConstPtr(Box<Type>),
    Array(usize, Box<Type>),
    Func(Vec<Type>, Box<Type>),
    Named(String),
    Error,
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Void => 0,
            Type::Bool => 1,
            Type::I8 | Type::U8 => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 | Type::F32 => 4,
            Type::I64 | Type::U64 | Type::F64 | Type::Usize | Type::Isize => 8,
            Type::BitInt(bits, _) => {
                if *bits <= 8 {
                    1
                } else if *bits <= 16 {
                    2
                } else if *bits <= 32 {
                    4
                } else if *bits <= 64 {
                    8
                } else if *bits <= 128 {
                    16
                } else {
                    (*bits as usize + 7) / 8
                }
            }
            Type::Ptr(_) | Type::MutPtr(_) | Type::ConstPtr(_) => 8,
            Type::Array(n, t) => *n * t.size(),
            Type::Func(_, _) => 8,
            Type::Named(_) => 0,
            Type::Error => 0,
        }
    }

    pub fn align(&self) -> usize {
        match self {
            Type::Void => 1,
            Type::Bool => 1,
            Type::I8 | Type::U8 => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 | Type::F32 => 4,
            Type::I64 | Type::U64 | Type::F64 | Type::Usize | Type::Isize => 8,
            Type::BitInt(bits, _) => {
                if *bits <= 8 {
                    1
                } else if *bits <= 16 {
                    2
                } else if *bits <= 32 {
                    4
                } else if *bits <= 64 {
                    8
                } else if *bits <= 128 {
                    8
                } else {
                    16
                }
            }
            Type::Ptr(_) | Type::MutPtr(_) | Type::ConstPtr(_) => 8,
            Type::Array(_, t) => t.align(),
            Type::Func(_, _) => 1,
            Type::Named(_) => 1,
            Type::Error => 1,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::Isize
                | Type::BitInt(_, _)
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Ptr(_) | Type::MutPtr(_) | Type::ConstPtr(_))
    }
}
