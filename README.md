# AlgoSpeak

A **natural-language-inspired programming language** designed for writing algorithms and learning data structures. AlgoSpeak compiles directly to **x86-64 NASM assembly** — no JVM, no interpreter, no LLVM — just blazing-fast native code.

## Quick Start

### Build the compiler

```bash
cargo build --release
```

### Write your first program

Create a file `hello.alg`:

```
create numbers as [1, 3, 5, 7, 9]
create total as 0

for each number in numbers
    add number to total
end

show total
```

### Compile and run

```bash
# Compile AlgoSpeak → assembly
cargo run -- build hello.alg

# Assemble and link
nasm -f elf64 hello.asm -o hello.o
ld hello.o -o hello

# Run!
./hello
# Output: 25
```

Or use the integrated run command:

```bash
cargo run -- run hello.alg
```

## CLI Usage

```bash
algoc build <source.alg>     # Compile to assembly (.asm)
algoc run <source.alg>       # Compile, assemble, link, and execute
algoc repl                   # Start interactive REPL
algoc <source.alg>           # Same as 'build' (backward compatible)
```

## Language Overview

AlgoSpeak reads like plain English. Here's what it supports:

| Feature | Syntax |
|---------|--------|
| Variables | `create x as 5` / `set x to 10` |
| Arrays | `create arr as [1, 2, 3]` |
| Arithmetic | `+`, `-`, `*`, `/`, `%` or `add x to y`, `subtract 1 from x` |
| Conditions | `if x equals 5 ... otherwise ... end` |
| While loops | `while x is less than 10 ... end` |
| For-each | `for each item in array ... end` |
| Functions | `algorithm sum(a, b) ... reveal a + b ... end` |
| Output | `show value` |
| Array length | `length of arr` |
| Array access | `arr[i]` (with automatic bounds checking) |
| Stack | `create stack s` / `push 10 into s` / `pop from s` |
| Queue | `create queue q` / `enqueue 5 into q` / `dequeue from q` |
| Sort | `sort array` |
| Reverse | `reverse array` |
| String output | `show "hello world"` |

### Comparison Operators

```
equals                    →  ==
is less than              →  <
is greater than           →  >
is less than or equal to  →  <=
is greater than or equal to → >=
```

### Natural-Language Algorithm Blocks

Define algorithms using natural syntax:

```
algorithm binary_search in numbers for target
    set low to 0
    set high to length of numbers minus 1

    while low is less than or equal to high
        set mid to (low + high) / 2

        if numbers[mid] equals target
            reveal mid
        end

        if numbers[mid] is less than target
            set low to mid + 1
        otherwise
            set high to mid - 1
        end
    end

    reveal -1
end
```

## Built-in Data Structures

### Stack (LIFO)

```
create stack s
push 10 into s
push 20 into s
show pop from s     # Output: 20
show pop from s     # Output: 10
```

### Queue (FIFO)

```
create queue q
enqueue 100 into q
enqueue 200 into q
show dequeue from q    # Output: 100
show dequeue from q    # Output: 200
```

## Standard Library

| Function | Description |
|----------|-------------|
| `length of arr` | Returns the number of elements |
| `sort arr` | Sorts array in ascending order (in-place) |
| `reverse arr` | Reverses array in-place |

## Examples

See the `examples/` directory:

| File | Description | Expected Output |
|------|-------------|-----------------|
| `sum.alg` | Array sum with for-each | `25` |
| `factorial.alg` | Recursive factorial | `120` |
| `binary_search.alg` | Binary search on sorted array | `3` |
| `bubble_sort.alg` | Bubble sort with nested loops | `1 2 3 5 8 9` |
| `stack.alg` | Stack push/pop operations | `30 20 10` |
| `queue.alg` | Queue enqueue/dequeue operations | `100 200 300` |
| `sort_demo.alg` | Sort and display array | `1 2 3 5 8 9` |

## Compiler Architecture

```
AlgoSpeak Source (.alg)
        │
        ▼
   ┌─────────┐
   │  Lexer   │  → Token stream
   └────┬─────┘
        ▼
   ┌─────────┐
   │  Parser  │  → Abstract Syntax Tree
   └────┬─────┘
        ▼
   ┌──────────────┐
   │  Semantic     │  → Validated AST
   │  Analysis     │     (checks variables, types, arity)
   └────┬─────────┘
        ▼
   ┌──────────────┐
   │  AlgoIR       │  → IR instruction stream
   │  Lowering     │
   └────┬─────────┘
        ▼
   ┌──────────────┐
   │  Optimizer    │  → Optimized IR
   │               │     (constant folding, DCE)
   └────┬─────────┘
        ▼
   ┌──────────────┐
   │  Code Gen     │  → NASM x86-64 assembly
   └────┬─────────┘
        ▼
   program.asm
```

### Source Files

```
src/
├── main.rs       — CLI entry point (build / run / repl)
├── token.rs      — Token type definitions
├── lexer.rs      — Tokenizer / scanner
├── ast.rs        — AST node definitions
├── parser.rs     — Recursive descent parser
├── semantic.rs   — Semantic analysis & symbol table
├── ir.rs         — Intermediate representation & lowering
├── optimizer.rs  — IR optimisation passes
├── codegen.rs    — x86-64 NASM code generator
└── repl.rs       — Interactive REPL
```

## Safety Features

1. **Variables must be declared before use** — caught at compile time
2. **Duplicate declarations rejected** — caught at compile time
3. **Array bounds checking** — runtime checks before every access
4. **No pointer arithmetic** — arrays are the only indirect access
5. **Function arity checking** — argument count verified at compile time
6. **Stack overflow / underflow detection** — runtime guards on all push/pop ops
7. **Queue overflow / underflow detection** — runtime guards on enqueue/dequeue
8. **Type checking for data structures** — cannot push to a queue, cannot dequeue from a stack

## Performance

The compiler is intentionally simple and fast:

- **No JVM startup** — native Rust binary
- **No LLVM** — direct AST → assembly, no multi-pass optimization
- **No interpretation** — pure ahead-of-time compilation
- **O(n) compilation** — single pass for each stage
- **Minimal pipeline** — source → tokens → AST → assembly
- **Sub-millisecond** compilation for typical programs
- **Direct syscalls** — no C library dependency in generated code

## Requirements

- **Rust** (for building the compiler)
- **NASM** (for assembling the output)
- **ld** (GNU linker, for linking)
- **Linux x86-64** (target platform)

## License

This project is provided as-is for educational purposes.
