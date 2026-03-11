# AlgoSpeak Compiler Architecture

## Overview

The AlgoSpeak compiler translates natural-language-inspired source code into x86-64 NASM assembly for Linux. The architecture follows a classic multi-stage pipeline design, optimized for simplicity and speed.

```
AlgoSpeak Source (.alg)
        │
        ▼
   ┌─────────┐
   │  Lexer   │  Character stream → Token stream
   └────┬─────┘
        ▼
   ┌─────────┐
   │  Parser  │  Token stream → Abstract Syntax Tree (AST)
   └────┬─────┘
        ▼
   ┌──────────────┐
   │  Semantic     │  AST → Validated AST
   │  Analysis     │  (symbol tables, type checking)
   └────┬─────────┘
        ▼
   ┌──────────────┐
   │  IR Lowering  │  AST → AlgoIR instruction stream
   └────┬─────────┘
        ▼
   ┌──────────────┐
   │  Optimizer    │  AlgoIR → Optimized AlgoIR
   │               │  (constant folding, DCE, strength reduction)
   └────┬─────────┘
        ▼
   ┌──────────────┐
   │  Code Gen     │  AST → NASM x86-64 assembly
   └────┬─────────┘
        ▼
   program.asm
```

---

## Stage 1: Lexer (`lexer.rs`)

The lexer (also called the scanner or tokenizer) reads the source code character by character and produces a flat sequence of tokens.

### Design Decisions

- **Line-oriented**: Newlines are emitted as `Newline` tokens so the parser can use them as statement terminators, similar to Python.
- **Case-insensitive keywords**: All keywords are detected by lowercasing the scanned identifier.
- **Allocation-light**: Only allocates for identifier names and string literal values.
- **Comment support**: Both `#` and `//` line comments.

### Token Categories

| Category | Examples |
|----------|---------|
| Keywords | `create`, `set`, `if`, `while`, `algorithm`, `push`, `sort`, ... |
| Literals | `42` (integer), `"hello"` (string) |
| Identifiers | `x`, `numbers`, `my_var` |
| Symbols | `+`, `-`, `*`, `/`, `%`, `(`, `)`, `[`, `]`, `,` |
| Special | `NEWLINE`, `EOF` |

---

## Stage 2: Parser (`parser.rs`)

The parser transforms the flat token stream into a tree structure (AST) using **recursive descent** parsing.

### Precedence Levels

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (lowest) | `or` | Left |
| 2 | `and` | Left |
| 3 | `equals`, `is less than`, `is greater than`, etc. | None |
| 4 | `+`, `-`, `minus` | Left |
| 5 | `*`, `/`, `%`, `divided by`, `times` | Left |
| 6 (highest) | `-` (unary), `not` | Right |

### Statement Types

The parser recognizes: variable declarations, assignments, if/otherwise/end, while/end, for-each/end, function definitions, return, stop, natural-language arithmetic, and data structure operations (push, pop, enqueue, dequeue, sort, reverse).

### Function Definition Syntax

Two forms are supported:

1. **Standard**: `algorithm name(param1, param2) ... end`
2. **Natural-language**: `algorithm name in param1 for param2 ... end`

Both produce the same AST node.

---

## Stage 3: Semantic Analysis (`semantic.rs`)

The semantic analyzer performs a single-pass walk over the AST to enforce safety rules before code generation.

### Symbol Table

A scope stack (`Vec<HashMap<String, SymbolKind>>`) tracks variable visibility:

- Each block (`if`, `while`, `for`, function) pushes a new scope.
- Variable resolution walks scopes from innermost to outermost.
- Duplicate declarations within the same scope are rejected.

### Symbol Kinds

| Kind | Description |
|------|-------------|
| `Variable` | Scalar integer variable |
| `Array` | Fixed-length integer array |
| `Stack` | LIFO data structure |
| `Queue` | FIFO data structure |
| `Function { arity }` | Function with known parameter count |

### Checks Performed

1. Variables must be declared before use
2. No duplicate declarations in the same scope
3. Functions must be defined before called
4. Function call arity must match definition
5. Push/pop targets must be stacks
6. Enqueue/dequeue targets must be queues
7. Sort/reverse targets must be arrays

---

## Stage 4: IR Lowering (`ir.rs`)

The IR layer defines a flat, **stack-machine** intermediate representation called AlgoIR.

### Instruction Categories

| Category | Instructions |
|----------|-------------|
| Constants & Variables | `LoadConst(i64)`, `LoadVar(name)`, `StoreVar(name)` |
| Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate` |
| Comparison | `CmpEq`, `CmpNe`, `CmpLt`, `CmpGt`, `CmpLe`, `CmpGe` |
| Logic | `LogicAnd`, `LogicOr` |
| Control Flow | `Label(name)`, `Jump(name)`, `JumpIfZero(name)` |
| Functions | `Call(name, arity)`, `Return`, `FuncBegin`, `FuncEnd` |
| I/O | `PrintInt`, `PrintNewline`, `PrintString(text)` |
| Arrays | `AllocArray`, `StoreArrayElem`, `LoadArrayElem`, `LoadArrayLen` |
| Data Structures | `AllocStack`, `StackPush`, `StackPop`, `AllocQueue`, `QueueEnqueue`, `QueueDequeue` |
| Stdlib | `SortArray`, `ReverseArray` |

### Example IR Output

For `create x as 2 + 3`:
```
LoadConst(2)
LoadConst(3)
Add
StoreVar("x")
```

After constant folding:
```
LoadConst(5)
StoreVar("x")
```

---

## Stage 5: Optimizer (`optimizer.rs`)

The optimizer runs three passes over the IR instruction stream:

### Pass 1: Constant Folding

Evaluates constant expressions at compile time.

```
Before:  LoadConst(2), LoadConst(3), Add
After:   LoadConst(5)
```

### Pass 2: Strength Reduction

Replaces expensive operations with cheaper alternatives.

```
x * 2  →  x + x    (at the codegen level)
```

### Pass 3: Dead Code Elimination

Removes stores to variables that are never read.

```
Before:  LoadConst(10), StoreVar("unused_var"), LoadConst(5), StoreVar("x")
After:   LoadConst(5), StoreVar("x")
```

---

## Stage 6: Code Generation (`codegen.rs`)

The code generator walks the AST and emits NASM-compatible x86-64 assembly.

### Variable Storage

Variables are stored on the stack relative to RBP:
- **Scalars**: 1 slot (8 bytes) at `[rbp + offset]`
- **Arrays**: `[length, elem0, elem1, ...]` — (1 + N) contiguous slots
- **Stacks**: `[capacity, top, elem0, elem1, ...]` — (2 + 256) slots
- **Queues**: `[capacity, head, tail, count, elem0, ...]` — (4 + 256) slots

### Expression Evaluation

Expressions evaluate their result into RAX. Binary operations use a push/pop pattern:

```nasm
; x + y
mov rax, [rbp-8]    ; load x
push rax
mov rax, [rbp-16]   ; load y
mov rcx, rax
pop rax
add rax, rcx        ; result in rax
```

### Calling Convention

Functions follow the System V AMD64 ABI:
- Arguments passed in: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
- Return value in: `rax`

### Runtime Routines

The generated assembly includes built-in runtime routines:

| Routine | Description |
|---------|-------------|
| `_print_int` | Converts RAX to decimal string and writes to stdout |
| `_print_newline` | Writes a newline character to stdout |
| `_bounds_error` | Prints array bounds error to stderr and exits |
| `_stack_overflow_error` | Prints stack overflow error and exits |
| `_stack_underflow_error` | Prints stack underflow error and exits |
| `_queue_overflow_error` | Prints queue overflow error and exits |
| `_queue_underflow_error` | Prints queue underflow error and exits |
| `_sort_array` | In-place bubble sort |
| `_reverse_array` | In-place array reversal |

---

## REPL (`repl.rs`)

The REPL (Read-Eval-Print Loop) provides interactive execution:

1. Reads AlgoSpeak code until a blank line
2. Compiles through the full pipeline
3. Writes a temporary `.asm` file
4. Shells out to `nasm` and `ld` to assemble/link
5. Executes the resulting binary and displays output
6. Cleans up temporary files

---

## Performance Analysis

| Factor | Design Choice | Impact |
|--------|--------------|--------|
| **No JVM** | Native Rust binary | Zero startup overhead |
| **No LLVM** | Direct AST → assembly | No multi-pass optimization overhead |
| **No interpreter** | Pure ahead-of-time compilation | No dispatch overhead at runtime |
| **O(n) per stage** | Each stage is a single pass | Linear compilation time |
| **No GC** | Stack-only allocation | Zero GC pauses |
| **Direct syscalls** | No libc linking | Minimal binary size |

Typical compilation time for programs under 100 lines: **< 1ms**.

---

## File Structure

```
algospeak/
│
├── src/
│   ├── main.rs         — CLI entry point (build / run / repl)
│   ├── token.rs        — Token type definitions (50+ token kinds)
│   ├── lexer.rs        — Character-by-character tokenizer
│   ├── ast.rs          — AST node definitions (20+ node types)
│   ├── parser.rs       — Recursive descent parser
│   ├── semantic.rs     — Semantic analysis & symbol table
│   ├── ir.rs           — AlgoIR instruction set & AST→IR lowering
│   ├── optimizer.rs    — Constant folding, DCE, strength reduction
│   ├── codegen.rs      — x86-64 NASM assembly emitter
│   └── repl.rs         — Interactive REPL
│
├── examples/
│   ├── binary_search.alg
│   ├── bubble_sort.alg
│   ├── factorial.alg
│   ├── sum.alg
│   ├── stack.alg
│   ├── queue.alg
│   └── sort_demo.alg
│
├── README.md
├── LANGUAGE_SPEC.md
└── COMPILER_ARCHITECTURE.md
```
