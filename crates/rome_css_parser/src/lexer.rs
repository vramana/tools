use cssparser::{BasicParseError, CowRcStr, ParseError, Parser, Token};
use rome_css_syntax::CssSyntaxKind;
use rome_css_syntax::CssSyntaxKind::*;
use rome_css_syntax::*;
use rome_rowan::{TextRange, TextSize};

enum CssLexerError {}

pub(crate) struct Lexer<'i, 't> {
    source: &'i str,

    parser: Parser<'i, 't>,

    /// Byte offset of the current token from the start of the source
    /// The range of the current token can be computed by `self.position - self.current_start`
    current_start: TextSize,

    /// The kind of the current token
    current_kind: CssSyntaxKind,

    current_token: Option<Token<'t>>,
}

impl<'i, 't> Lexer<'i, 't> {
    pub fn new(source: &'i str, parser: Parser<'i, 't>) -> Self {
        Self {
            source,
            parser,
            current_start: TextSize::from(0),
            current_kind: TOMBSTONE,
            current_token: None,
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

    pub fn next_token(&mut self) -> CssSyntaxKind {
        self.current_start = TextSize::from(self.position());

        let kind = if self.parser.is_exhausted() {
            EOF
        } else {
            self.lex_token().unwrap_or(EOF)
        };

        self.current_kind = kind;

        if !kind.is_trivia() {
            // self.after_newline = false;
        }

        kind
    }

    fn lex_token(&mut self) -> Option<CssSyntaxKind> {
        self.get_token();
        self.map_token_to_kind()
    }

    fn get_token(&mut self) {
        let token = self
            .parser
            .next_including_whitespace_and_comments()
            .unwrap();
        self.current_token = Some(token.clone());
    }

    fn map_token_to_kind(&mut self) -> Option<CssSyntaxKind> {
        if let Some(token) = &self.current_token {
            let kind = match token {
                Token::Delim(delim) => self.resolve_delimiter(delim),
                Token::Ident(value) => self.resolve_identifier(value),
                Token::AtKeyword(_) => todo!(),
                Token::Hash(value) => {
                    todo!()
                }
                Token::IDHash(_) => todo!(),
                Token::QuotedString(_) => todo!(),
                Token::UnquotedUrl(_) => todo!(),
                Token::Number {
                    has_sign,
                    value,
                    int_value,
                } => todo!(),
                Token::Percentage {
                    has_sign,
                    unit_value,
                    int_value,
                } => todo!(),
                Token::Dimension {
                    has_sign,
                    value,
                    int_value,
                    unit,
                } => todo!(),
                Token::WhiteSpace(_) => todo!(),
                Token::Comment(_) => todo!(),
                Token::Colon => todo!(),
                Token::Semicolon => todo!(),
                Token::Comma => todo!(),
                Token::IncludeMatch => todo!(),
                Token::DashMatch => todo!(),
                Token::PrefixMatch => todo!(),
                Token::SuffixMatch => todo!(),
                Token::SubstringMatch => todo!(),
                Token::CDO => todo!(),
                Token::CDC => todo!(),
                Token::Function(_) => todo!(),
                Token::ParenthesisBlock => todo!(),
                Token::SquareBracketBlock => todo!(),
                Token::CurlyBracketBlock => todo!(),
                Token::BadUrl(_) => todo!(),
                Token::BadString(_) => todo!(),
                Token::CloseParenthesis => todo!(),
                Token::CloseSquareBracket => todo!(),
                Token::CloseCurlyBracket => todo!(),
            };

            Some(kind)
        } else {
            None
        }
    }

    fn resolve_delimiter(&self, delim: &char) -> CssSyntaxKind {
        match delim {
            '.' => DOT,
            ',' => COMMA,
            ';' => SEMICOLON,
            ':' => COLON,

            _ => CSS_UNKNOWN,
        }
    }

    fn resolve_identifier(&self, ident: &CowRcStr) -> CssSyntaxKind {
        let ident: &str = ident.as_ref();
        match ident {
            "to" => TO_KW,
            "from" => FROM_KW,
            "not" => NOT_KW,
            "and" => AND_KW,
            _ => T![ident],
        }
    }
}
