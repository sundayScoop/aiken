use chumsky::prelude::*;

use crate::ast;

use super::{error::ParseError, token::Token};

pub fn parser() -> impl Parser<Token, ast::Annotation, Error = ParseError> {
    recursive(|r| {
        choice((
            // Type hole
            select! {Token::DiscardName { name } => name}.map_with_span(|name, span| {
                ast::Annotation::Hole {
                    location: span,
                    name,
                }
            }),
            // Tuple
            r.clone()
                .separated_by(just(Token::Comma))
                .at_least(2)
                .allow_trailing()
                .delimited_by(
                    choice((just(Token::LeftParen), just(Token::NewLineLeftParen))),
                    just(Token::RightParen),
                )
                .map_with_span(|elems, span| ast::Annotation::Tuple {
                    location: span,
                    elems,
                }),
            // Function
            just(Token::Fn)
                .ignore_then(
                    r.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LeftParen), just(Token::RightParen)),
                )
                .then_ignore(just(Token::RArrow))
                .then(r.clone())
                .map_with_span(|(arguments, ret), span| ast::Annotation::Fn {
                    location: span,
                    arguments,
                    ret: Box::new(ret),
                }),
            // Constructor function
            select! {Token::UpName { name } => name}
                .then(
                    r.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::Less), just(Token::Greater))
                        .or_not(),
                )
                .map_with_span(|(name, arguments), span| ast::Annotation::Constructor {
                    location: span,
                    module: None,
                    name,
                    arguments: arguments.unwrap_or_default(),
                }),
            // Constructor Module or type Variable
            select! {Token::Name { name } => name}
                .then(
                    just(Token::Dot)
                        .ignore_then(select! {Token::UpName {name} => name})
                        .then(
                            r.separated_by(just(Token::Comma))
                                .allow_trailing()
                                .delimited_by(just(Token::Less), just(Token::Greater))
                                .or_not(),
                        )
                        .or_not(),
                )
                .map_with_span(|(mod_name, opt_dot), span| {
                    if let Some((name, arguments)) = opt_dot {
                        ast::Annotation::Constructor {
                            location: span,
                            module: Some(mod_name),
                            name,
                            arguments: arguments.unwrap_or_default(),
                        }
                    } else {
                        // TODO: parse_error(ParseErrorType::NotConstType, SrcSpan { start, end })
                        ast::Annotation::Var {
                            location: span,
                            name: mod_name,
                        }
                    }
                }),
        ))
    })
}

#[cfg(test)]
mod tests {
    use crate::assert_annotation;

    #[test]
    fn type_annotation_with_module_prefix() {
        assert_annotation!("aiken.Option<Int>");
    }
}
