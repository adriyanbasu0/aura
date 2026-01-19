pub mod binary;
use crate::ast::*;
pub use binary::*;
use std::fmt;

#[derive(Debug)]
pub struct CodegenError {
    pub message: String,
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Codegen error: {}", self.message)
    }
}

impl std::error::Error for CodegenError {}

// FEATURE 9: Bit-precise integer type tracking
#[derive(Debug, Clone, Copy)]
struct IntType {
    bits: u8,
    signed: bool,
}

impl IntType {
    fn from_aura_type(ty: &Type) -> Option<Self> {
        match ty {
            Type::I8 => Some(IntType {
                bits: 8,
                signed: true,
            }),
            Type::I16 => Some(IntType {
                bits: 16,
                signed: true,
            }),
            Type::I32 => Some(IntType {
                bits: 32,
                signed: true,
            }),
            Type::I64 => Some(IntType {
                bits: 64,
                signed: true,
            }),
            Type::U8 => Some(IntType {
                bits: 8,
                signed: false,
            }),
            Type::U16 => Some(IntType {
                bits: 16,
                signed: false,
            }),
            Type::U32 => Some(IntType {
                bits: 32,
                signed: false,
            }),
            Type::U64 => Some(IntType {
                bits: 64,
                signed: false,
            }),
            Type::BitInt(bits, signed) => Some(IntType {
                bits: *bits,
                signed: *signed,
            }),
            _ => None,
        }
    }

    fn from_suffix(suffix: &IntSuffix) -> Option<Self> {
        match suffix {
            IntSuffix::I8 => Some(IntType {
                bits: 8,
                signed: true,
            }),
            IntSuffix::I16 => Some(IntType {
                bits: 16,
                signed: true,
            }),
            IntSuffix::I32 => Some(IntType {
                bits: 32,
                signed: true,
            }),
            IntSuffix::I64 => Some(IntType {
                bits: 64,
                signed: true,
            }),
            IntSuffix::U8 => Some(IntType {
                bits: 8,
                signed: false,
            }),
            IntSuffix::U16 => Some(IntType {
                bits: 16,
                signed: false,
            }),
            IntSuffix::U32 => Some(IntType {
                bits: 32,
                signed: false,
            }),
            IntSuffix::U64 => Some(IntType {
                bits: 64,
                signed: false,
            }),
            IntSuffix::Usize | IntSuffix::Isize => Some(IntType {
                bits: 64,
                signed: false,
            }),
            IntSuffix::None => None,
        }
    }

    // FEATURE 9: Get minimum storage size in bytes
    fn storage_size(&self) -> u8 {
        (self.bits + 7) / 8
    }

    // FEATURE 9: Get mask to constrain value to bit width
    fn mask(&self) -> u64 {
        if self.bits >= 64 {
            0xFFFFFFFFFFFFFFFF
        } else {
            (1u64 << self.bits) - 1
        }
    }

    // FEATURE 9: Check if value fits in this type
    fn fits(&self, val: i64) -> bool {
        if self.bits >= 64 {
            return true;
        }

        if self.signed {
            let min = -(1i64 << (self.bits - 1));
            let max = (1i64 << (self.bits - 1)) - 1;
            val >= min && val <= max
        } else {
            let max = (1u64 << self.bits) - 1;
            val >= 0 && (val as u64) <= max
        }
    }
}

#[derive(Debug)]
pub struct AuraObject {
    pub entry_point: u64,
    pub text: Vec<u8>,
    pub data: Vec<u8>,
    pub bss_size: usize,
    pub relocations: Vec<Relocation>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: usize,
    pub symbol: String,
    pub kind: RelocationKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelocationKind {
    Absolute64,
    Relative32,
    Absolute32,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Data,
    Object,
}

pub fn generate(typed_ast: &Program) -> Result<AuraObject, CodegenError> {
    let mut codegen = CodeGenerator::new();

    for item in &typed_ast.items {
        if let Item::Function(f) = item {
            for attr in &f.attrs {
                if let FunctionAttribute::Entry(Some(entry_name)) = attr {
                    codegen.entry_point_name = Some(entry_name.clone());
                } else if let FunctionAttribute::Entry(None) = attr {
                    codegen.entry_point_name = Some(f.name.clone());
                }
            }
        }
    }

    if let Some(ref entry_name) = codegen.entry_point_name {
        let mut function_exists = false;
        for item in &typed_ast.items {
            if let Item::Function(f) = item {
                if f.name == *entry_name {
                    function_exists = true;
                    break;
                }
            }
        }
        if !function_exists {
            return Err(CodegenError {
                message: format!("Entry point function '{}' does not exist", entry_name),
            });
        }
    }

    for item in &typed_ast.items {
        codegen.generate_item(item)?;
    }

    Ok(AuraObject {
        entry_point: codegen.entry_point,
        text: codegen.text,
        data: codegen.data,
        bss_size: codegen.bss_size,
        relocations: codegen.relocations,
        symbols: codegen.symbols,
    })
}

struct CodeGenerator {
    text: Vec<u8>,
    data: Vec<u8>,
    bss_size: usize,
    relocations: Vec<Relocation>,
    symbols: Vec<Symbol>,
    entry_point: u64,
    current_offset: usize,
    label_positions: HashMap<String, usize>,
    entry_point_name: Option<String>,
    variables: HashMap<String, u64>,
}

type HashMap<K, V> = std::collections::HashMap<K, V>;

impl CodeGenerator {
    fn new() -> Self {
        CodeGenerator {
            text: Vec::new(),
            data: Vec::new(),
            bss_size: 0,
            relocations: Vec::new(),
            symbols: Vec::new(),
            entry_point: 0,
            current_offset: 0,
            label_positions: HashMap::new(),
            entry_point_name: None,
            variables: HashMap::new(),
        }
    }

    fn generate_item(&mut self, item: &Item) -> Result<(), CodegenError> {
        match item {
            Item::Function(f) => {
                self.generate_function(f)?;
            }
            Item::Const(c) => {
                self.generate_const_item(c)?;
            }
            Item::Var(v) => self.generate_var_item(v)?,
            _ => {}
        }
        Ok(())
    }

    fn generate_const_item(&mut self, c: &ConstDecl) -> Result<(), CodegenError> {
        match &*c.value {
            Expr::Literal(Literal::Int(val, _)) => {
                let offset = self.data.len();
                self.data.extend_from_slice(&val.to_le_bytes());
                self.symbols.push(Symbol {
                    name: c.name.clone(),
                    offset: offset as u64,
                    size: 8,
                    kind: SymbolKind::Data,
                });
            }
            Expr::Literal(Literal::String(bytes)) => {
                let offset = self.data.len();
                self.data.extend_from_slice(bytes);
                self.data.push(0);
                self.symbols.push(Symbol {
                    name: c.name.clone(),
                    offset: offset as u64,
                    size: bytes.len() as u64,
                    kind: SymbolKind::Data,
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_var_item(&mut self, _v: &VarDecl) -> Result<(), CodegenError> {
        self.bss_size += 8;
        Ok(())
    }

    fn generate_function(&mut self, f: &Function) -> Result<(), CodegenError> {
        let func_start = self.text.len();

        self.symbols.push(Symbol {
            name: f.name.clone(),
            offset: func_start as u64,
            size: 0,
            kind: SymbolKind::Function,
        });

        if let Some(entry_name) = &self.entry_point_name {
            if f.name == *entry_name {
                self.entry_point = func_start as u64;
            }
        }

        for stmt in &f.body {
            self.generate_stmt(stmt)?;
        }

        if let Some((idx, _)) = self
            .symbols
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.name == f.name)
        {
            self.symbols[idx].size = (self.text.len() - func_start) as u64;
        }

        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match stmt {
            Stmt::Return(Some(expr)) => {
                self.generate_return(expr)?;
                self.ret();
            }
            Stmt::Return(None) => {
                self.xor_rax_rax();
                self.ret();
            }
            Stmt::Const(c) => {
                self.generate_const_stmt(c)?;
            }
            Stmt::Let(l) => {
                self.generate_let(l)?;
            }
            Stmt::Expr(e) => {
                self.generate_expr(e)?;
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.generate_stmt(s)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_const_stmt(&mut self, c: &ConstStmt) -> Result<(), CodegenError> {
        match &*c.value {
            Expr::Literal(Literal::Int(val, _)) => {
                let offset = self.data.len();
                self.data.extend_from_slice(&val.to_le_bytes());
                self.symbols.push(Symbol {
                    name: c.name.clone(),
                    offset: offset as u64,
                    size: 8,
                    kind: SymbolKind::Data,
                });
            }
            Expr::Literal(Literal::String(bytes)) => {
                let offset = self.data.len();
                self.data.extend_from_slice(bytes);
                self.data.push(0);
                self.symbols.push(Symbol {
                    name: c.name.clone(),
                    offset: offset as u64,
                    size: bytes.len() as u64,
                    kind: SymbolKind::Data,
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_let(&mut self, l: &LetStmt) -> Result<(), CodegenError> {
        let value_type = match &*l.value {
            Expr::Literal(Literal::Int(val, _)) => {
                let offset = self.data.len();
                self.data.extend_from_slice(&val.to_le_bytes());
                self.variables.insert(l.name.clone(), offset as u64);
                return Ok(());
            }
            Expr::Identifier(name) => {
                if let Some(&offset) = self.variables.get(name) {
                    let addr = self.get_data_address(offset as usize);
                    self.mov_r10_immediate(addr);
                    self.mov_rax_from_r10(); // Load value into RAX
                } else {
                    return Err(CodegenError {
                        message: format!("Undefined variable: {}", name),
                    });
                }
                // Now RAX holds the value, store it
                let offset = self.data.len();
                self.data.extend_from_slice(&[0u8; 8]); // Assume 8 bytes for now
                let var_addr = self.get_data_address(offset);
                self.mov_r10_immediate(var_addr);
                self.mov_rax_to_r10_mem();
                self.variables.insert(l.name.clone(), offset as u64);
                return Ok(());
            }
            _ => self.generate_expr(&l.value)?,
        };

        // If the expression was not a literal or identifier, its result is in RAX.
        // Store it in the data section.
        let offset = self.data.len();
        self.data.extend_from_slice(&[0u8; 8]); // Reserve 8 bytes for the result
        let var_addr = self.get_data_address(offset);
        self.mov_r10_immediate(var_addr);
        self.mov_rax_to_r10_mem();
        self.variables.insert(l.name.clone(), offset as u64);

        Ok(())
    }

    fn generate_return(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match expr {
            Expr::Literal(Literal::Int(val, _)) => {
                self.mov_rax_immediate(*val as u64);
            }
            Expr::Identifier(name) => {
                if let Some(sym) = self
                    .symbols
                    .iter()
                    .find(|s| s.name == *name && s.kind == SymbolKind::Data)
                {
                    self.mov_rax_from_mem(sym.offset as u64);
                } else {
                    self.xor_rax_rax();
                }
            }
            _ => {
                self.xor_rax_rax();
            }
        }
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<u64, CodegenError> {
        match expr {
            Expr::Literal(Literal::Int(val, int_suffix)) => {
                let int_type = IntType::from_suffix(int_suffix);

                // FEATURE 9: Check if literal fits in type and apply mask
                if let Some(int_type) = int_type {
                    if !int_type.fits(*val) {
                        return Err(CodegenError {
                            message: format!("Integer literal {} does not fit in type", val),
                        });
                    }
                    // Apply mask to constrain to bit width
                    let masked = *val as u64 & int_type.mask();
                    // FEATURE 9: Emit width-aware immediate value
                    self.emit_width_immediate(masked, int_type.bits);
                    Ok(masked)
                } else {
                    // No type suffix, emit full width
                    Ok(*val as u64)
                }
            }
            Expr::Identifier(name) => {
                if let Some(sym) = self
                    .symbols
                    .iter()
                    .find(|s| s.name == *name && s.kind == SymbolKind::Data)
                {
                    self.mov_rax_from_mem(sym.offset as u64);
                    return Ok(0);
                }
                if let Some(&offset) = self.variables.get(name) {
                    let addr = self.get_data_address(offset as usize);
                    self.mov_r10_immediate(addr);
                    self.mov_rax_from_r10();
                    return Ok(0);
                }
                Ok(0)
            }
            Expr::Syscall(method_name, args) => {
                self.generate_syscall(method_name, args)?;
                Ok(0)
            }
            // FEATURE 1: Explicit memory allocation
            Expr::Alloc(ty, count) => {
                let size = self.generate_expr(count)?;
                // Put count in rdi
                self.mov_rdi_immediate(size);
                // Call __aura_alloc
                self.call_external("__aura_alloc");
                Ok(0)
            }
            // FEATURE 1: Explicit memory deallocation
            Expr::Free(ptr, size) => {
                let _ = self.generate_expr(ptr)?;
                // ptr is in rax, move to rdi
                self.mov_rdi_rax();
                let size_val = self.generate_expr(size)?;
                self.mov_rsi_immediate(size_val);
                // Call __aura_free
                self.call_external("__aura_free");
                Ok(0)
            }
            // FEATURE 8: Explicit cast with type checking
            Expr::Cast(expr, target_type) => {
                // FEATURE 8: Generate source expression
                let _ = self.generate_expr(expr)?;
                // Check if cast is allowed (only explicit Cast nodes)
                // Emit appropriate conversion for target type
                self.generate_cast_conversion(target_type)?;
                Ok(0)
            }
            _ => Ok(0),
        }
    }

    fn mov_rax_immediate(&mut self, val: u64) {
        self.text.push(0x48);
        self.text.push(0xb8);
        self.text.extend_from_slice(&val.to_le_bytes());
    }

    fn mov_rax_from_mem(&mut self, addr: u64) {
        self.mov_rax_immediate(addr);
        self.text.push(0x48);
        self.text.push(0x8b);
        self.text.push(0x00);
    }

    fn mov_rax_to_mem(&mut self, addr: u64) {
        self.mov_rax_immediate(addr);
        self.text.push(0x48);
        self.text.push(0x89);
        self.text.push(0x00);
    }

    fn mov_rax_to_mem_via_register(&mut self, addr: u64) {
        self.mov_rax_immediate(addr);
        self.text.push(0x48);
        self.text.push(0x89);
        self.text.push(0x00);
    }

    fn mov_rax_from_mem_via_register(&mut self, addr: u64) {
        self.mov_rax_immediate(addr);
        self.text.push(0x48);
        self.text.push(0x8b);
        self.text.push(0x00);
    }

    fn xor_rax_rax(&mut self) {
        self.text.push(0x48);
        self.text.push(0x31);
        self.text.push(0xc0);
    }

    fn ret(&mut self) {
        self.text.push(0xc3);
    }

    fn generate_syscall(&mut self, method_name: &str, args: &[Expr]) -> Result<(), CodegenError> {
        match method_name {
            "write" => self.generate_write_syscall(args)?,
            _ => {
                return Err(CodegenError {
                    message: format!("Unknown syscall method: {}", method_name),
                })
            }
        }
        Ok(())
    }

    fn get_data_address(&self, offset: usize) -> u64 {
        // For now, assume a fixed data address.
        // This will need to be updated with the actual data segment address.
        let addr = 0x1000000 + offset as u64;
        addr
    }

    fn generate_write_syscall(&mut self, args: &[Expr]) -> Result<(), CodegenError> {
        if args.is_empty() {
            return Err(CodegenError {
                message: "write syscall requires at least one argument".to_string(),
            });
        }

        let fd = if args.len() > 1 {
            match &args[0] {
                Expr::Literal(Literal::Int(val, _)) => *val as u64,
                _ => 1,
            }
        } else {
            1
        };

        let data_arg_idx = if args.len() > 1 { 1 } else { 0 };

        match &args[data_arg_idx] {
            Expr::Literal(Literal::String(bytes)) => {
                let offset = self.data.len();
                self.data.extend_from_slice(bytes);
                let len = bytes.len() as u64;
                let data_addr = self.get_data_address(offset);

                self.mov_rdi_immediate(fd);
                self.mov_rsi_immediate(data_addr);
                self.mov_rdx_immediate(len);
                self.mov_rax_immediate(1);
                self.syscall();
            }
            Expr::Identifier(name) => {
                if let Some(sym) = self
                    .symbols
                    .iter()
                    .find(|s| s.name == *name && s.kind == SymbolKind::Data)
                    .cloned()
                {
                    let len = sym.size;
                    let data_addr = self.get_data_address(sym.offset as usize);

                    self.mov_rdi_immediate(fd);
                    self.mov_rsi_immediate(data_addr);
                    self.mov_rdx_immediate(len);
                    self.mov_rax_immediate(1);
                    self.syscall();
                } else {
                    return Err(CodegenError {
                        message: format!("Unknown identifier: {}", name),
                    });
                }
            }
            _ => {
                return Err(CodegenError {
                    message: "write syscall argument must be a string literal or identifier"
                        .to_string(),
                });
            }
        }

        Ok(())
    }

    fn mov_rdi_immediate(&mut self, val: u64) {
        self.text.push(0x48);
        self.text.push(0xbf);
        self.text.extend_from_slice(&val.to_le_bytes());
    }

    fn mov_rsi_immediate(&mut self, val: u64) {
        self.text.push(0x48);
        self.text.push(0xbe);
        self.text.extend_from_slice(&val.to_le_bytes());
    }
    fn lea_rsi_rip_relative(&mut self, offset: i32) {
        self.text.push(0x48);
        self.text.push(0x8d);
        self.text.push(0x35);
        self.text.extend_from_slice(&offset.to_le_bytes());
    }

    fn mov_rdx_immediate(&mut self, val: u64) {
        self.text.push(0x48);
        self.text.push(0xba);
        self.text.extend_from_slice(&val.to_le_bytes());
    }

    fn mov_r10_immediate(&mut self, val: u64) {
        self.text.push(0x49);
        self.text.push(0xba);
        self.text.extend_from_slice(&val.to_le_bytes());
    }

    fn mov_r10_to_mem(&mut self) {
        self.text.push(0x49);
        self.text.push(0x89);
        self.text.push(0x10);
    }

    fn mov_rax_to_r10_mem(&mut self) {
        self.text.push(0x49);
        self.text.push(0x89);
        self.text.push(0x02); // mov [r10], rax
    }

    fn mov_rax_from_r10(&mut self) {
        self.text.push(0x49);
        self.text.push(0x8b);
        self.text.push(0xc2);
    }

    fn syscall(&mut self) {
        self.text.push(0x0f);
        self.text.push(0x05);
    }

    // FEATURE 1: Emit alloc<T>(count) - puts result in rax
    fn emit_alloc(&mut self, count_expr: &Expr) -> Result<(), CodegenError> {
        let count = self.generate_expr(count_expr)?;
        // Put count in rdi
        self.mov_rdi_immediate(count);
        // Call __aura_alloc(count)
        self.call_external("__aura_alloc");
        Ok(())
    }

    // FEATURE 1: Emit free(ptr) - takes ptr from rax/rax
    fn emit_free(&mut self, ptr_expr: &Expr) -> Result<(), CodegenError> {
        let _ = self.generate_expr(ptr_expr)?;
        // ptr is in rax, move to rdi
        self.mov_rdi_rax();
        // Call __aura_free(ptr)
        self.call_external("__aura_free");
        Ok(())
    }

    // FEATURE 9: Emit immediate value respecting bit width
    fn emit_width_immediate(&mut self, val: u64, bits: u8) {
        match bits {
            1..=8 => {
                // mov al, imm8
                self.text.push(0xb0);
                self.text.push(val as u8);
            }
            9..=16 => {
                // mov ax, imm16
                self.text.push(0x66);
                self.text.push(0xb8);
                self.text.extend_from_slice(&(val as u16).to_le_bytes());
            }
            17..=32 => {
                // mov eax, imm32
                self.text.push(0xb8);
                self.text.extend_from_slice(&(val as u32).to_le_bytes());
            }
            _ => {
                // mov rax, imm64
                self.text.push(0x48);
                self.text.push(0xb8);
                self.text.extend_from_slice(&val.to_le_bytes());
            }
        }
    }

    // FEATURE 1: Move rax to rdi
    fn mov_rdi_rax(&mut self) {
        self.text.push(0x48);
        self.text.push(0x89);
        self.text.push(0xf8);
    }

    // FEATURE 1: Call external function
    fn call_external(&mut self, symbol: &str) {
        match symbol {
            "__aura_alloc" => {
                // call r14
                self.text.push(0x41);
                self.text.push(0xff);
                self.text.push(0xd6);
            }
            "__aura_free" => {
                // call r15
                self.text.push(0x41);
                self.text.push(0xff);
                self.text.push(0xd7);
            }
            _ => {
                // Emit: call [rip + offset]
                self.text.push(0xff);
                self.text.push(0x15);
                // Add relocation for external symbol
                self.relocations.push(Relocation {
                    offset: self.text.len(),
                    symbol: symbol.to_string(),
                    kind: RelocationKind::Relative32,
                });
                self.text.extend_from_slice(&[0u8; 4]);
            }
        }
    }

    // FEATURE 8: Generate explicit cast conversion
    fn generate_cast_conversion(&mut self, target_type: &Type) -> Result<(), CodegenError> {
        match target_type {
            Type::I8 => {
                // movsx eax, al (sign-extend 8-bit to 32-bit)
                self.text.push(0x0f);
                self.text.push(0xbe);
                self.text.push(0xc0);
            }
            Type::U8 => {
                // movzx eax, al (zero-extend 8-bit to 32-bit)
                self.text.push(0x0f);
                self.text.push(0xb6);
                self.text.push(0xc0);
            }
            Type::I16 => {
                // movsx eax, ax
                self.text.push(0x0f);
                self.text.push(0xbf);
                self.text.push(0xc0);
            }
            Type::U16 => {
                // movzx eax, ax
                self.text.push(0x0f);
                self.text.push(0xb7);
                self.text.push(0xc0);
            }
            Type::I32 | Type::U32 => {
                // Already in eax, no conversion needed
            }
            Type::I64 | Type::U64 => {
                // Already in rax, no conversion needed
            }
            Type::BitInt(bits, _) => {
                // FEATURE 9: Apply mask for bit-precise type
                self.mask_rax(*bits);
            }
            _ => {}
        }
        Ok(())
    }

    // FEATURE 9: Apply mask to rax for bit-precise types
    fn mask_rax(&mut self, bits: u8) {
        if bits < 64 {
            // and rax, mask
            let mask = if bits >= 63 {
                u64::MAX
            } else {
                (1u64 << bits) - 1
            };
            self.text.push(0x48);
            self.text.push(0x25);
            self.text.extend_from_slice(&mask.to_le_bytes());
        }
    }
}
