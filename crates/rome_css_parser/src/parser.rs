use crate::lexer::Lexer;

pub(crate) struct CssParser<'i, 't> {
    file_id: usize,
    lexer: Lexer<'i, 't>,
}

impl<'i, 't> CssParser<'i, 't> {
    pub fn parse(lexer: Lexer<'i, 't>) {
        
    }
}
