# AlgoSpeak Language Specification

## 1. Overview

AlgoSpeak is a natural-language-inspired programming language designed for writing algorithms. Programs read like plain English instructions, making them accessible to beginners and non-programmers.

**Target**: x86-64 Linux (compiled to NASM assembly)
**Type system**: Implicit — integers and arrays of integers
**Execution model**: Compiled, ahead-of-time, no garbage collection

---

## 2. Lexical Structure

### 2.1 Character Set
AlgoSpeak source files are UTF-8 encoded. Only ASCII characters are significant to the lexer.

### 2.2 Comments
```
# This is a line comment
// This is also a line comment
```

### 2.3 Whitespace
Spaces and tabs are insignificant except as token separators. Newlines are significant — they separate statements.

### 2.4 Keywords
The following words are reserved (case-insensitive):

```
create  set  as  to  if  otherwise  while  for  each  in  end
show  reveal  algorithm  stop  add  subtract  multiply  divide
divided  by  from  and  or  not  is  less  greater  than
equal  equals  of  length  minus  plus  times
```

### 2.5 Identifiers
Identifiers begin with a letter or underscore and may contain letters, digits, and underscores. They are case-sensitive.

### 2.6 Literals
- **Integer literals**: Sequences of digits, e.g. `42`, `0`, `999`
- **Negative numbers**: Expressed as unary minus, e.g. `-1`
- **Array literals**: Bracket-delimited, comma-separated, e.g. `[1, 2, 3]`

---

## 3. Types

AlgoSpeak has two types, both inferred:

| Type | Description |
|------|-------------|
| **Integer** | 64-bit signed integer |
| **Array** | Fixed-length sequence of integers |

There are no explicit type annotations. The compiler infers types from usage.

---

## 4. Variables

### 4.1 Declaration
```
create <name> as <expression>
```
Declares a new variable. The variable must not already exist in the current scope.

### 4.2 Assignment
```
set <name> to <expression>
```
Assigns a new value to an existing variable.

### 4.3 Array Element Assignment
```
set <name>[<index>] to <expression>
```
Assigns a value to a specific array element. Bounds-checked at runtime.

---

## 5. Expressions

### 5.1 Arithmetic Operators

| Operator | Symbolic | Natural Language |
|----------|----------|-----------------|
| Addition | `a + b` | — |
| Subtraction | `a - b` | `a minus b` |
| Multiplication | `a * b` | `a times b` |
| Division | `a / b` | `a divided by b` |

### 5.2 Comparison Operators

| Meaning | Syntax |
|---------|--------|
| Equal | `a equals b` |
| Less than | `a is less than b` |
| Greater than | `a is greater than b` |
| Less or equal | `a is less than or equal to b` |
| Greater or equal | `a is greater than or equal to b` |
| Identity | `a is b` (equality shorthand) |

### 5.3 Logical Operators
```
a and b
a or b
not a
```

### 5.4 Array Access
```
<array>[<index>]
```
Zero-indexed. Bounds-checked at runtime — out-of-bounds access terminates the program with an error message.

### 5.5 Array Length
```
length of <array>
```
Returns the number of elements in an array.

### 5.6 Function Calls
```
<name>(<arg1>, <arg2>, ...)
```

### 5.7 Operator Precedence (lowest to highest)

1. `or`
2. `and`
3. Comparison (`equals`, `is less than`, etc.)
4. Additive (`+`, `-`, `minus`)
5. Multiplicative (`*`, `/`, `divided by`, `times`)
6. Unary (`-`, `not`)
7. Primary (literals, variables, array access, function calls, parenthesized expressions)

---

## 6. Statements

### 6.1 Natural-Language Arithmetic
These statements are shorthand for reassignment:

```
add <expr> to <variable>          →  set variable to variable + expr
subtract <expr> from <variable>   →  set variable to variable - expr
multiply <variable> by <expr>     →  set variable to variable * expr
divide <variable> by <expr>       →  set variable to variable / expr
```

### 6.2 Output
```
show <expression>
```
Prints the integer value followed by a newline to stdout.

### 6.3 Conditional
```
if <condition>
    <statements>
end
```

With else branch:
```
if <condition>
    <statements>
otherwise
    <statements>
end
```

### 6.4 While Loop
```
while <condition>
    <statements>
end
```

### 6.5 For-Each Loop
```
for each <variable> in <array>
    <statements>
end
```
Iterates over every element in the array, assigning each to `<variable>`.

### 6.6 Function Definition
```
algorithm <name>(<param1>, <param2>, ...)
    <statements>
end
```

### 6.7 Return
```
reveal <expression>
```
Returns a value from a function. Immediately exits the function.

### 6.8 Stop
```
stop
```
Exits the innermost loop. If not inside a loop, terminates the program.

---

## 7. Scoping Rules

- Variables are scoped to the block in which they are declared.
- Function parameters are scoped to the function body.
- For-each loop variables are scoped to the loop body.
- Inner scopes can access variables from outer scopes.
- Variables cannot be re-declared in the same scope.

---

## 8. Safety Guarantees

1. **Use-before-declaration**: Compile-time error
2. **Duplicate declaration**: Compile-time error
3. **Undefined function call**: Compile-time error
4. **Function arity mismatch**: Compile-time error
5. **Array out-of-bounds**: Runtime error with diagnostic message
6. **No pointer arithmetic**: Not expressible in the language

---

## 9. Runtime Behavior

- Integers are 64-bit signed (`int64`).
- Division is integer division (truncates toward zero).
- Arrays have fixed size determined at creation.
- The `show` statement prints to stdout.
- Bounds errors print to stderr and exit with code 1.
- Programs exit with code 0 on successful completion.
