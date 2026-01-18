# Aura Language Specification

## Philosophy

Aura is a systems programming language designed for:
- OS kernels
- Bootloaders
- Bare-metal programming
- High-performance applications

**Core principles:**
- Everything is explicit
- No hidden runtime
- No garbage collector
- No implicit allocations
- ABI-aware
- Predictable behavior

## Mutability Rules (Inverted)

Aura intentionally inverts common mutability semantics:

| Keyword | Meaning | Mutability |
|---------|---------|------------|
| `const` | Immutable binding | **Mutable value** |
| `var` | Mutable binding | **Immutable value** |

```aura
const counter: i32 = 0     // counter is fixed, value can change
counter += 1               // ✅ Valid

var max_limit: i32 = 10    // max_limit can change, value is fixed
max_limit += 1             // ❌ Compile error
```

## Lexical Structure

### Keywords

```
fn      const  var     if      else    while   for
return  break  continue struct  union   enum    sizeof
alignof offsetof asm     noreturn pub     priv
```

### Identifiers

```
[a-zA-Z_][a-zA-Z0-9_]*
```

### Literals

- Integer: `123`, `0x7B`, `0o173`, `0b01111011`, `123i32`, `123u64`
- Float: `3.14`, `3.14f32`, `1e10`
- Character: `'a'`, `'\n'`, `'\x7F'`
- String: `"hello"` (raw bytes, no escaping except standard C escapes)

### Operators

```
+  -  *  /  %  <<  >>  &  |  ^  ~
+= -= *= /= %= <<= >>= &= |= ^=
== != <  >  <= >=
&& || !
&  *  []  .  ->  ?:  ,  ;  :  (  )  {  }  [  ]
```

## Type System

### Primitive Types

| Type | Size | Description |
|------|------|-------------|
| `i8`, `i16`, `i32`, `i64` | 1/2/4/8 | Signed integers |
| `u8`, `u16`, `u32`, `u64` | 1/2/4/8 | Unsigned integers |
| `f32`, `f64` | 4/8 | Floating point |
| `bool` | 1 | Boolean (`true`/`false`) |
| `void` | 0 | No value |
| `noreturn` | 0 | Never returns |

### Pointer Types

```aura
*u8          // Pointer to u8
**i32        // Pointer to pointer to i32
*const u8    // Pointer to const u8
```

### Array Types

```aura
[10]i32      // Array of 10 i32s
[*]i32       // Pointer to array (runtime length)
```

### Function Types

```aura
fn(i32, i32) i32              // Function taking two i32s, returning i32
fn(ptr: *u8, len: usize) i32  // Named parameters
```

### Struct Types

```aura
struct Point {
    x: f64,
    y: f64,
}

struct Header {
    magic: u32,
    size: u32,
    next: *Header,
}
```

### Union Types

```aura
union Value {
    as_i32: i32,
    as_f32: f32,
    as_ptr: *void,
}
```

### Enum Types

```aura
enum Opcode {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
}
```

## Expressions

### Primary Expressions

```aura
identifier          // Variable/constant access
literal             // Integer, float, char, string
( expression )      // Grouped expression
fn_call             // Function call
```

### Unary Expressions

```aura
- expr              // Negation
! expr              // Logical NOT
~ expr              // Bitwise NOT
* expr              // Dereference
& expr              // Address-of
```

### Binary Expressions

```aura
expr + expr
expr - expr
expr * expr
expr / expr
expr % expr
expr << expr
expr >> expr
expr & expr
expr | expr
expr ^ expr
```

### Comparison

```aura
expr == expr
expr != expr
expr < expr
expr > expr
expr <= expr
expr >= expr
```

### Logical

```aura
expr && expr        // Logical AND (short-circuit)
expr || expr        // Logical OR (short-circuit)
```

### Index and Field Access

```aura
array[index]        // Index access
struct.field        // Field access
ptr->field          // Pointer field access
```

### Cast

```aura
expr as target_type
```

## Statements

### Variable Declarations

```aura
const name: type = value;     // Mutable value, immutable binding
var name: type = value;       // Immutable value, mutable binding
```

### Constant Declarations

```aura
const PI: f64 = 3.14159265359;
```

### Assignment

```aura
// Only valid for `const` declarations (mutable values)
const x: i32 = 0;
x = 10;                       // ✅ Valid

// Invalid for `var` declarations
var y: i32 = 5;
y = 20;                       // ❌ Compile error
```

### If Statements

```aura
if condition {
    // ...
} else if other_condition {
    // ...
} else {
    // ...
}
```

### While Loops

```aura
while condition {
    // ...
}
```

### For Loops

```aura
for i: i32 = 0; i < 10; i += 1 {
    // ...
}
```

### Return Statement

```aura
return expression;
```

### Break/Continue

```aura
break;        // Exit loop
continue;     // Next iteration
```

### Block Statement

```aura
{
    const x: i32 = 5;
    // ...
}
```

### Inline Assembly

```aura
asm("mov $$1, %eax");
asm("syscall" :: "a"(syscall_number) :: "memory");
```

## Functions

```aura
fn add(a: i32, b: i32) i32 {
    return a + b;
}

fn noreturn exit(code: i32) -> noreturn {
    asm("mov $$60, %rax" :: "D"(code));
    asm("syscall");
}
```

### Function Attributes

```aura
fn main() i32 {
    // Entry point
}

noreturn fn panic(msg: *u8) -> noreturn {
    // Never returns
}
```

## Memory Layout

### Alignment

```aura
struct Packed {
    a: u8,      // Offset 0
    b: i32,     // Offset 4 (packed, no padding)
}

struct Aligned {
    a: u8,      // Offset 0
    _padding: [3]u8,  // Offset 1-3
    b: i32,     // Offset 4
}
```

### Size and Alignment Builtins

```aura
const size: usize = sizeof(i32);      // 4
const align: usize = alignof(i32);    // 4
const offset: usize = offsetof(Point, y);  // 8
```

## Binary Format (.aura)

### Header

```
Offset  Size  Field
0       4     Magic (0x41555241 "AURA")
4       1     Version (1)
5       1     Flags
6       2     Reserved
8       8     Entry Point RVA
16      8     Stack Size
24      8     Text Offset
32      8     Text Size
40      8     Data Offset
48      8     Data Size
56      8     BSS Size
64      8     Relocation Count
72      8     Symbol Count
```

### Sections

1. **Text Section**: Executable code
2. **Data Section**: Initialized data
3. **Relocation Table**: Fixups for absolute addresses
4. **Symbol Table**: Debug info (optional)

## ABI Requirements (x86_64 System V)

- Stack aligned to 16 bytes at function call
- Red zone: 128 bytes below %rsp
- Arguments: %rdi, %rsi, %rdx, %rcx, %r8, %r9
- Return: %rax (integer), %xmm0 (float)
- Callee-saved: %rbx, %r12, %r13, %r14, %r15, %rbp
- Caller-saved: %rax, %rcx, %rdx, %rsi, %rdi, %r8, %r9, %r10, %r11

## Example Programs

### Hello World (Linux syscall)

```aura
fn main() i32 {
    const message: *u8 = "Hello from Aura\n";
    const len: usize = 17;

    // write(fd, buf, count)
    asm("mov $$1, %rdi" :::);
    asm("mov %0, %%rsi" :: "r"(message) :);
    asm("mov %0, %%rdx" :: "r"(len) :);
    asm("mov $$1, %rax" :::);      // sys_write
    asm("syscall" :::);

    return 0;
}
```

### Syscall Wrapper

```aura
fn syscall3(num: usize, a1: usize, a2: usize, a3: usize) usize {
    asm("syscall"
        : "=a"({result})
        : "a"(num), "D"(a1), "S"(a2), "d"(a3)
        : "rcx", "r11", "memory");
    return result;
}

fn write(fd: i32, buf: *void, count: usize) i32 {
    return syscall3(1, fd as usize, count as usize, count) as i32;
}
```

### Pointer Arithmetic

```aura
fn memcopy(dest: *void, src: *void, count: usize) *void {
    const d: *u8 = dest as *u8;
    const s: *u8 = src as *u8;

    var i: usize = 0;
    while i < count {
        d[i] = s[i];
        i += 1;
    }

    return dest;
}
```
