# Lexical Structure

## Status

**Specified for M1; not implemented.** This document is the observable contract
for the first source-to-token slice. Tokenization does not imply that later
parser, checker, or emitter phases accept a construct.

## Source text and positions

- `.zen` files are UTF-8. Invalid UTF-8 is a source diagnostic, not a panic.
- Spans remain half-open byte ranges in a single `SourceId`.
- Rendered locations are 1-based lines and 1-based Unicode-scalar columns.
- `\r\n` is one line break. A lone `\r` or `\n` is also one line break.
- A tab advances one reported column in M1. Display-cell and LSP UTF-16
  conversions are separate future views over the same byte span.
- Lexer cursors advance only at UTF-8 character boundaries.

Unicode is allowed in comments and string contents. M1 identifiers are ASCII so
their canonical form is deterministic and compatible with the Apex target.

## Words and names

An identifier-shaped word begins with an ASCII letter and continues with ASCII
letters, digits, or underscores:

```text
[A-Za-z][A-Za-z0-9_]*
```

The lexer preserves exact spelling and stores an ASCII-lowercase canonical key.
Leading underscores and non-ASCII identifier characters produce lexical
diagnostics. Consecutive or trailing underscores remain lexically valid because
Salesforce API names such as `Field__c` require them; declaration-specific Apex
naming restrictions belong to resolution or schema checking.

Keyword matching is case-insensitive. M1 uses one centralized word table with
three classes:

1. The Apex reserved-word baseline reviewed in the
   [Salesforce Apex Developer Guide](https://resources.docs.salesforce.com/latest/latest/en-us/sfdc/pdf/salesforce_apex_developer_guide.pdf)
   on 2026-07-16:

   ```text
   abstract activate and any array as asc autonomous begin bigdecimal blob
   boolean break bulk by byte case cast catch char class collect commit const
   continue currency date datetime decimal default delete desc do double else
   end enum exception exit export extends false final finally float for from
   global goto group having hint if implements import in inner insert int
   integer interface into instanceof join like limit list long loop map merge
   new not null nulls number object of on or outer override package parallel
   pragma private protected public retrieve return rollback select set short
   sobject sort static string super switch synchronized system testmethod then
   this throw time transaction transient trigger true try undelete update
   upsert using virtual void webservice when where while
   ```

2. Context-sensitive Apex words needed by the baseline grammar, including
   `after`, `before`, `count`, `excludes`, `first`, `get`, `includes`,
   `inherited`, `last`, `order`, `sharing`, `with`, and `without`. They receive
   a distinct `contextual(<canonical>)` token kind. The parser decides whether
   a contextual word acts as grammar or as an identifier where Apex permits
   both; it never asks the lexer to reclassify the source.
3. Currently documented Zenith words: `effects`, `fn`, `let`, `query`, and
   `record`.

Adding or reclassifying a word is a compatibility change and updates this
specification plus executable keyword-casing tests. Platform type and schema
names such as `Account` are not lexical keywords; resolution owns them.

## Literals

M1 recognizes:

- Decimal integer spellings consisting of one or more ASCII digits. The lexer
  preserves the spelling and does not choose `Integer`, `Long`, or a range;
  typing owns numeric interpretation.
- Single-quoted strings. Supported Apex escapes are `\b`, `\t`, `\n`, `\f`,
  `\r`, `\"`, `\'`, and `\\`.
- `true`, `false`, and `null` through case-insensitive keyword classification.

An unknown escape is a lexical diagnostic. A physical line break before the
closing quote terminates the invalid string and lets recovery continue on the
next line. A double-quoted sequence receives a dedicated diagnostic and is
consumed through its closing quote or line end so it does not create a cascade
of character errors.

Decimal fractions, exponents, hexadecimal values, date/time forms, and other
literal families are not M1 literals. Their characters may tokenize into the
ordinary M1 pieces, but no later phase may accept them without a specification
and explicit implementation.

## Comments and trivia

- Space, tab, form feed, `\r`, `\n`, and `\r\n` are trivia.
- `//` comments extend through, but do not consume, the line break.
- `/* ... */` comments do not nest.
- An unterminated block comment reports from the opening delimiter through EOF.
- Trivia and comments are omitted from the ordinary token stream.

Other Unicode whitespace is rejected in M1 until compatibility evidence
justifies accepting it.

## Punctuation and operators

M1 recognizes these individual or maximal-munch spellings. The first group is
the selected Apex-compatible baseline; the final line reserves spellings used
by documented Zenith syntax even when the M1/M2 parser does not yet accept
them:

```text
( ) { } [ ] ; , . : @
= == === != !== < <= > >=
+ ++ += -- - -= * *= / /= %
&& || ! & &= | |= ^ ^= ~ << <<= >> >>= >>> >>>=
? ?.
=> -> ??
```

Comment openers take precedence over `/` and `/=`. Longer valid spellings take
precedence over their prefixes. Recognition only reserves a boundary; parser
and later-phase support remain governed by their milestones.

An adjacent spelling that is not in this inventory is not silently promoted to
an operator. Each maximal valid token is emitted separately; for example,
`%=` becomes `%` followed by `=`, while `===` is one token. Adding a combined
spelling is an observable lexical compatibility change.

Every token carries its kind and span. Identifiers additionally carry exact and
canonical spelling. String tokens carry the decoded value; their exact source
spelling remains recoverable from the source map. An explicit EOF token uses an
empty span at the source byte length.

## Recovery and diagnostics

The public lexer result contains both tokens and an ordered diagnostic list.
Lexing guarantees forward progress. Word recovery is defined precisely:

- After an ASCII-letter start, the lexer consumes the maximal run of Unicode
  alphanumeric scalars and `_`. The run is a valid identifier only when every
  scalar matches the ASCII identifier grammar. Thus `fooébar` produces one
  `lex.invalid-identifier` diagnostic for the complete run.
- A leading `_` or non-ASCII alphanumeric scalar consumes the same maximal run
  and produces one `lex.invalid-identifier` diagnostic. Thus `_foo` and
  `éclair` each produce one diagnostic rather than a cascade.
- An ASCII digit starts an integer and consumes only following ASCII digits.
  Consequently `123abc` tokenizes as `integer("123")` followed by
  `identifier(abc)`; adjacency legality belongs to parsing.
- A non-ASCII scalar that is neither alphanumeric nor part of a recognized
  token is one invalid character. Combining marks therefore recover one scalar
  at a time in M1.

The remaining recovery boundaries are:

- An invalid character reports one diagnostic and consumes that character.
- An invalid or unterminated string consumes through its recovery boundary.
- An unterminated block comment consumes through EOF.

The lexer still appends EOF after recoverable errors. Diagnostics follow
`diagnostics.md` and are sorted by source order, span start, and stable code.

## `tokens` CLI contract

For a valid file, `zenith tokens <file.zen>` prints one token per line:

```text
<start-line>:<start-column>..<end-line>:<end-column>\t<kind>\t"<source spelling>"
```

The spelling uses JSON-style escapes. Stable kind forms are
`keyword(<canonical>)`, `contextual(<canonical>)`,
`identifier(<canonical>)`, `integer`, `string`, `punctuation(<spelling>)`,
`operator(<spelling>)`, and `eof`.

Example:

```text
1:1..1:7	keyword(public)	"public"
1:8..1:13	keyword(class)	"class"
1:14..1:25	identifier(hellozenith)	"HelloZenith"
2:1..2:5	contextual(with)	"with"
```

On lexical or source errors, the command writes deterministic diagnostics to
stderr, writes no token stream to stdout, and exits with status 1. CLI usage
errors exit with status 2. The first implemented token fixture becomes a golden
test so formatting changes are intentional.

## Acceptance fixtures

- `examples/hello.zen` is the smallest smoke fixture.
- `examples/lexical-baseline.zen` is the M1/M2 Apex-shaped token baseline.

Neither fixture is a claim that parsing, checking, or Apex emission exists.
