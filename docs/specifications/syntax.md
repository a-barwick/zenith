# Parsed Syntax

## Status

**Specified for active M2.** This document is the observable contract for
Zenith's token-to-immutable-AST boundary. Parser acceptance does not imply name
resolution, type checking, lowering, Apex emission, or runtime compatibility.

## Compilation units and declarations

A compilation unit contains one or more class declarations followed by EOF.
M2 accepts this selected Apex-shaped declaration grammar:

```text
compilation-unit  := class-declaration* EOF
class-declaration := class-modifier* "class" identifier class-suffix* class-body
class-suffix      := "extends" type
                   | "implements" type ("," type)*
class-body        := "{" class-member* "}"
```

Class modifiers are `public`, `private`, `protected`, `global`, `abstract`,
`virtual`, `static`, `final`, `transient`, `with sharing`, `without sharing`,
and `inherited sharing`. Modifiers are retained in source order without
duplicate or legality checks; resolution owns declaration consistency.

Class members are fields, properties, methods, and constructors:

```text
class-member := member-modifier*
                (constructor
                | type identifier member-tail)
member-tail  := ";"                                      // field
              | "=" expression ";"                       // initialized field
              | property-body                             // property
              | "(" parameter-list? ")" block             // method
constructor  := identifier "(" parameter-list? ")" block
property-body := "{" accessor+ "}"
accessor     := member-modifier* ("get" | "set") ";"
parameter-list := parameter ("," parameter)*
parameter   := type identifier
```

Member modifiers are `public`, `private`, `protected`, `global`, `abstract`,
`virtual`, `override`, `static`, `final`, `transient`, `testmethod`, and
`webservice`. Constructor recognition is syntactic: a member beginning with an
identifier immediately followed by `(` is represented as a constructor. M2
does not check whether its name matches the containing class.

Annotations, interfaces, enums, triggers, nested generic declarations, method
type parameters, default parameter values, accessor bodies, and other Apex
declarations are outside M2. They produce localized parse diagnostics rather
than being approximated.

## Types

M2 parses type syntax without resolving names:

```text
type := type-name ("<" type ("," type)* ">")? "?"?
```

A type name is an identifier or one of the type-shaped reserved words:
`any`, `bigdecimal`, `blob`, `boolean`, `byte`, `char`, `currency`, `date`,
`datetime`, `decimal`, `double`, `float`, `id`, `int`, `integer`, `list`,
`long`, `map`, `number`, `object`, `set`, `short`, `sobject`, `string`, `time`,
and `void`. `Id` currently lexes as an identifier and remains source-faithful.

Generic arguments may nest. The parser interprets adjacent `>` tokens in type
context so `Map<String, List<Integer>>` closes both argument lists even though
the lexer emits maximal-munch shift operators. `T?` is reserved and retained in
the AST, but nullability is not checked until M4.

## Statements

Blocks contain zero or more statements:

```text
statement := block
           | variable-declaration ";"
           | expression ";"
           | "if" "(" expression ")" statement ("else" statement)?
           | "while" "(" expression ")" statement
           | "do" statement "while" "(" expression ")" ";"
           | "for" "(" for-control ")" statement
           | "return" expression? ";"
           | "throw" expression ";"
           | "break" ";"
           | "continue" ";"
           | ";"

variable-declaration := type identifier ("=" expression)?
for-control := type identifier ":" expression
             | (variable-declaration | expression-list)? ";"
               expression? ";"
               expression-list?
expression-list := expression ("," expression)*
```

The variable-declaration choice uses bounded syntactic lookahead only. It does
not consult a symbol table or decide whether a type exists. An enhanced `for`
header therefore requires the explicit `type identifier : expression` shape.

## Expressions

M2 records source syntax and precedence without selecting calls, members, or
types. From tightest to loosest, the supported expression forms are:

1. primary expressions: identifiers, the Apex `System` namespace word,
   integers, strings, `true`, `false`, `null`, `this`, `super`,
   parenthesized expressions, and `new type(arguments)`
2. postfix calls, member access, safe member access reservation, indexing,
   postfix `++`, and postfix `--`
3. prefix `!`, `~`, `+`, `-`, `++`, and `--`
4. multiplicative `*`, `/`, `%`
5. additive `+`, `-`
6. shifts `<<`, `>>`, `>>>`
7. relational `<`, `<=`, `>`, `>=`, `instanceof`
8. equality `==`, `!=`, `===`, `!==`
9. bitwise `&`, then `^`, then `|`
10. logical `&&`, then `||`
11. null coalescing `??`
12. conditional `?:`
13. right-associative assignment `=`, `+=`, `-=`, `*=`, `/=`, `&=`, `|=`,
    `^=`, `<<=`, `>>=`, and `>>>=`

Calls may target any parsed expression; target legality belongs to later
phases. Assignment targets likewise remain syntax until checking. Lambda
operators and other reserved operator spellings are not accepted in M2.

## Immutable AST

The public AST owns its parsed data and exposes read-only accessors. No parser
state, token cursor, resolution result, inferred type, effect, schema fact, or
emission choice is stored in the AST. Every node carries one file-aware span
covering its complete source construct, while identifiers retain their exact
spelling, canonical key, and identifier span.

The shared visitor walks all declaration, type, statement, and expression
children in deterministic source order. The visitor API receives shared
references only. Consumers that need semantic facts build a separate
representation or side table rather than mutating syntax.

## Recovery and diagnostics

The public parser result contains an optional compilation unit plus an ordered
diagnostic list. Parsing is attempted only when lexing succeeded.

M2 uses these stable diagnostic codes:

- `parse.expected-declaration`
- `parse.expected-member`
- `parse.expected-type`
- `parse.expected-identifier`
- `parse.expected-token`
- `parse.expected-expression`
- `parse.unsupported-syntax`

All diagnostics are owned by the parse phase and use the unexpected token span,
including the empty EOF span for a missing closing token. Recovery always makes
progress:

- At compilation-unit scope, synchronize at `class`, a class modifier, or EOF.
- In a class body, synchronize after `;`, at the next member modifier/type-like
  token, at `}`, or at EOF.
- In a block, synchronize after `;`, at a statement-start token, at `}`, or at
  EOF.
- A missing `;` before an unambiguous next statement or member reports at that
  boundary without consuming the next construct.
- Balanced block, parenthesis, bracket, and generic delimiters prevent an inner
  error from consuming a containing declaration's closing brace.

The parser may return a partial immutable tree with diagnostics so tooling can
inspect recovered structure. Later semantic phases must not run for a source
unit with parse errors.

## `ast` CLI contract

For a valid file, `zenith ast <file.zen>` prints a deterministic, line-oriented
tree. Each node is one line:

```text
<indent><node-kind> <node-details> @<start-line>:<start-column>..<end-line>:<end-column>
```

Indentation is two ASCII spaces per parent. Node details use source spelling
for names, decoded and JSON-escaped string values, and canonical operator
spellings. Optional or empty children are omitted.

On lexical, parse, or source errors, the command writes deterministic
diagnostics to stderr, writes no AST to stdout, and exits with status 1. CLI
usage errors exit with status 2.

## Acceptance fixtures

- `examples/hello.zen` exercises the smallest class, method, call, member
  access, and string expression slice.
- `examples/lexical-baseline.zen` exercises sharing modifiers, a property,
  generic types, a method, local declarations, enhanced `for`, `if`,
  `continue`, compound assignment, a Unicode string, and `return`.

Both fixtures have stable AST goldens. Negative parser fixtures cover
declaration, member, type, delimiter, expression, and statement recovery.
