//! Lexical declaration parsing.
//!
//! This parses `let`, `const`, `using`, and `await using` declarations.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations

use crate::{
    Error,
    lexer::{Error as LexError, Token, TokenKind},
    parser::{
        AllowAwait, AllowIn, AllowYield, OrAbrupt, ParseResult, TokenParser,
        cursor::{Cursor, SemicolonResult},
        expression::Initializer,
        statement::{ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern},
    },
    source::ReadChar,
};
use ast::operations::bound_names;
use boa_ast::{self as ast, Keyword, Punctuator, Spanned, declaration::Variable};
use boa_interner::{Interner, Sym};
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UsingDeclarationKind {
    Using,
    AwaitUsing,
}

/// Parses a lexical declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LexicalDeclaration
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct LexicalDeclaration {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    loop_init: bool,
}

impl LexicalDeclaration {
    /// Creates a new `LexicalDeclaration` parser.
    pub(in crate::parser) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        loop_init: bool,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            loop_init,
        }
    }
}

fn is_plain_identifier_name(kind: &TokenKind, interner: &Interner, name: &str) -> bool {
    matches!(kind, TokenKind::IdentifierName((sym, contains_escape))
        if !contains_escape.0 && interner.resolve_expect(*sym).utf8() == Some(name))
}

fn same_line(left: &Token, right: &Token) -> bool {
    left.span().end().line_number() == right.span().start().line_number()
}

pub(crate) fn using_declaration_kind<R>(
    cursor: &mut Cursor<R>,
    interner: &mut Interner,
    allow_await: bool,
    for_head: bool,
) -> ParseResult<Option<UsingDeclarationKind>>
where
    R: ReadChar,
{
    let Some(current) = cursor.peek(0, interner)?.cloned() else {
        return Ok(None);
    };

    if allow_await && matches!(current.kind(), TokenKind::Keyword((Keyword::Await, false))) {
        let Some(using) = cursor.peek(1, interner)?.cloned() else {
            return Ok(None);
        };
        if !is_plain_identifier_name(using.kind(), interner, "using") {
            return Ok(None);
        }
        let Some(next) = cursor.peek(2, interner)?.cloned() else {
            return Ok(None);
        };
        if !same_line(&current, &using) || !same_line(&using, &next) {
            return Ok(None);
        }
        if matches!(
            next.kind(),
            TokenKind::Punctuator(Punctuator::OpenBracket | Punctuator::OpenBlock)
        ) {
            return Ok(None);
        }

        return Ok(Some(UsingDeclarationKind::AwaitUsing));
    }

    if !is_plain_identifier_name(current.kind(), interner, "using") {
        return Ok(None);
    }

    let Some(next) = cursor.peek(1, interner)?.cloned() else {
        return Ok(None);
    };
    if !same_line(&current, &next) {
        return Ok(None);
    }

    if for_head && matches!(next.kind(), TokenKind::Keyword((Keyword::Of, false))) {
        let Some(after_of) = cursor.peek(2, interner)? else {
            return Ok(None);
        };
        return Ok(matches!(
            after_of.kind(),
            TokenKind::Punctuator(Punctuator::Assign | Punctuator::Semicolon | Punctuator::Colon)
        )
        .then_some(UsingDeclarationKind::Using));
    }

    if matches!(
        next.kind(),
        TokenKind::Punctuator(Punctuator::OpenBracket | Punctuator::OpenBlock)
    ) {
        return Ok(None);
    }

    Ok(allowed_token_after_using(Some(&next)).then_some(UsingDeclarationKind::Using))
}

impl<R> TokenParser<R> for LexicalDeclaration
where
    R: ReadChar,
{
    type Output = ast::declaration::LexicalDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let tok = cursor.next(interner).or_abrupt()?;

        let lexical_declaration = match tok.kind() {
            TokenKind::Keyword((Keyword::Const | Keyword::Let, true)) => {
                return Err(Error::general(
                    "Keyword must not contain escaped characters",
                    tok.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Const, false)) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                BindingDeclarationKind::Const,
                self.loop_init,
            )
            .parse(cursor, interner)?,
            TokenKind::Keyword((Keyword::Let, false)) => BindingList::new(
                self.allow_in,
                self.allow_yield,
                self.allow_await,
                BindingDeclarationKind::Let,
                self.loop_init,
            )
            .parse(cursor, interner)?,
            TokenKind::IdentifierName(_)
                if is_plain_identifier_name(tok.kind(), interner, "using") =>
            {
                BindingList::new(
                    self.allow_in,
                    self.allow_yield,
                    self.allow_await,
                    BindingDeclarationKind::Using,
                    self.loop_init,
                )
                .parse(cursor, interner)?
            }
            TokenKind::Keyword((Keyword::Await, false))
                if self.allow_await.0
                    && {
                        let using = cursor.peek(0, interner)?.cloned();
                        let next = cursor.peek(1, interner)?.cloned();
                        matches!(
                            (using, next),
                            (Some(using), Some(next))
                                if is_plain_identifier_name(using.kind(), interner, "using")
                                    && same_line(&tok, &using)
                                    && same_line(&using, &next)
                        )
                    } =>
            {
                cursor.advance(interner);
                BindingList::new(
                    self.allow_in,
                    self.allow_yield,
                    self.allow_await,
                    BindingDeclarationKind::AwaitUsing,
                    self.loop_init,
                )
                .parse(cursor, interner)?
            }
            _ => {
                return Err(Error::unexpected(
                    tok.to_string(interner),
                    tok.span(),
                    "lexical declaration",
                ));
            }
        };

        if !self.loop_init {
            cursor.expect_semicolon("lexical declaration", interner)?;
        }

        // It is a Syntax Error if the BoundNames of BindingList contains "let".
        // It is a Syntax Error if the BoundNames of BindingList contains any duplicate entries.
        let bound_names = bound_names(&lexical_declaration);
        let mut names = FxHashSet::default();
        for name in bound_names {
            if name == Sym::LET {
                return Err(Error::general(
                    "'let' is disallowed as a lexically bound name",
                    tok.span().start(),
                ));
            }
            if !names.insert(name) {
                return Err(Error::general(
                    "lexical name declared multiple times",
                    tok.span().start(),
                ));
            }
        }

        Ok(lexical_declaration)
    }
}

/// Check if the given token is valid after the `let` keyword of a lexical declaration.
pub(crate) fn allowed_token_after_let(token: Option<&Token>) -> bool {
    matches!(
        token.map(Token::kind),
        Some(
            TokenKind::IdentifierName(_)
                | TokenKind::Keyword((
                    Keyword::Await | Keyword::Yield | Keyword::Let | Keyword::Async,
                    _
                ))
                | TokenKind::Punctuator(Punctuator::OpenBlock | Punctuator::OpenBracket),
        )
    )
}

pub(crate) fn allowed_token_after_using(token: Option<&Token>) -> bool {
    matches!(
        token.map(Token::kind),
        Some(
            TokenKind::IdentifierName(_)
                | TokenKind::Keyword(_)
                | TokenKind::Punctuator(Punctuator::OpenBlock | Punctuator::OpenBracket),
        )
    )
}

/// Parses a binding list.
///
/// It will return an error if a `const` declaration is being parsed and there is no
/// initializer.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BindingList
#[derive(Debug, Clone, Copy)]
struct BindingList {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    declaration_kind: BindingDeclarationKind,
    loop_init: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingDeclarationKind {
    Const,
    Let,
    Using,
    AwaitUsing,
}

impl BindingList {
    /// Creates a new `BindingList` parser.
    fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
        declaration_kind: BindingDeclarationKind,
        loop_init: bool,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            declaration_kind,
            loop_init,
        }
    }
}

impl<R> TokenParser<R> for BindingList
where
    R: ReadChar,
{
    type Output = ast::declaration::LexicalDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        // Create vectors to store the variable declarations
        // Const and Let signatures are slightly different, Const needs definitions, Lets don't
        let mut decls = Vec::new();

        loop {
            let decl_position = cursor.peek(0, interner).or_abrupt()?.span().start();
            let decl = LexicalBinding::new(self.allow_in, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;

            if matches!(
                self.declaration_kind,
                BindingDeclarationKind::Using | BindingDeclarationKind::AwaitUsing
            ) && matches!(decl.binding(), boa_ast::declaration::Binding::Pattern(_))
            {
                return Err(Error::general(
                    format!(
                        "{} declarations only allow binding identifiers",
                        self.declaration_kind.description()
                    ),
                    decl_position,
                ));
            }

            if self.declaration_kind.requires_initializer() {
                let init_is_some = decl.init().is_some();

                if init_is_some || self.loop_init {
                    decls.push(decl);
                } else {
                    let next = cursor.next(interner).or_abrupt()?;
                    return Err(Error::general(
                        format!(
                            "Expected initializer for {} declaration",
                            self.declaration_kind.description()
                        ),
                        next.span().start(),
                    ));
                }
            } else {
                decls.push(decl);
            }

            match cursor.peek_semicolon(interner)? {
                SemicolonResult::Found(_) => break,
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Keyword((Keyword::Of, true))
                        || tk.kind() == &TokenKind::Keyword((Keyword::In, true)) =>
                {
                    return Err(Error::general(
                        "Keyword must not contain escaped characters",
                        tk.span().start(),
                    ));
                }
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Keyword((Keyword::Of, false))
                        || tk.kind() == &TokenKind::Keyword((Keyword::In, false)) =>
                {
                    break;
                }
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Punctuator(Punctuator::Comma) =>
                {
                    // We discard the comma
                    cursor.advance(interner);
                }
                SemicolonResult::NotFound(_) if self.loop_init => break,
                SemicolonResult::NotFound(_) => {
                    let next = cursor.next(interner).or_abrupt()?;
                    return Err(Error::expected(
                        [";".to_owned(), "line terminator".to_owned()],
                        next.to_string(interner),
                        next.span(),
                        "lexical declaration binding list",
                    ));
                }
            }
        }

        let decls = decls
            .try_into()
            .expect("`LexicalBinding` must return at least one variable");

        Ok(match self.declaration_kind {
            BindingDeclarationKind::Const => ast::declaration::LexicalDeclaration::Const(decls),
            BindingDeclarationKind::Let => ast::declaration::LexicalDeclaration::Let(decls),
            BindingDeclarationKind::Using => ast::declaration::LexicalDeclaration::Using(decls),
            BindingDeclarationKind::AwaitUsing => {
                ast::declaration::LexicalDeclaration::AwaitUsing(decls)
            }
        })
    }
}

impl BindingDeclarationKind {
    const fn requires_initializer(self) -> bool {
        matches!(self, Self::Const | Self::Using | Self::AwaitUsing)
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Const => "const",
            Self::Let => "let",
            Self::Using => "using",
            Self::AwaitUsing => "await using",
        }
    }
}

/// Lexical binding parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-LexicalBinding
struct LexicalBinding {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LexicalBinding {
    /// Creates a new `BindingList` parser.
    fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for LexicalBinding
where
    R: ReadChar,
{
    type Output = Variable;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let peek_token = cursor.peek(0, interner).or_abrupt()?;
        let position = peek_token.span().start();

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings = ObjectBindingPattern::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                } else {
                    None
                };

                let declaration = bindings.into();

                if bound_names(&declaration).contains(&Sym::LET) {
                    return Err(Error::lex(LexError::Syntax(
                        "'let' is disallowed as a lexically bound name".into(),
                        position,
                    )));
                }

                Ok(Variable::from_pattern(declaration, init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings = ArrayBindingPattern::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                } else {
                    None
                };

                let declaration = bindings.into();

                if bound_names(&declaration).contains(&Sym::LET) {
                    return Err(Error::lex(LexError::Syntax(
                        "'let' is disallowed as a lexically bound name".into(),
                        position,
                    )));
                }

                Ok(Variable::from_pattern(declaration, init))
            }
            _ => {
                let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                if ident == Sym::LET {
                    return Err(Error::lex(LexError::Syntax(
                        "'let' is disallowed as a lexically bound name".into(),
                        position,
                    )));
                }

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    let mut init =
                        Initializer::new(self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                    init.set_anonymous_function_definition_name(&ident);
                    Some(init)
                } else {
                    None
                };
                Ok(Variable::from_identifier(ident, init))
            }
        }
    }
}
