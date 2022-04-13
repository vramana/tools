use crate::lexer::Lexer;
use cssparser::Token;
use rome_css_syntax::CssSyntaxKind;
use rome_css_syntax::CssSyntaxKind::EOF;

pub(crate) struct CssParser<'i, 't> {
    file_id: usize,
    lexer: Lexer<'i, 't>,
}

impl<'i, 't> CssParser<'i, 't> {
    pub fn parse(lexer: &mut Lexer<'i, 't>) -> Vec<CssSyntaxKind> {
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();

            tokens.push(token);

            if token == EOF {
                break;
            }
        }

        tokens
    }

    pub fn parse_raw(lexer: &mut Lexer<'i, 't>) -> Vec<Token<'i>> {
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_raw_token();

            if let Some(token) = token {
                tokens.push(token)
            } else {
                break;
            }
        }

        tokens
    }
}
