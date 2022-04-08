//! Rome official CSS Parser

use crate::lexer::Lexer;
use crate::parser::CssParser;
use cssparser::{Parser as Tokenizer, ParserInput};
use tree_sitter::Tree;

pub(crate) mod lexer;
pub(crate) mod parser;

pub fn parse(input: &str) {
    let mut parser_input = ParserInput::new(input);
    let lexer = Tokenizer::new(&mut parser_input);
    let result = CssParser::parse(Lexer::new(input, lexer));
}

pub fn parse_tree_sitter(source: &str) -> Tree {
    use tree_sitter::{Language, Parser};
    extern "C" {
        fn tree_sitter_css() -> Language;
    }

    let language = unsafe { tree_sitter_css() };

    let mut parser = Parser::new();

    parser.set_language(language).unwrap();

    parser.parse(source, None).unwrap()
}

#[cfg(test)]
mod test {
    use crate::parse_tree_sitter;

    #[test]
    fn test() {
        let source = dbg!(
            r#"
                .style { margin: 0 }"#
        );

        let tree = parse_tree_sitter(source);

        println!("{:?}", &tree);
        let root_node = tree.root_node();
        let mut tree_walk = root_node.walk();
        // let children = root_node.children(&mut tree_walk);

        dbg!(tree_walk.node());
        dbg!(tree_walk.goto_first_child());
        dbg!(tree_walk.node());
        dbg!(tree_walk.goto_first_child());
        dbg!(tree_walk.node());
        dbg!(tree_walk.goto_first_child());
        dbg!(tree_walk.node());
        dbg!(tree_walk.goto_first_child());
        dbg!(tree_walk.node());
        dbg!(tree_walk.goto_first_child());
        let node = dbg!(tree_walk.node());
        dbg!(node.kind(), node.kind_id());
        assert_eq!(root_node.kind(), "stylesheet");
    }
}
