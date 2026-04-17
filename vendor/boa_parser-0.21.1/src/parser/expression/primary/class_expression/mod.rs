use crate::{
    lexer::TokenKind,
    parser::{
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
        expression::{BindingIdentifier, Expression as ExpressionParser},
        statement::ClassTail,
    },
    source::ReadChar,
};
use boa_ast::{
    Keyword, Punctuator, Span, Spanned, function::ClassExpression as ClassExpressionNode,
};
use boa_interner::Interner;

/// Class expression parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct ClassExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassExpression {
    /// Creates a new `ClassExpression` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassExpression
where
    R: ReadChar,
{
    type Output = ClassExpressionNode;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let mut decorators = Vec::new();
        while matches!(
            cursor.peek(0, interner).or_abrupt()?.kind(),
            TokenKind::Punctuator(Punctuator::At)
        ) {
            cursor.advance(interner);
            decorators.push(
                ExpressionParser::new(true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            );
        }
        let class_token = cursor.expect(
            TokenKind::Keyword((Keyword::Class, false)),
            "class expression",
            interner,
        )?;
        let class_span_start = class_token.span().start();
        let start_linear_span = class_token.linear_span();

        let strict = cursor.strict();
        cursor.set_strict(true);

        let token = cursor.peek(0, interner).or_abrupt()?;
        let name = match token.kind() {
            TokenKind::IdentifierName(_)
            | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
                    .into()
            }
            _ => None,
        };
        cursor.set_strict(strict);

        let (super_ref, constructor, elements, end, linear_end) =
            ClassTail::new(name, self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let mut expression = ClassExpressionNode::new_with_decorators(
            name,
            super_ref,
            constructor,
            elements.into_boxed_slice(),
            decorators.into_boxed_slice(),
            name.is_some(),
            Span::new(class_span_start, end),
        );
        expression.set_linear_span(start_linear_span.union(linear_end));

        Ok(expression)
    }
}
