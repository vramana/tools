//! Rome official CSS Parser

use crate::lexer::Lexer;
use crate::parser::CssParser;
use cssparser::{Parser as Tokenizer, ParserInput};

pub(crate) mod lexer;
pub(crate) mod parser;

pub fn parse(input: &str) {
    let mut parser_input = ParserInput::new(input);
    let lexer = Tokenizer::new(&mut parser_input);
    CssParser::parse(Lexer::new(input, lexer))
}
