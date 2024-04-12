use crate::{ParseError, SyntaxKind};

use super::{define_scope, token_stream::TokenStream, Parser};

define_scope! {
    #[doc(hidden)]
    pub PathScope,
    Path,
    (Colon2)
}
impl super::Parse for PathScope {
    type Error = ParseError;

    fn parse<S: TokenStream>(&mut self, parser: &mut Parser<S>) -> Result<(), Self::Error> {
        parser.set_newline_as_trivia(false);
        parser.parse(PathSegmentScope::default())?;
        while parser.bump_if(SyntaxKind::Colon2) {
            parser.parse(PathSegmentScope::default())?;
        }
        Ok(())
    }
}

define_scope! { PathSegmentScope, PathSegment }
impl super::Parse for PathSegmentScope {
    type Error = ParseError;

    fn parse<S: TokenStream>(&mut self, parser: &mut Parser<S>) -> Result<(), Self::Error> {
        match parser.current_kind() {
            Some(kind) if is_path_segment(kind) => {
                parser.bump();
                Ok(())
            }
            _ => Err(ParseError::expected(
                &[SyntaxKind::PathSegment],
                None,
                parser.end_of_prev_token,
            )),
        }
    }
}

pub(super) fn is_path_segment(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SelfTypeKw
            | SyntaxKind::SelfKw
            | SyntaxKind::IngotKw
            | SyntaxKind::SuperKw
            | SyntaxKind::Ident
    )
}
