use rome_formatter::comments::{CommentKind, CommentStyle};
use rome_js_syntax::{JsLanguage, JsSyntaxKind};
use rome_rowan::SyntaxTriviaPieceComments;

pub(crate) struct JsCommentStyle;

impl CommentStyle for JsCommentStyle {
    type Language = JsLanguage;

    fn comment_kind(comment: &SyntaxTriviaPieceComments<Self::Language>) -> CommentKind {
        if comment.text().starts_with("//") {
            CommentKind::Line
        } else if comment.has_newline() {
            CommentKind::Block
        } else {
            CommentKind::InlineBlock
        }
    }

    fn is_end_grouping_token(token: JsSyntaxKind) -> bool {
        matches!(
            token,
            JsSyntaxKind::R_CURLY
                | JsSyntaxKind::R_PAREN
                | JsSyntaxKind::R_BRACK
                | JsSyntaxKind::COMMA
                | JsSyntaxKind::SEMICOLON
        )
    }

    fn is_start_grouping_token(token: JsSyntaxKind) -> bool {
        matches!(
            token,
            JsSyntaxKind::L_CURLY | JsSyntaxKind::L_PAREN | JsSyntaxKind::L_BRACK
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{format_node, JsFormatOptions};
    use rome_js_parser::parse_module;

    #[test]
    fn test_inline_comment_same_line() {
        assert_formatted("[/* test */];", "[/* test */];");
        assert_formatted("[       /* test */             ];", "[/* test */];");

        assert_formatted("[       /* test */  a];", "[/* test */ a];");
    }

    #[test]
    fn test_inline_comment_leading_new_lines() {
        assert_formatted("[\n/* test */];", "[/* test */];");
        assert_formatted("[\n\n/* test */];", "[/* test */];");
    }

    #[test]
    fn test_inline_comment_trailing_new_lines() {
        assert_formatted("[/* test */ a];", "[/* test */ a];");
        assert_formatted("[/* test */\n\na];", "[\n\t/* test */\n\n\ta,\n];");
    }

    #[test]
    fn test_inline_comment_eof() {
        assert_formatted("/* test */", "/* test */");
    }

    #[test]
    fn test_line_comment() {
        assert_formatted("[]; // test", "[]; // test");
        assert_formatted("[] // test\n;", "[]; // test");
        assert_formatted(
            r#"[
        a // test
        ,
        b
        ]"#,
            "[\n\ta, // test\n\tb,\n];",
        );
    }

    #[test]
    fn test_line_comment_spacing() {
        assert_formatted("a; // test\nb;", "a; // test\nb;");

        assert_formatted("// test\n\n b", "// test\n\nb;");
    }

    fn assert_formatted(source: &str, expected: &str) {
        let root = parse_module(source, 0).syntax();

        let formatted =
            format_node(JsFormatOptions::default(), &root).expect("Expected formatting to succeed");

        dbg!(&formatted);

        let actual = formatted.print();

        assert_eq!(actual.as_code().trim(), expected);
    }
}
