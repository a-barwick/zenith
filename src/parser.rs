use crate::ast::{
    AccessorKind, Block, ClassDeclaration, ClassMember, CompilationUnit, ConstructorDeclaration,
    Expression, ExpressionKind, FieldDeclaration, ForInitializer, MethodDeclaration, Modifier,
    Operator, Parameter, PropertyAccessor, PropertyDeclaration, Statement, StatementKind, Type,
    TypeName, VariableDeclaration,
};
use crate::diagnostic::{Diagnostic, Phase};
use crate::source::{SourceFile, Span};
use crate::token::{Identifier, Token, TokenKind};

const TYPE_KEYWORDS: &[&str] = &[
    "any",
    "bigdecimal",
    "blob",
    "boolean",
    "byte",
    "char",
    "currency",
    "date",
    "datetime",
    "decimal",
    "double",
    "float",
    "int",
    "integer",
    "list",
    "long",
    "map",
    "number",
    "object",
    "set",
    "short",
    "sobject",
    "string",
    "time",
    "void",
];

const SIMPLE_CLASS_MODIFIERS: &[&str] = &[
    "public",
    "private",
    "protected",
    "global",
    "abstract",
    "virtual",
    "static",
    "final",
    "transient",
];

const SIMPLE_MEMBER_MODIFIERS: &[&str] = &[
    "public",
    "private",
    "protected",
    "global",
    "abstract",
    "virtual",
    "override",
    "static",
    "final",
    "transient",
    "testmethod",
    "webservice",
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParseResult {
    pub unit: Option<CompilationUnit>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Parses an already successful lexical token stream into immutable syntax.
pub fn parse(file: &SourceFile, tokens: &[Token]) -> ParseResult {
    Parser::new(file, tokens).run()
}

struct Parser<'a> {
    file: &'a SourceFile,
    tokens: &'a [Token],
    cursor: usize,
    pending_type_closers: Vec<Span>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    fn new(file: &'a SourceFile, tokens: &'a [Token]) -> Self {
        Self {
            file,
            tokens,
            cursor: 0,
            pending_type_closers: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> ParseResult {
        if self.tokens.is_empty() {
            return ParseResult {
                unit: None,
                diagnostics: vec![
                    Diagnostic::coded_error(
                        Phase::Parse,
                        "parse.expected-declaration",
                        "expected a class declaration",
                        None,
                    )
                    .with_help("pass the complete token stream including EOF"),
                ],
            };
        }

        let start = self.current_span();
        let mut declarations = Vec::new();
        while !self.at_eof() {
            let checkpoint = self.cursor;
            if self.at_class_start() {
                if let Some(declaration) = self.parse_class_declaration() {
                    declarations.push(declaration);
                }
            } else {
                self.error(
                    "parse.expected-declaration",
                    "expected a class declaration",
                    "class declarations begin with `class` or a class modifier",
                );
                self.synchronize_declaration();
            }
            self.ensure_progress(checkpoint);
        }
        let span = join_spans(start, self.current_span());
        ParseResult {
            unit: Some(CompilationUnit::new(declarations, span)),
            diagnostics: self.diagnostics,
        }
    }

    fn parse_class_declaration(&mut self) -> Option<ClassDeclaration> {
        let start = self.current_span();
        let modifiers = self.parse_modifiers(ModifierContext::Class);
        if self.take_keyword("class").is_none() {
            self.error(
                "parse.expected-declaration",
                "expected `class` after class modifiers",
                "a top-level declaration must be a class",
            );
            self.synchronize_declaration();
            return None;
        }
        let name = self.parse_identifier("class name")?;

        let extends = if self.take_keyword("extends").is_some() {
            self.parse_type()
        } else {
            None
        };

        let mut implements = Vec::new();
        if self.take_keyword("implements").is_some() {
            if let Some(implemented) = self.parse_type() {
                implements.push(implemented);
            }
            while self.take_punctuation(",").is_some() {
                if let Some(implemented) = self.parse_type() {
                    implements.push(implemented);
                } else {
                    break;
                }
            }
        }

        if self.take_punctuation("{").is_none() {
            self.expected_token("`{` to begin the class body");
            self.synchronize_declaration();
            return None;
        }

        let mut members = Vec::new();
        while !self.at_punctuation("}") && !self.at_eof() {
            let checkpoint = self.cursor;
            if let Some(member) = self.parse_class_member() {
                members.push(member);
            } else {
                self.synchronize_member();
            }
            self.ensure_progress(checkpoint);
        }

        let end = if let Some(close) = self.take_punctuation("}") {
            close
        } else {
            self.expected_token("`}` to close the class body");
            self.current_span()
        };

        Some(ClassDeclaration::new(
            modifiers,
            name,
            extends,
            implements,
            members,
            join_spans(start, end),
        ))
    }

    fn parse_class_member(&mut self) -> Option<ClassMember> {
        if self.at_punctuation("@") {
            self.error(
                "parse.unsupported-syntax",
                "annotations are not supported in M2",
                "remove the annotation or wait for an explicitly specified declaration slice",
            );
            return None;
        }

        let start = self.current_span();
        let modifiers = self.parse_modifiers(ModifierContext::Member);

        if self.at_name() && self.next_is_punctuation("(") {
            let name = self.parse_identifier("constructor name")?;
            let (parameters, _) = self.parse_parameters()?;
            let body = self.parse_block()?;
            let span = join_spans(start, body.span());
            return Some(ClassMember::Constructor(ConstructorDeclaration::new(
                modifiers, name, parameters, body, span,
            )));
        }

        let Some(ty) = self.parse_type() else {
            if self.at_eof() || self.at_punctuation("}") {
                return None;
            }
            self.error(
                "parse.expected-member",
                "expected a field, property, method, or constructor",
                "class members begin with an optional modifier followed by a type or constructor name",
            );
            return None;
        };
        let name = self.parse_identifier("member name")?;

        if self.at_punctuation("(") {
            let (parameters, _) = self.parse_parameters()?;
            if self.at_punctuation(";") {
                self.error(
                    "parse.unsupported-syntax",
                    "bodyless methods are not supported in M2",
                    "provide a method block for the M2 parsed baseline",
                );
                self.bump();
                return None;
            }
            let body = self.parse_block()?;
            let span = join_spans(start, body.span());
            return Some(ClassMember::Method(MethodDeclaration::new(
                modifiers, ty, name, parameters, body, span,
            )));
        }

        if self.at_punctuation("{") {
            let (accessors, end) = self.parse_property_body();
            return Some(ClassMember::Property(PropertyDeclaration::new(
                modifiers,
                ty,
                name,
                accessors,
                join_spans(start, end),
            )));
        }

        let initializer = if self.take_operator("=").is_some() {
            self.parse_expression()
        } else {
            None
        };
        let end = self.expect_member_semicolon();
        Some(ClassMember::Field(FieldDeclaration::new(
            modifiers,
            ty,
            name,
            initializer,
            join_spans(start, end),
        )))
    }

    fn parse_property_body(&mut self) -> (Vec<PropertyAccessor>, Span) {
        let open = self
            .take_punctuation("{")
            .expect("property body caller checked opening brace");
        let mut accessors = Vec::new();
        while !self.at_punctuation("}") && !self.at_eof() {
            let checkpoint = self.cursor;
            let start = self.current_span();
            let modifiers = self.parse_modifiers(ModifierContext::Member);
            let kind = if self.take_contextual("get").is_some() {
                Some(AccessorKind::Get)
            } else if self.take_keyword("set").is_some() {
                Some(AccessorKind::Set)
            } else {
                None
            };

            if let Some(kind) = kind {
                let end = if let Some(semicolon) = self.take_punctuation(";") {
                    semicolon
                } else {
                    self.expected_token("`;` after the property accessor");
                    self.current_span()
                };
                accessors.push(PropertyAccessor::new(
                    modifiers,
                    kind,
                    join_spans(start, end),
                ));
            } else {
                self.error(
                    "parse.expected-member",
                    "expected `get` or `set` property accessor",
                    "M2 property accessors end with `;` and do not have bodies",
                );
                self.synchronize_member();
            }
            self.ensure_progress(checkpoint);
        }

        let end = if let Some(close) = self.take_punctuation("}") {
            close
        } else {
            self.expected_token("`}` to close the property body");
            self.current_span()
        };
        (accessors, join_spans(open, end))
    }

    fn parse_parameters(&mut self) -> Option<(Vec<Parameter>, Span)> {
        let open = self.take_punctuation("(")?;
        let mut parameters = Vec::new();
        if !self.at_punctuation(")") {
            loop {
                let ty = self.parse_type()?;
                let name = self.parse_identifier("parameter name")?;
                let span = join_spans(ty.span(), name.span());
                parameters.push(Parameter::new(ty, name, span));
                if self.take_punctuation(",").is_none() {
                    break;
                }
            }
        }
        let close = if let Some(close) = self.take_punctuation(")") {
            close
        } else {
            self.expected_token("`)` after the parameter list");
            self.current_span()
        };
        Some((parameters, join_spans(open, close)))
    }

    fn parse_type(&mut self) -> Option<Type> {
        let token = self.current();
        let span = token.span();
        let name = match token.kind() {
            TokenKind::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.bump();
                TypeName::Identifier(identifier)
            }
            TokenKind::Keyword(canonical) if TYPE_KEYWORDS.contains(&canonical.as_str()) => {
                let spelling = self.spelling(span).to_owned();
                let canonical = canonical.clone();
                self.bump();
                TypeName::Keyword {
                    spelling,
                    canonical,
                    span,
                }
            }
            _ => {
                self.error(
                    "parse.expected-type",
                    "expected a type",
                    "use a type name such as `Integer`, `String`, or `List<Integer>`",
                );
                return None;
            }
        };

        let mut end = span;
        let mut arguments = Vec::new();
        if self.take_operator("<").is_some() {
            if self.at_type_close() {
                self.error(
                    "parse.expected-type",
                    "expected a generic type argument",
                    "generic argument lists cannot be empty",
                );
            } else {
                while let Some(argument) = self.parse_type() {
                    end = argument.span();
                    arguments.push(argument);
                    if self.take_punctuation(",").is_none() {
                        break;
                    }
                }
            }
            if let Some(close) = self.take_type_close() {
                end = close;
            } else {
                self.expected_token("`>` to close the generic argument list");
            }
        }

        let nullable = if self.pending_type_closers.is_empty()
            && let Some(question) = self.take_operator("?")
        {
            end = question;
            true
        } else {
            false
        };
        Some(Type::new(name, arguments, nullable, join_spans(span, end)))
    }

    fn parse_block(&mut self) -> Option<Block> {
        let Some(open) = self.take_punctuation("{") else {
            self.expected_token("`{` to begin a block");
            return None;
        };
        let mut statements = Vec::new();
        while !self.at_punctuation("}") && !self.at_eof() {
            let checkpoint = self.cursor;
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.synchronize_statement();
            }
            self.ensure_progress(checkpoint);
        }
        let close = if let Some(close) = self.take_punctuation("}") {
            close
        } else {
            self.expected_token("`}` to close the block");
            self.current_span()
        };
        Some(Block::new(statements, join_spans(open, close)))
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        if self.at_punctuation("{") {
            let block = self.parse_block()?;
            let span = block.span();
            return Some(Statement::new(StatementKind::Block(block), span));
        }

        if let Some(start) = self.take_keyword("if") {
            return self.parse_if(start);
        }
        if let Some(start) = self.take_keyword("while") {
            return self.parse_while(start);
        }
        if let Some(start) = self.take_keyword("do") {
            return self.parse_do_while(start);
        }
        if let Some(start) = self.take_keyword("for") {
            return self.parse_for(start);
        }
        if let Some(start) = self.take_keyword("return") {
            let expression = if self.at_punctuation(";") {
                None
            } else {
                self.parse_expression()
            };
            let end = self.expect_statement_semicolon();
            return Some(Statement::new(
                StatementKind::Return(expression),
                join_spans(start, end),
            ));
        }
        if let Some(start) = self.take_keyword("throw") {
            let expression = self.parse_expression()?;
            let end = self.expect_statement_semicolon();
            return Some(Statement::new(
                StatementKind::Throw(expression),
                join_spans(start, end),
            ));
        }
        if let Some(start) = self.take_keyword("break") {
            let end = self.expect_statement_semicolon();
            return Some(Statement::new(StatementKind::Break, join_spans(start, end)));
        }
        if let Some(start) = self.take_keyword("continue") {
            let end = self.expect_statement_semicolon();
            return Some(Statement::new(
                StatementKind::Continue,
                join_spans(start, end),
            ));
        }
        if let Some(semicolon) = self.take_punctuation(";") {
            return Some(Statement::new(StatementKind::Empty, semicolon));
        }

        let start = self.current_span();
        if self.looks_like_variable_declaration() {
            let variable = self.parse_variable_declaration()?;
            let end = self.expect_statement_semicolon();
            return Some(Statement::new(
                StatementKind::Variable(variable),
                join_spans(start, end),
            ));
        }

        let expression = self.parse_expression()?;
        let end = self.expect_statement_semicolon();
        Some(Statement::new(
            StatementKind::Expression(expression),
            join_spans(start, end),
        ))
    }

    fn parse_if(&mut self, start: Span) -> Option<Statement> {
        self.expect_punctuation("(", "`(` after `if`");
        let condition = self.parse_expression()?;
        self.expect_punctuation(")", "`)` after the `if` condition");
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.take_keyword("else").is_some() {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map_or_else(|| then_branch.span(), |branch| branch.span());
        Some(Statement::new(
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            },
            join_spans(start, end),
        ))
    }

    fn parse_while(&mut self, start: Span) -> Option<Statement> {
        self.expect_punctuation("(", "`(` after `while`");
        let condition = self.parse_expression()?;
        self.expect_punctuation(")", "`)` after the `while` condition");
        let body = Box::new(self.parse_statement()?);
        let end = body.span();
        Some(Statement::new(
            StatementKind::While { condition, body },
            join_spans(start, end),
        ))
    }

    fn parse_do_while(&mut self, start: Span) -> Option<Statement> {
        let body = Box::new(self.parse_statement()?);
        if self.take_keyword("while").is_none() {
            self.expected_token("`while` after the `do` body");
            return None;
        }
        self.expect_punctuation("(", "`(` after `while`");
        let condition = self.parse_expression()?;
        self.expect_punctuation(")", "`)` after the `do while` condition");
        let end = self.expect_statement_semicolon();
        Some(Statement::new(
            StatementKind::DoWhile { body, condition },
            join_spans(start, end),
        ))
    }

    fn parse_for(&mut self, start: Span) -> Option<Statement> {
        self.expect_punctuation("(", "`(` after `for`");

        if self.looks_like_variable_declaration() {
            let variable = self.parse_variable_declaration()?;
            if self.take_punctuation(":").is_some() {
                let iterable = self.parse_expression()?;
                self.expect_punctuation(")", "`)` after the enhanced `for` iterable");
                let body = Box::new(self.parse_statement()?);
                let end = body.span();
                return Some(Statement::new(
                    StatementKind::EnhancedFor {
                        variable,
                        iterable,
                        body,
                    },
                    join_spans(start, end),
                ));
            }

            self.expect_punctuation(";", "`;` after the `for` initializer");
            let condition = if self.at_punctuation(";") {
                None
            } else {
                self.parse_expression()
            };
            self.expect_punctuation(";", "`;` after the `for` condition");
            let update = if self.at_punctuation(")") {
                Vec::new()
            } else {
                self.parse_expression_list()
            };
            self.expect_punctuation(")", "`)` after the `for` clauses");
            let body = Box::new(self.parse_statement()?);
            let end = body.span();
            return Some(Statement::new(
                StatementKind::For {
                    initializer: Some(ForInitializer::Variable(Box::new(variable))),
                    condition,
                    update,
                    body,
                },
                join_spans(start, end),
            ));
        }

        let initializer = if self.at_punctuation(";") {
            None
        } else {
            Some(ForInitializer::Expressions(self.parse_expression_list()))
        };
        self.expect_punctuation(";", "`;` after the `for` initializer");
        let condition = if self.at_punctuation(";") {
            None
        } else {
            self.parse_expression()
        };
        self.expect_punctuation(";", "`;` after the `for` condition");
        let update = if self.at_punctuation(")") {
            Vec::new()
        } else {
            self.parse_expression_list()
        };
        self.expect_punctuation(")", "`)` after the `for` clauses");
        let body = Box::new(self.parse_statement()?);
        let end = body.span();
        Some(Statement::new(
            StatementKind::For {
                initializer,
                condition,
                update,
                body,
            },
            join_spans(start, end),
        ))
    }

    fn parse_variable_declaration(&mut self) -> Option<VariableDeclaration> {
        let ty = self.parse_type()?;
        let name = self.parse_identifier("variable name")?;
        let initializer = if self.take_operator("=").is_some() {
            self.parse_expression()
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.span(), Expression::span);
        let span = join_spans(ty.span(), end);
        Some(VariableDeclaration::new(ty, name, initializer, span))
    }

    fn parse_expression_list(&mut self) -> Vec<Expression> {
        let mut expressions = Vec::new();
        if let Some(expression) = self.parse_expression() {
            expressions.push(expression);
        }
        while self.take_punctuation(",").is_some() {
            if let Some(expression) = self.parse_expression() {
                expressions.push(expression);
            } else {
                break;
            }
        }
        expressions
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expression> {
        let target = self.parse_conditional()?;
        if let Some(operator) = self.take_any_operator(&[
            "=", "+=", "-=", "*=", "/=", "&=", "|=", "^=", "<<=", ">>=", ">>>=",
        ]) {
            let value = self.parse_assignment()?;
            let span = join_spans(target.span(), value.span());
            Some(Expression::new(
                ExpressionKind::Assignment {
                    target: Box::new(target),
                    operator,
                    value: Box::new(value),
                },
                span,
            ))
        } else {
            Some(target)
        }
    }

    fn parse_conditional(&mut self) -> Option<Expression> {
        let condition = self.parse_null_coalescing()?;
        if self.take_operator("?").is_none() {
            return Some(condition);
        }
        let then_expression = self.parse_expression()?;
        self.expect_punctuation(":", "`:` in the conditional expression");
        let else_expression = self.parse_conditional()?;
        let span = join_spans(condition.span(), else_expression.span());
        Some(Expression::new(
            ExpressionKind::Conditional {
                condition: Box::new(condition),
                then_expression: Box::new(then_expression),
                else_expression: Box::new(else_expression),
            },
            span,
        ))
    }

    fn parse_null_coalescing(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_logical_or, &["??"], &[])
    }

    fn parse_logical_or(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_logical_and, &["||"], &[])
    }

    fn parse_logical_and(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_bitwise_or, &["&&"], &[])
    }

    fn parse_bitwise_or(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_bitwise_xor, &["|"], &[])
    }

    fn parse_bitwise_xor(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_bitwise_and, &["^"], &[])
    }

    fn parse_bitwise_and(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_equality, &["&"], &[])
    }

    fn parse_equality(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_relational, &["==", "!=", "===", "!=="], &[])
    }

    fn parse_relational(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_shift, &["<", "<=", ">", ">="], &["instanceof"])
    }

    fn parse_shift(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_additive, &["<<", ">>", ">>>"], &[])
    }

    fn parse_additive(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_multiplicative, &["+", "-"], &[])
    }

    fn parse_multiplicative(&mut self) -> Option<Expression> {
        self.parse_binary_left(Self::parse_unary, &["*", "/", "%"], &[])
    }

    fn parse_binary_left(
        &mut self,
        operand_parser: fn(&mut Self) -> Option<Expression>,
        operators: &[&str],
        keyword_operators: &[&str],
    ) -> Option<Expression> {
        let mut left = operand_parser(self)?;
        loop {
            let operator = self
                .take_any_operator(operators)
                .or_else(|| self.take_any_keyword_operator(keyword_operators));
            let Some(operator) = operator else {
                break;
            };
            let right = operand_parser(self)?;
            let span = join_spans(left.span(), right.span());
            left = Expression::new(
                ExpressionKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expression> {
        if let Some(operator) = self.take_any_operator(&["!", "~", "+", "-", "++", "--"]) {
            let start = operator.span();
            let operand = self.parse_unary()?;
            let span = join_spans(start, operand.span());
            return Some(Expression::new(
                ExpressionKind::Unary {
                    operator,
                    operand: Box::new(operand),
                },
                span,
            ));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Option<Expression> {
        let mut expression = self.parse_primary()?;
        loop {
            if let Some(open) = self.take_punctuation("(") {
                let mut arguments = Vec::new();
                if !self.at_punctuation(")") {
                    arguments = self.parse_expression_list();
                }
                let end = if let Some(close) = self.take_punctuation(")") {
                    close
                } else {
                    self.expected_token("`)` after call arguments");
                    self.current_span()
                };
                let span = join_spans(expression.span(), end);
                let _ = open;
                expression = Expression::new(
                    ExpressionKind::Call {
                        callee: Box::new(expression),
                        arguments,
                    },
                    span,
                );
                continue;
            }
            let safe = if self.take_punctuation(".").is_some() {
                Some(false)
            } else if self.take_operator("?.").is_some() {
                Some(true)
            } else {
                None
            };
            if let Some(safe) = safe {
                let member = self.parse_identifier("member name")?;
                let span = join_spans(expression.span(), member.span());
                expression = Expression::new(
                    ExpressionKind::Member {
                        object: Box::new(expression),
                        member,
                        safe,
                    },
                    span,
                );
                continue;
            }
            if self.take_punctuation("[").is_some() {
                let index = self.parse_expression()?;
                let end = if let Some(close) = self.take_punctuation("]") {
                    close
                } else {
                    self.expected_token("`]` after the index expression");
                    self.current_span()
                };
                let span = join_spans(expression.span(), end);
                expression = Expression::new(
                    ExpressionKind::Index {
                        object: Box::new(expression),
                        index: Box::new(index),
                    },
                    span,
                );
                continue;
            }
            if let Some(operator) = self.take_any_operator(&["++", "--"]) {
                let span = join_spans(expression.span(), operator.span());
                expression = Expression::new(
                    ExpressionKind::Postfix {
                        operand: Box::new(expression),
                        operator,
                    },
                    span,
                );
                continue;
            }
            break;
        }
        Some(expression)
    }

    fn parse_primary(&mut self) -> Option<Expression> {
        let token = self.current();
        let span = token.span();
        match token.kind() {
            TokenKind::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.bump();
                Some(Expression::new(ExpressionKind::Name(identifier), span))
            }
            TokenKind::Contextual(_) => {
                let name = self.identifier_from_current();
                self.bump();
                Some(Expression::new(ExpressionKind::Name(name), span))
            }
            TokenKind::Keyword(word)
                if word == "system" || TYPE_KEYWORDS.contains(&word.as_str()) =>
            {
                let name = self.identifier_from_current();
                self.bump();
                Some(Expression::new(ExpressionKind::Name(name), span))
            }
            TokenKind::Integer => {
                let value = self.spelling(span).to_owned();
                self.bump();
                Some(Expression::new(ExpressionKind::Integer(value), span))
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.bump();
                Some(Expression::new(ExpressionKind::String(value), span))
            }
            TokenKind::Keyword(word) if word == "true" || word == "false" => {
                let value = word == "true";
                self.bump();
                Some(Expression::new(ExpressionKind::Boolean(value), span))
            }
            TokenKind::Keyword(word) if word == "null" => {
                self.bump();
                Some(Expression::new(ExpressionKind::Null, span))
            }
            TokenKind::Keyword(word) if word == "this" => {
                self.bump();
                Some(Expression::new(ExpressionKind::This, span))
            }
            TokenKind::Keyword(word) if word == "super" => {
                self.bump();
                Some(Expression::new(ExpressionKind::Super, span))
            }
            TokenKind::Keyword(word) if word == "new" => {
                self.bump();
                let ty = self.parse_type()?;
                self.expect_punctuation("(", "`(` after the constructed type");
                let arguments = if self.at_punctuation(")") {
                    Vec::new()
                } else {
                    self.parse_expression_list()
                };
                let end = if let Some(close) = self.take_punctuation(")") {
                    close
                } else {
                    self.expected_token("`)` after constructor arguments");
                    self.current_span()
                };
                Some(Expression::new(
                    ExpressionKind::New { ty, arguments },
                    join_spans(span, end),
                ))
            }
            TokenKind::Punctuation("(") => {
                self.bump();
                let expression = self.parse_expression()?;
                let end = if let Some(close) = self.take_punctuation(")") {
                    close
                } else {
                    self.expected_token("`)` after the parenthesized expression");
                    self.current_span()
                };
                Some(Expression::new(
                    ExpressionKind::Parenthesized(Box::new(expression)),
                    join_spans(span, end),
                ))
            }
            _ => {
                self.error(
                    "parse.expected-expression",
                    "expected an expression",
                    "use a literal, name, call, member access, operator expression, or `new` expression",
                );
                None
            }
        }
    }

    fn parse_modifiers(&mut self, context: ModifierContext) -> Vec<Modifier> {
        let simple = match context {
            ModifierContext::Class => SIMPLE_CLASS_MODIFIERS,
            ModifierContext::Member => SIMPLE_MEMBER_MODIFIERS,
        };
        let mut modifiers = Vec::new();
        loop {
            if let Some(modifier) = self.take_simple_modifier(simple) {
                modifiers.push(modifier);
                continue;
            }
            if context == ModifierContext::Class {
                if let Some(modifier) = self.take_sharing_modifier() {
                    modifiers.push(modifier);
                    continue;
                }
            }
            break;
        }
        modifiers
    }

    fn take_simple_modifier(&mut self, allowed: &[&str]) -> Option<Modifier> {
        let TokenKind::Keyword(canonical) = self.current().kind() else {
            return None;
        };
        if !allowed.contains(&canonical.as_str()) {
            return None;
        }
        let span = self.current_span();
        let spelling = self.spelling(span).to_owned();
        let canonical = canonical.clone();
        self.bump();
        Some(Modifier::new(spelling, canonical, span))
    }

    fn take_sharing_modifier(&mut self) -> Option<Modifier> {
        let first = match self.current().kind() {
            TokenKind::Contextual(word)
                if matches!(word.as_str(), "with" | "without" | "inherited") =>
            {
                word.clone()
            }
            _ => return None,
        };
        let start = self.current_span();
        if !self.next_is_contextual("sharing") {
            return None;
        }
        self.bump();
        let end = self.current_span();
        self.bump();
        Some(Modifier::new(
            format!("{} {}", self.spelling(start), self.spelling(end)),
            format!("{first} sharing"),
            join_spans(start, end),
        ))
    }

    fn parse_identifier(&mut self, role: &str) -> Option<Identifier> {
        if self.at_name() {
            let identifier = self.identifier_from_current();
            self.bump();
            Some(identifier)
        } else {
            self.error(
                "parse.expected-identifier",
                format!("expected {role}"),
                "use an ASCII identifier",
            );
            None
        }
    }

    fn identifier_from_current(&self) -> Identifier {
        let span = self.current_span();
        match self.current().kind() {
            TokenKind::Identifier(identifier) => identifier.clone(),
            TokenKind::Contextual(_) | TokenKind::Keyword(_) => {
                Identifier::new(self.spelling(span), span)
            }
            _ => unreachable!("identifier conversion requires a name token"),
        }
    }

    fn looks_like_variable_declaration(&self) -> bool {
        let Some(after_type) = self.scan_type(self.cursor) else {
            return false;
        };
        self.token_is_name(after_type)
            && matches!(
                self.tokens.get(after_type + 1).map(Token::kind),
                Some(TokenKind::Operator("="))
                    | Some(TokenKind::Punctuation(";"))
                    | Some(TokenKind::Punctuation(":"))
            )
    }

    fn scan_type(&self, start: usize) -> Option<usize> {
        if !self.token_is_type_start(start) {
            return None;
        }
        let mut index = start + 1;
        if matches!(
            self.tokens.get(index).map(Token::kind),
            Some(TokenKind::Operator("<"))
        ) {
            let mut depth = 1usize;
            index += 1;
            let mut expect_type = true;
            while index < self.tokens.len() {
                match self.tokens[index].kind() {
                    kind if expect_type && self.kind_is_type_start(kind) => {
                        expect_type = false;
                        index += 1;
                    }
                    TokenKind::Operator("<") if !expect_type => {
                        depth += 1;
                        expect_type = true;
                        index += 1;
                    }
                    TokenKind::Punctuation(",") if !expect_type => {
                        expect_type = true;
                        index += 1;
                    }
                    TokenKind::Operator(operator)
                        if !expect_type && operator.chars().all(|character| character == '>') =>
                    {
                        let closers = operator.len();
                        if closers > depth {
                            return None;
                        }
                        depth -= closers;
                        index += 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => return None,
                }
            }
            if depth != 0 {
                return None;
            }
        }
        if matches!(
            self.tokens.get(index).map(Token::kind),
            Some(TokenKind::Operator("?"))
        ) {
            index += 1;
        }
        Some(index)
    }

    fn kind_is_type_start(&self, kind: &TokenKind) -> bool {
        matches!(kind, TokenKind::Identifier(_))
            || matches!(kind, TokenKind::Keyword(word) if TYPE_KEYWORDS.contains(&word.as_str()))
    }

    fn token_is_type_start(&self, index: usize) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|token| self.kind_is_type_start(token.kind()))
    }

    fn token_is_name(&self, index: usize) -> bool {
        matches!(
            self.tokens.get(index).map(Token::kind),
            Some(TokenKind::Identifier(_)) | Some(TokenKind::Contextual(_))
        )
    }

    fn at_name(&self) -> bool {
        self.token_is_name(self.cursor)
    }

    fn at_class_start(&self) -> bool {
        self.at_keyword("class")
            || matches!(
                self.current().kind(),
                TokenKind::Keyword(word) if SIMPLE_CLASS_MODIFIERS.contains(&word.as_str())
            )
            || matches!(
                self.current().kind(),
                TokenKind::Contextual(word)
                    if matches!(word.as_str(), "with" | "without" | "inherited")
            )
    }

    fn at_statement_start(&self) -> bool {
        self.at_punctuation("{")
            || self.at_punctuation(";")
            || matches!(
                self.current().kind(),
                TokenKind::Keyword(word)
                    if matches!(
                        word.as_str(),
                        "if"
                            | "while"
                            | "do"
                            | "for"
                            | "return"
                            | "throw"
                            | "break"
                            | "continue"
                    )
            )
            || self.looks_like_variable_declaration()
    }

    fn synchronize_declaration(&mut self) {
        while !self.at_eof() {
            if self.at_class_start() {
                return;
            }
            self.bump();
        }
    }

    fn synchronize_member(&mut self) {
        let mut brace_depth = 0usize;
        while !self.at_eof() {
            if brace_depth == 0 && self.at_punctuation("}") {
                return;
            }
            if self.at_punctuation("{") {
                brace_depth += 1;
                self.bump();
                continue;
            }
            if self.at_punctuation("}") {
                brace_depth = brace_depth.saturating_sub(1);
                self.bump();
                continue;
            }
            if brace_depth == 0 && self.at_punctuation(";") {
                self.bump();
                return;
            }
            self.bump();
        }
    }

    fn synchronize_statement(&mut self) {
        let mut delimiter_depth = 0usize;
        while !self.at_eof() {
            if delimiter_depth == 0 && (self.at_punctuation("}") || self.at_statement_start()) {
                return;
            }
            if self.at_punctuation("(") || self.at_punctuation("[") {
                delimiter_depth += 1;
            } else if self.at_punctuation(")") || self.at_punctuation("]") {
                delimiter_depth = delimiter_depth.saturating_sub(1);
            } else if delimiter_depth == 0 && self.at_punctuation(";") {
                self.bump();
                return;
            }
            self.bump();
        }
    }

    fn expect_member_semicolon(&mut self) -> Span {
        if let Some(semicolon) = self.take_punctuation(";") {
            semicolon
        } else {
            self.expected_token("`;` after the field declaration");
            self.current_span()
        }
    }

    fn expect_statement_semicolon(&mut self) -> Span {
        if let Some(semicolon) = self.take_punctuation(";") {
            semicolon
        } else {
            self.expected_token("`;` after the statement");
            self.current_span()
        }
    }

    fn expect_punctuation(&mut self, punctuation: &str, expectation: &str) -> Option<Span> {
        if let Some(span) = self.take_punctuation(punctuation) {
            Some(span)
        } else {
            self.expected_token(expectation);
            None
        }
    }

    fn expected_token(&mut self, expectation: &str) {
        self.error(
            "parse.expected-token",
            format!("expected {expectation}"),
            "insert the missing token before this location",
        );
    }

    fn error(&mut self, code: &str, message: impl Into<String>, label: impl Into<String>) {
        self.diagnostics.push(
            Diagnostic::coded_error(Phase::Parse, code, message, Some(self.current_span()))
                .with_primary_label(label),
        );
    }

    fn ensure_progress(&mut self, checkpoint: usize) {
        if self.cursor == checkpoint && !self.at_eof() {
            self.bump();
        }
    }

    fn take_type_close(&mut self) -> Option<Span> {
        if let Some(span) = self.pending_type_closers.pop() {
            return Some(span);
        }
        let TokenKind::Operator(operator) = self.current().kind() else {
            return None;
        };
        if !matches!(*operator, ">" | ">>" | ">>>") {
            return None;
        }
        let token_span = self.current_span();
        let count = operator.len();
        self.bump();
        for offset in (1..count).rev() {
            self.pending_type_closers.push(
                Span::new(
                    token_span.source(),
                    token_span.start() + offset,
                    token_span.start() + offset + 1,
                )
                .expect("split generic closer is ordered"),
            );
        }
        Span::new(
            token_span.source(),
            token_span.start(),
            token_span.start() + 1,
        )
    }

    fn at_type_close(&self) -> bool {
        !self.pending_type_closers.is_empty()
            || matches!(
                self.current().kind(),
                TokenKind::Operator(operator) if matches!(*operator, ">" | ">>" | ">>>")
            )
    }

    fn take_any_operator(&mut self, operators: &[&str]) -> Option<Operator> {
        let TokenKind::Operator(operator) = self.current().kind() else {
            return None;
        };
        if !operators.contains(operator) {
            return None;
        }
        let span = self.current_span();
        let spelling = (*operator).to_owned();
        self.bump();
        Some(Operator::new(spelling, span))
    }

    fn take_any_keyword_operator(&mut self, operators: &[&str]) -> Option<Operator> {
        let TokenKind::Keyword(operator) = self.current().kind() else {
            return None;
        };
        if !operators.contains(&operator.as_str()) {
            return None;
        }
        let span = self.current_span();
        let spelling = operator.clone();
        self.bump();
        Some(Operator::new(spelling, span))
    }

    fn take_operator(&mut self, operator: &str) -> Option<Span> {
        if self.at_operator(operator) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn take_punctuation(&mut self, punctuation: &str) -> Option<Span> {
        if self.at_punctuation(punctuation) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn take_keyword(&mut self, keyword: &str) -> Option<Span> {
        if self.at_keyword(keyword) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn take_contextual(&mut self, word: &str) -> Option<Span> {
        if self.at_contextual(word) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn at_operator(&self, operator: &str) -> bool {
        matches!(self.current().kind(), TokenKind::Operator(found) if *found == operator)
    }

    fn at_punctuation(&self, punctuation: &str) -> bool {
        matches!(self.current().kind(), TokenKind::Punctuation(found) if *found == punctuation)
    }

    fn at_keyword(&self, keyword: &str) -> bool {
        matches!(self.current().kind(), TokenKind::Keyword(found) if found == keyword)
    }

    fn at_contextual(&self, word: &str) -> bool {
        matches!(self.current().kind(), TokenKind::Contextual(found) if found == word)
    }

    fn next_is_punctuation(&self, punctuation: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor + 1).map(Token::kind),
            Some(TokenKind::Punctuation(found)) if *found == punctuation
        )
    }

    fn next_is_contextual(&self, word: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor + 1).map(Token::kind),
            Some(TokenKind::Contextual(found)) if found == word
        )
    }

    fn at_eof(&self) -> bool {
        matches!(self.current().kind(), TokenKind::Eof)
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.cursor)
            .unwrap_or_else(|| self.tokens.last().expect("parser requires a token"))
    }

    fn current_span(&self) -> Span {
        self.current().span()
    }

    fn bump(&mut self) -> Span {
        debug_assert!(
            self.pending_type_closers.is_empty(),
            "ordinary token consumption cannot skip pending generic closers"
        );
        let span = self.current_span();
        if !self.at_eof() {
            self.cursor += 1;
        }
        span
    }

    fn spelling(&self, span: Span) -> &str {
        self.file
            .slice(span)
            .expect("parser token span belongs to its source")
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ModifierContext {
    Class,
    Member,
}

fn join_spans(start: Span, end: Span) -> Span {
    debug_assert_eq!(start.source(), end.source());
    Span::new(start.source(), start.start(), end.end()).expect("syntax spans are ordered")
}

#[cfg(test)]
mod tests {
    use super::{ParseResult, parse};
    use crate::ast::{
        ClassMember, ExpressionKind, ForInitializer, StatementKind, Visitor,
        walk_class_declaration, walk_constructor_declaration, walk_expression,
        walk_field_declaration, walk_method_declaration, walk_property_accessor,
        walk_property_declaration, walk_statement, walk_type, walk_variable_declaration,
    };
    use crate::lexer::lex;
    use crate::source::SourceMap;

    fn parse_text(text: &str) -> ParseResult {
        let mut sources = SourceMap::new();
        let source = sources.add("test.zen", text);
        let file = sources.get(source).unwrap();
        let lexical = lex(file);
        assert!(
            lexical.diagnostics.is_empty(),
            "test source should lex: {:?}",
            lexical.diagnostics
        );
        parse(file, &lexical.tokens)
    }

    #[test]
    fn parses_declarations_with_source_faithful_names_and_modifiers() {
        let result = parse_text(
            "PUBLIC WITH SHARING class Example extends Base implements One, Two {
                private Integer count = 1;
                public String label { get; private set; }
                public Example(Integer initial) { count = initial; }
                public static Integer value(String text) { return count; }
            }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let unit = result.unit.unwrap();
        let class = &unit.declarations()[0];
        assert_eq!(class.name().spelling(), "Example");
        assert_eq!(class.name().canonical(), "example");
        assert_eq!(
            class
                .modifiers()
                .iter()
                .map(|modifier| modifier.spelling())
                .collect::<Vec<_>>(),
            ["PUBLIC", "WITH SHARING"]
        );
        assert_eq!(class.extends().unwrap().display_name(), "Base");
        assert_eq!(
            class
                .implements()
                .iter()
                .map(crate::ast::Type::display_name)
                .collect::<Vec<_>>(),
            ["One", "Two"]
        );
        assert!(matches!(class.members()[0], ClassMember::Field(_)));
        assert!(matches!(class.members()[1], ClassMember::Property(_)));
        assert!(matches!(class.members()[2], ClassMember::Constructor(_)));
        assert!(matches!(class.members()[3], ClassMember::Method(_)));
    }

    #[test]
    fn parses_nested_generics_and_nullable_suffix_with_shift_tokens() {
        let result = parse_text(
            "class Types {
                Map<String, List<Map<Integer, String>>>? nested;
            }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let unit = result.unit.unwrap();
        let ClassMember::Field(field) = &unit.declarations()[0].members()[0] else {
            panic!("expected field");
        };
        assert_eq!(
            field.ty().display_name(),
            "Map<String, List<Map<Integer, String>>>?"
        );
        assert!(field.ty().is_nullable());
        assert_eq!(field.ty().arguments()[1].arguments().len(), 1);
    }

    #[test]
    fn preserves_expression_precedence_and_assignment_associativity() {
        let result = parse_text(
            "class Expressions {
                void run() {
                    result = left + right * 2 < ceiling && ready ?? fallback;
                    first = second = new Box(items[0]).value++;
                    text = String.valueOf(result);
                }
            }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let unit = result.unit.unwrap();
        let ClassMember::Method(method) = &unit.declarations()[0].members()[0] else {
            panic!("expected method");
        };
        let StatementKind::Expression(first) = method.body().statements()[0].kind() else {
            panic!("expected expression statement");
        };
        let ExpressionKind::Assignment { value, .. } = first.kind() else {
            panic!("expected assignment");
        };
        let ExpressionKind::Binary { operator, left, .. } = value.kind() else {
            panic!("expected null-coalescing binary expression");
        };
        assert_eq!(operator.spelling(), "??");
        let ExpressionKind::Binary { operator, .. } = left.kind() else {
            panic!("expected logical binary expression");
        };
        assert_eq!(operator.spelling(), "&&");

        let StatementKind::Expression(second) = method.body().statements()[1].kind() else {
            panic!("expected expression statement");
        };
        let ExpressionKind::Assignment { value, .. } = second.kind() else {
            panic!("expected outer assignment");
        };
        assert!(matches!(value.kind(), ExpressionKind::Assignment { .. }));
    }

    #[test]
    fn parses_every_m2_statement_family() {
        let result = parse_text(
            "class Statements {
                void run(List<Integer> values) {
                    ;
                    Integer total = 0;
                    if (total == 0) total++; else total--;
                    while (total < 2) { total += 1; }
                    do total -= 1; while (total > 0);
                    for (Integer i = 0; i < 2; i++) total += i;
                    for (Integer value : values) { if (value < 0) continue; }
                    throw new Problem();
                    break;
                    return;
                }
            }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let unit = result.unit.unwrap();
        let ClassMember::Method(method) = &unit.declarations()[0].members()[0] else {
            panic!("expected method");
        };
        assert!(matches!(
            method.body().statements()[0].kind(),
            StatementKind::Empty
        ));
        assert!(matches!(
            method.body().statements()[1].kind(),
            StatementKind::Variable(_)
        ));
        assert!(matches!(
            method.body().statements()[2].kind(),
            StatementKind::If { .. }
        ));
        assert!(matches!(
            method.body().statements()[3].kind(),
            StatementKind::While { .. }
        ));
        assert!(matches!(
            method.body().statements()[4].kind(),
            StatementKind::DoWhile { .. }
        ));
        let StatementKind::For { initializer, .. } = method.body().statements()[5].kind() else {
            panic!("expected traditional for");
        };
        assert!(matches!(initializer, Some(ForInitializer::Variable(_))));
        assert!(matches!(
            method.body().statements()[6].kind(),
            StatementKind::EnhancedFor { .. }
        ));
        assert!(matches!(
            method.body().statements()[7].kind(),
            StatementKind::Throw(_)
        ));
        assert!(matches!(
            method.body().statements()[8].kind(),
            StatementKind::Break
        ));
        assert!(matches!(
            method.body().statements()[9].kind(),
            StatementKind::Return(None)
        ));
    }

    #[test]
    fn shared_visitor_walks_types_statements_and_expressions_in_source_order() {
        struct Counter {
            types: usize,
            statements: usize,
            expressions: usize,
            modifiers: usize,
            fields: usize,
            properties: usize,
            accessors: usize,
            methods: usize,
            constructors: usize,
            variables: usize,
            names: Vec<String>,
        }

        impl Visitor for Counter {
            fn visit_class_declaration(&mut self, node: &crate::ast::ClassDeclaration) {
                walk_class_declaration(self, node);
            }

            fn visit_modifier(&mut self, _node: &crate::ast::Modifier) {
                self.modifiers += 1;
            }

            fn visit_field_declaration(&mut self, node: &crate::ast::FieldDeclaration) {
                self.fields += 1;
                walk_field_declaration(self, node);
            }

            fn visit_property_declaration(&mut self, node: &crate::ast::PropertyDeclaration) {
                self.properties += 1;
                walk_property_declaration(self, node);
            }

            fn visit_property_accessor(&mut self, node: &crate::ast::PropertyAccessor) {
                self.accessors += 1;
                walk_property_accessor(self, node);
            }

            fn visit_method_declaration(&mut self, node: &crate::ast::MethodDeclaration) {
                self.methods += 1;
                walk_method_declaration(self, node);
            }

            fn visit_constructor_declaration(&mut self, node: &crate::ast::ConstructorDeclaration) {
                self.constructors += 1;
                walk_constructor_declaration(self, node);
            }

            fn visit_type(&mut self, node: &crate::ast::Type) {
                self.types += 1;
                walk_type(self, node);
            }

            fn visit_statement(&mut self, node: &crate::ast::Statement) {
                self.statements += 1;
                walk_statement(self, node);
            }

            fn visit_variable_declaration(&mut self, node: &crate::ast::VariableDeclaration) {
                self.variables += 1;
                walk_variable_declaration(self, node);
            }

            fn visit_expression(&mut self, node: &crate::ast::Expression) {
                self.expressions += 1;
                if let ExpressionKind::Name(name) = node.kind() {
                    self.names.push(name.spelling().to_owned());
                }
                walk_expression(self, node);
            }
        }

        let result = parse_text(
            "public class Visit {
                Integer field = seed;
                String label { get; private set; }
                Visit(String label) { this.label = label; }
                Integer run(Integer input) {
                    Integer local = input;
                    return local + field;
                }
            }",
        );
        let mut counter = Counter {
            types: 0,
            statements: 0,
            expressions: 0,
            modifiers: 0,
            fields: 0,
            properties: 0,
            accessors: 0,
            methods: 0,
            constructors: 0,
            variables: 0,
            names: Vec::new(),
        };
        result.unit.unwrap().visit(&mut counter);
        assert_eq!(counter.types, 6);
        assert_eq!(counter.statements, 3);
        assert_eq!(counter.expressions, 9);
        assert_eq!(counter.modifiers, 2);
        assert_eq!(counter.fields, 1);
        assert_eq!(counter.properties, 1);
        assert_eq!(counter.accessors, 2);
        assert_eq!(counter.methods, 1);
        assert_eq!(counter.constructors, 1);
        assert_eq!(counter.variables, 1);
        assert_eq!(counter.names, ["seed", "label", "input", "local", "field"]);
    }

    #[test]
    fn recovers_at_declaration_member_and_statement_boundaries() {
        let result = parse_text(
            "garbage;
             public class Recovered {
                 Integer = 1;
                 Integer good;
                 void run() {
                     Integer broken = ;
                     @;
                     return good
                     good = 2;
                 }
                 String after;
             }",
        );
        let codes = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();
        assert!(codes.contains(&"parse.expected-declaration"));
        assert!(codes.contains(&"parse.expected-identifier"));
        assert!(codes.contains(&"parse.expected-expression"));
        assert!(codes.contains(&"parse.expected-token"));

        let unit = result.unit.unwrap();
        let class = &unit.declarations()[0];
        assert_eq!(class.name().spelling(), "Recovered");
        assert!(class
            .members()
            .iter()
            .any(|member| matches!(member, ClassMember::Field(field) if field.name().spelling() == "good")));
        assert!(class
            .members()
            .iter()
            .any(|member| matches!(member, ClassMember::Field(field) if field.name().spelling() == "after")));
    }

    #[test]
    fn emits_stable_diagnostic_families_for_invalid_and_unsupported_syntax() {
        let cases = [
            ("interface Nope {}", "parse.expected-declaration"),
            (
                "class C { @AuraEnabled Integer x; }",
                "parse.unsupported-syntax",
            ),
            ("class C { ; }", "parse.expected-member"),
            ("class C { Integer = 1; }", "parse.expected-identifier"),
            ("class C { Unknown x() ; }", "parse.unsupported-syntax"),
            (
                "class C { void run() { value = ; } }",
                "parse.expected-expression",
            ),
            ("class C { void run(, value) {} }", "parse.expected-type"),
            (
                "class C { void run() { return 1 } }",
                "parse.expected-token",
            ),
        ];
        for (source, expected) in cases {
            let result = parse_text(source);
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == expected),
                "{source}: {:?}",
                result.diagnostics
            );
            assert!(
                result
                    .diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.phase == crate::Phase::Parse)
            );
        }
    }

    #[test]
    fn syntax_layer_does_not_apply_name_type_or_assignment_rules() {
        let result = parse_text(
            "class Semantics {
                public public OtherName() {
                    1 = missing;
                    Unknown value = null;
                }
            }",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn rejects_a_token_stream_without_eof_without_panicking() {
        let mut sources = SourceMap::new();
        let source = sources.add("test.zen", "");
        let result = parse(sources.get(source).unwrap(), &[]);
        assert!(result.unit.is_none());
        assert_eq!(result.diagnostics[0].code, "parse.expected-declaration");
    }
}
