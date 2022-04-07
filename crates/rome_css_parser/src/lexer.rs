use cssparser::Parser;
use rome_css_syntax::CssSyntaxKind;
use rome_css_syntax::CssSyntaxKind::*;
use rome_rowan::{TextRange, TextSize};

pub(crate) struct Lexer<'i, 't> {
    source: &'i str,

    parser: Parser<'i, 't>,

    /// Byte offset of the current token from the start of the source
    /// The range of the current token can be computed by `self.position - self.current_start`
    current_start: TextSize,

    /// The kind of the current token
    current_kind: CssSyntaxKind,
}

impl<'i, 't> Lexer<'i, 't> {
    pub fn new(source: &'i str, parser: Parser<'i, 't>) -> Self {
        Self {
            source,
            parser,
            current_start: TextSize::from(0),
            current_kind: TOMBSTONE,
        }
    }

    /// Returns the kind of the current token
    #[inline]
    pub const fn current(&self) -> CssSyntaxKind {
        self.current_kind
    }

    /// Returns the range of the current token (The token that was lexed by the last `next` call)
    #[inline]
    pub fn current_range(&self) -> TextRange {
        TextRange::new(self.current_start, TextSize::from(self.position()))
    }

    fn position(&self) -> u32 {
        self.parser.position().byte_index() as u32
    }
}
