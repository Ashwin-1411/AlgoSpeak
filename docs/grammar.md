# AlgoSpeak Grammar

Formal grammar in Extended Backus-Naur Form (EBNF). Terminals are in `'single quotes'` or `UPPER_CASE`.

---

## Program Structure

```ebnf
program        = { NEWLINE } , { statement , { NEWLINE } } ;
```

## Statements

```ebnf
statement      = var_decl
               | assignment
               | show_stmt
               | if_stmt
               | while_stmt
               | for_each_stmt
               | function_def
               | return_stmt
               | stop_stmt
               | natural_add
               | natural_subtract
               | natural_multiply
               | natural_divide
               | expr_stmt ;

var_decl       = 'create' , IDENTIFIER , 'as' , expression ;

assignment     = 'set' , IDENTIFIER , [ '[' , expression , ']' ] , 'to' , expression ;

show_stmt      = 'show' , expression ;

if_stmt        = 'if' , expression , NEWLINE ,
                 { statement , NEWLINE } ,
                 [ 'otherwise' , NEWLINE , { statement , NEWLINE } ] ,
                 'end' ;

while_stmt     = 'while' , expression , NEWLINE ,
                 { statement , NEWLINE } ,
                 'end' ;

for_each_stmt  = 'for' , 'each' , IDENTIFIER , 'in' , IDENTIFIER , NEWLINE ,
                 { statement , NEWLINE } ,
                 'end' ;

function_def   = 'algorithm' , IDENTIFIER , '(' , [ param_list ] , ')' , NEWLINE ,
                 { statement , NEWLINE } ,
                 'end' ;

param_list     = IDENTIFIER , { ',' , IDENTIFIER } ;

return_stmt    = 'reveal' , expression ;

stop_stmt      = 'stop' ;

natural_add    = 'add' , expression , 'to' , IDENTIFIER ;
natural_subtract = 'subtract' , expression , 'from' , IDENTIFIER ;
natural_multiply = 'multiply' , IDENTIFIER , 'by' , expression ;
natural_divide = 'divide' , IDENTIFIER , 'by' , expression ;

expr_stmt      = expression ;
```

## Expressions

```ebnf
expression     = or_expr ;

or_expr        = and_expr , { 'or' , and_expr } ;

and_expr       = comparison , { 'and' , comparison } ;

comparison     = additive , [ comparison_op , additive ] ;

comparison_op  = 'equals'
               | 'is' , 'less' , 'than' , [ 'or' , 'equal' , 'to' ]
               | 'is' , 'greater' , 'than' , [ 'or' , 'equal' , 'to' ]
               | 'is' , additive ;

additive       = multiplicative , { ( '+' | '-' | 'minus' ) , multiplicative } ;

multiplicative = unary , { ( '*' | '/' | 'divided' , 'by' | 'times' ) , unary } ;

unary          = ( '-' | 'not' ) , unary
               | primary ;

primary        = NUMBER
               | IDENTIFIER , [ '[' , expression , ']' ]
               | IDENTIFIER , '(' , [ arg_list ] , ')'
               | '(' , expression , ')'
               | '[' , [ expression , { ',' , expression } ] , ']'
               | 'length' , 'of' , IDENTIFIER ;

arg_list       = expression , { ',' , expression } ;
```

## Lexical Elements

```ebnf
IDENTIFIER     = letter , { letter | digit | '_' } ;
NUMBER         = digit , { digit } ;
NEWLINE        = '\n' ;

letter         = 'a'..'z' | 'A'..'Z' | '_' ;
digit          = '0'..'9' ;
```

## Operator Precedence Table

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (lowest) | `or` | Left |
| 2 | `and` | Left |
| 3 | `equals`, `is less than`, `is greater than`, etc. | None |
| 4 | `+`, `-`, `minus` | Left |
| 5 | `*`, `/`, `divided by`, `times` | Left |
| 6 (highest) | `-` (unary), `not` | Right |
