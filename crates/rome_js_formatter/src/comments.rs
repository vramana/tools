use crate::prelude::*;
use rome_formatter::ConcatBuilder;
use rome_js_syntax::{JsLanguage, JsSyntaxNode, JsSyntaxToken};
use rome_rowan::{
    SyntaxElement, SyntaxElementChildren, SyntaxToken, SyntaxTriviaPiece, SyntaxTriviaPieceComments,
};
use std::iter::Peekable;

pub fn print_leading_comments(
    node: &JsSyntaxNode,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    match node.first_child_or_token() {
        None => Ok(empty_element()),
        // Handle the comments as part of the formatting of this nested node.
        Some(SyntaxElement::Node(_)) => Ok(empty_element()),
        Some(SyntaxElement::Token(token)) => print_leading_comments_token(&token, fmt),
    }
}

// Needed to handle things like a JsArrayHole that has no tokens
// pub fn print_leading_comments_for_empty_node(
//     node: &JsSyntaxNode,
//     fmt: &Formatter<JsFormatOptions>,
// ) -> FormatResult<FormatElement> {
//     if node.kind().is_list() {
//         return Ok(empty_element());
//     }
//
//     let prev_token = match node.prev_sibling_or_token() {
//         Some(SyntaxElement::Token(token)) => token,
//         _ => return Ok(empty_element()),
//     };
//
//     let mut comments = ConcatBuilder::new();
//     for piece in prev_token
//         .trailing_trivia()
//         .pieces()
//         .filter_map(|piece| piece.as_comments())
//     {
//         let leading_comment = LeadingComment {
//             piece,
//             leading_new_lines: 0,
//             trailing_new_lines: 0,
//         };
//
//         comments.entry(formatted![fmt, [leading_comment]]?);
//     }
//
//     Ok(comments.finish())
// }

pub fn print_leading_comments_token(
    first_token: &JsSyntaxToken,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    let mut leading_trivia = first_token.leading_trivia().pieces().peekable();
    let mut leading_newlines = skip_new_lines(leading_trivia.by_ref());
    let mut comments = ConcatBuilder::new();

    // TODO can we use prev_or_token here rather than getting the token and then testing if it is a
    // node. Same in trailing
    if let Some(prev_token) = first_token.prev_token() {
        // Tests if the previous token is the end of a node
        let is_end_of_node =
            prev_token
                .parent()
                .map_or(false, |parent| match parent.last_child_or_token() {
                    Some(SyntaxElement::Token(first_token)) => first_token == prev_token,
                    _ => false,
                });

        // `[/* comment */\n a]` or
        // `class A {} /* comment */ class B {}`
        if !is_end_of_node || leading_newlines == 0 {
            let mut trailing_trivia = prev_token.trailing_trivia().pieces().peekable();
            skip_new_lines(trailing_trivia.by_ref());

            let mut last_comment_piece: Option<SyntaxTriviaPieceComments<JsLanguage>> = None;

            while let Some(piece) = trailing_trivia.next_back() {
                if let Some(comment) = piece.as_comments() {
                    last_comment_piece = Some(comment);
                    break;
                }
            }

            for piece in trailing_trivia {
                if let Some(comment) = piece.as_comments() {
                    let leading_comment = LeadingComment {
                        piece: comment,
                        leading_new_lines: 0,
                        trailing_new_lines: 0,
                    };

                    comments.entry(formatted![fmt, [leading_comment]]?);
                }
            }

            if let Some(last) = last_comment_piece {
                let leading_comment = LeadingComment {
                    piece: last,
                    leading_new_lines: 0,
                    trailing_new_lines: leading_newlines,
                };

                comments.entry(formatted![fmt, [leading_comment]]?);
                leading_newlines = 0;
            }
        }
    }

    // ```js
    // a;
    // /* test */ class B {};
    // ```
    //
    // ``javascript
    // /* test */
    // class B {}
    //
    // /* test */
    //
    //  class C {}
    // ```
    // From here, process the leading comments that are attached to its first token.
    while let Some(piece) = leading_trivia.next() {
        if let Some(comment) = piece.as_comments() {
            let trailing_new_lines = skip_new_lines(leading_trivia.by_ref());
            let leading_comment = LeadingComment {
                piece: comment,
                leading_new_lines: leading_newlines,
                trailing_new_lines,
            };

            comments.entry(formatted![fmt, [leading_comment]]?);

            leading_newlines = 0;
        }
    }

    Ok(comments.finish())
}

/// Test if this comment piece is a block comment
fn is_block_comment(piece: &SyntaxTriviaPieceComments<JsLanguage>) -> bool {
    piece.text().trim_start().starts_with("/*")
}

struct LeadingComment {
    leading_new_lines: usize,
    piece: SyntaxTriviaPieceComments<JsLanguage>,
    trailing_new_lines: usize,
}

impl Format for LeadingComment {
    type Options = JsFormatOptions;

    fn format(&self, _: &Formatter<Self::Options>) -> FormatResult<FormatElement> {
        let is_block = is_block_comment(&self.piece);

        let after_comment = if is_block {
            if self.leading_new_lines > 0 {
                match self.trailing_new_lines {
                    0 => soft_line_break_or_space(),
                    1 => hard_line_break(),
                    _ => empty_line(),
                }
            } else {
                space_token()
            }
        } else {
            hard_line_break()
        };

        Ok(crate::comment(format_elements![
            Token::from(&self.piece),
            after_comment
        ]))
    }
}

pub fn print_trailing_comments(
    node: &JsSyntaxNode,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    let last_token = match node.last_child_or_token() {
        None => return print_trailing_comments_for_empty_node(node, fmt),
        // The trailing comment of the last token is handled as part of this node
        Some(SyntaxElement::Node(_)) => return Ok(empty_element()),
        Some(SyntaxElement::Token(token)) => token,
    };

    print_trailing_comments_for_token(&last_token, fmt)
}

fn print_trailing_comments_for_empty_node(
    node: &JsSyntaxNode,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    if node.kind().is_list() {
        return Ok(empty_element());
    }

    match node.next_sibling_or_token() {
        Some(SyntaxElement::Token(next_token)) => {
            // TODO verify
            let mut leading_new_lines = 0;
            let mut comments = ConcatBuilder::default();
            for piece in next_token.leading_trivia().pieces() {
                if let Some(comment) = piece.as_comments() {
                    let trailing_comment = TrailingComment {
                        piece: comment,
                        leading_new_lines,
                    };

                    comments.entry(formatted![fmt, [trailing_comment]]?);
                    leading_new_lines = 0;
                } else if piece.is_newline() {
                    leading_new_lines += 1;
                }
            }

            Ok(comments.finish())
        }
        _ => Ok(empty_element()),
    }
}

pub fn print_trailing_comments_for_token(
    token: &JsSyntaxToken,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    let (next_token, is_next_start_of_node) = if let Some(next_token) = token.next_token() {
        let has_line_break_between_tokens =
            skip_new_lines(next_token.leading_trivia().pieces().peekable().by_ref()) > 0;

        // Tests if the next token is the first child of a node
        let is_next_start_of_node =
            next_token
                .parent()
                .map_or(false, |parent| match parent.first_child_or_token() {
                    Some(SyntaxElement::Token(first_child)) => first_child == next_token,
                    _ => false,
                });

        // `a /* test */ b` is leading trivia if `b` is the start of a new node, otherwise trailing
        if !has_line_break_between_tokens && is_next_start_of_node {
            return Ok(empty_element());
        }

        (Some(next_token), is_next_start_of_node)
    } else {
        (None, false)
    };

    // All comments from here on are trailing comments.
    let mut comments = ConcatBuilder::new();
    let mut leading_new_lines = 0;

    for piece in token.trailing_trivia().pieces() {
        if piece.is_newline() {
            // Only possible for the last token in the program.
            leading_new_lines += 1;
        } else if let Some(comment) = piece.as_comments() {
            let trailing_comment = TrailingComment {
                piece: comment,
                leading_new_lines,
            };

            comments.entry(formatted![fmt, [trailing_comment]]?);

            leading_new_lines = 0;
        }
    }

    if let Some(next_token) = next_token {
        // `[a, /* test */]`
        if !is_next_start_of_node {
            for piece in next_token.leading_trivia().pieces() {
                if let Some(comment) = piece.as_comments() {
                    let trailing_comment = TrailingComment {
                        piece: comment,
                        leading_new_lines,
                    };

                    comments.entry(formatted![fmt, [trailing_comment]]?);
                    leading_new_lines = 0;
                } else if piece.is_newline() {
                    leading_new_lines += 1;
                }
            }
        }
    }

    Ok(comments.finish())
}

pub fn print_dangling_comments(
    node: &JsSyntaxNode,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    let mut comments = ConcatBuilder::new();

    for dangling_comment in DanglingCommentIterator::new(node) {
        comments.entry(formatted![fmt, [dangling_comment]]?);
    }

    Ok(comments.finish())
}

pub fn print_dangling_comments_for_token(
    token: &JsSyntaxToken,
    fmt: &Formatter<JsFormatOptions>,
) -> FormatResult<FormatElement> {
    let prev = token.prev_token();

    dbg!(&token, &prev);
    match prev {
        Some(prev_token) => {
            let dangling_comment = DanglingComments {
                prev_token,
                next_token: token.clone(),
            };

            formatted![fmt, [dangling_comment]]
        }
        _ => Ok(empty_element()),
    }
}

pub fn has_dangling_comments(node: &JsSyntaxNode) -> bool {
    return DanglingCommentIterator::new(node).next().is_some();
}

#[derive(Clone, Debug)]
struct DanglingCommentIterator {
    last_token: Option<SyntaxToken<JsLanguage>>,
    children: SyntaxElementChildren<JsLanguage>,
}

impl DanglingCommentIterator {
    fn new(node: &JsSyntaxNode) -> Self {
        Self {
            last_token: None,
            children: node.children_with_tokens(),
        }
    }
}

impl Iterator for DanglingCommentIterator {
    type Item = DanglingComments;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_element = self.children.next()?;

            match next_element {
                SyntaxElement::Node(node) => {
                    if node.slots().len() == 0 && node.kind().is_list() {
                        // Filters out empty lists, `[ /* test */ EMPTY_LIST ]`
                        // These aren't "real" nodes, consider it as if there was no node in between.
                        // Just skip it and keep the last token.
                    } else {
                        // either `node node` or `token node` in both cases, not a dangling comment
                        self.last_token = None;
                    }
                }
                SyntaxElement::Token(token) => {
                    // `token token`
                    if let Some(last_token) = self.last_token.take() {
                        // TODO this iterates twice, lets do this in a single iterator
                        if last_token.has_trailing_comments() || token.has_leading_comments() {
                            self.last_token = Some(last_token.clone());
                            return Some(DanglingComments {
                                prev_token: last_token,
                                next_token: token,
                            });
                        }

                        self.last_token = Some(last_token);
                    } else {
                        // `node token`, not a dangling comment
                        self.last_token = Some(token);
                    }
                }
            }
        }
    }
}

/// Dangling comment between the following two tokens.
#[derive(Debug)]
struct DanglingComments {
    prev_token: JsSyntaxToken,
    next_token: JsSyntaxToken,
}

impl Format for DanglingComments {
    type Options = JsFormatOptions;

    fn format(&self, _: &Formatter<Self::Options>) -> FormatResult<FormatElement> {
        let comments = self
            .prev_token
            .trailing_trivia()
            .pieces()
            .chain(self.next_token.leading_trivia().pieces())
            .filter_map(|piece| piece.as_comments())
            .map(|comment| crate::comment(FormatElement::from(Token::from(&comment))));

        Ok(join_elements(hard_line_break(), comments))
    }
}

struct TrailingComment {
    piece: SyntaxTriviaPieceComments<JsLanguage>,
    leading_new_lines: usize,
}

impl Format for TrailingComment {
    type Options = JsFormatOptions;

    fn format(&self, _: &Formatter<Self::Options>) -> FormatResult<FormatElement> {
        let block_comment = is_block_comment(&self.piece);
        let formatted_comment = Token::from(&self.piece);

        Ok(match self.leading_new_lines {
            0 => {
                if block_comment {
                    crate::comment(format_elements![space_token(), formatted_comment])
                } else {
                    format_elements![
                        line_suffix(crate::comment(format_elements![
                            space_token(),
                            formatted_comment
                        ])),
                        expand_parent()
                    ]
                }
            }
            1 => line_suffix(crate::comment(format_elements![
                hard_line_break(),
                formatted_comment
            ])),
            _ => line_suffix(crate::comment(format_elements![
                empty_line(),
                formatted_comment
            ])),
        })
    }
}

/// Skips all whitespace and new line trivia, returns the number of skipped lines.
fn skip_new_lines<I: Iterator<Item = SyntaxTriviaPiece<JsLanguage>>>(
    pieces: &mut Peekable<I>,
) -> usize {
    let mut lines = 0;

    while let Some(piece) = pieces.peek() {
        if piece.is_newline() {
            lines += 1;
        } else if !piece.is_whitespace() {
            break;
        }

        pieces.next();
    }

    lines
}

#[cfg(test)]
mod tests {
    use crate::{
        print_dangling_comments, print_leading_comments, print_trailing_comments, JsFormatOptions,
    };
    use rome_formatter::prelude::{Formatter, PrinterOptions};
    use rome_formatter::{format_elements, token, FormatElement, FormatResult, Formatted};
    use rome_js_parser::{parse, SourceType};

    use std::io::Write;

    #[test]
    fn leading_comment_single_node() {
        assert_comments(
            "/* test */ a;",
            [ExpectedComments::leading("a", "/* test */ ")],
        );

        assert_comments(
            "/* test */\na;",
            [ExpectedComments::leading("a", "/* test */ ")],
        );

        assert_comments(
            "/* test */\n\na;",
            [ExpectedComments::leading("a", "/* test */ ")],
        );
    }

    #[test]
    fn leading_between_two_nodes() {
        assert_comments(
            "{} /* test */ b;",
            [ExpectedComments::leading("b", "/* test */ ")],
        );

        assert_comments(
            "{}\n /* test */ b;",
            [ExpectedComments::leading("b", "/* test */LB")],
        );

        assert_comments(
            "{}\n\n/* test */ b;",
            [ExpectedComments::leading("b", "/* test */LB")],
        );
    }

    #[test]
    fn leading_inline_comment() {
        assert_comments(
            "{}\n // test\nb;",
            [ExpectedComments::leading("b", "// testLB")],
        );

        assert_comments(
            r#"
class A {
  // comment
  constructor() {}
}"#,
            [ExpectedComments::leading("constructor", "// commentLB")],
        );

        assert_comments(
            r#"
interface A {
  // comment
  new()
}"#,
            [ExpectedComments::leading("new()", "// commentLB")],
        );
    }

    #[test]
    fn leading_empty_node() {
        assert_comments(
            "[a, /*empty*/ ,];",
            [ExpectedComments::leading("", "/*empty*/ ")],
        );
    }

    #[test]
    fn leading_after_token() {
        assert_comments(
            "[/*empty*/a, ,]",
            [ExpectedComments::leading("a", "/*empty*/ ")],
        );

        assert_comments(
            "[/*empty*/\na, ,]",
            [ExpectedComments::leading("a", "/*empty*/ ")],
        );
    }

    #[test]
    fn trailing_comment_between_two_nodes() {
        assert_comments(
            "{} /* test */\nb;",
            [ExpectedComments::trailing("{}", " /* test */")],
        );

        assert_comments(
            "{} /* test */\n\nb;",
            [ExpectedComments::trailing("{}", " /* test */")],
        );
    }

    #[test]
    fn trailing_comment_between_node_and_token() {
        assert_comments(
            "[a /* test */];",
            [ExpectedComments::trailing("a", " /* test */")],
        );

        assert_comments(
            "[a /* test */\n];",
            [ExpectedComments::trailing("a", " /* test */")],
        );

        assert_comments(
            "[a /* test */\n\n];",
            [ExpectedComments::trailing("a", " /* test */")],
        );

        assert_comments(
            r#"
let a = {
    type: 'bar',
    // trailing comment
};
"#,
            [ExpectedComments::trailing(
                "type: 'bar',",
                "__EOLLB// trailing comment",
            )],
        );
    }

    #[test]
    fn dangling_comment() {
        assert_comments(
            "[/* test */];",
            [ExpectedComments::dangling("[/* test */]", "/* test */")],
        );

        assert_comments(
            "[/* test */ /* test 2 */];",
            [ExpectedComments::dangling(
                "[/* test */ /* test 2 */]",
                "/* test */LB/* test 2 */",
            )],
        );
    }

    struct ExpectedComments {
        node: &'static str,
        leading: Option<&'static str>,
        dangling: Option<&'static str>,
        trailing: Option<&'static str>,
    }

    impl ExpectedComments {
        fn leading(node: &'static str, comment: &'static str) -> Self {
            Self {
                node,
                leading: Some(comment),
                dangling: None,
                trailing: None,
            }
        }

        fn dangling(node: &'static str, comment: &'static str) -> Self {
            Self {
                node,
                leading: None,
                dangling: Some(comment),
                trailing: None,
            }
        }

        fn trailing(node: &'static str, comment: &'static str) -> Self {
            Self {
                node,
                leading: None,
                dangling: None,
                trailing: Some(comment),
            }
        }
    }

    fn assert_comments<I>(source: &str, expected_comments: I)
    where
        I: IntoIterator<Item = ExpectedComments>,
    {
        const SPACE: char = '᛫';

        let root = parse(source, 0, SourceType::ts()).syntax();
        let formatter: Formatter<JsFormatOptions> = Formatter::default();

        let mut actual = Vec::new();
        let mut first = true;

        for node in root.descendants() {
            let leading = empty_to_none(print_leading_comments(&node, &formatter));
            let dangling = empty_to_none(print_dangling_comments(&node, &formatter));
            let trailing = empty_to_none(print_trailing_comments(&node, &formatter));

            if leading.is_some() || dangling.is_some() || trailing.is_some() {
                if first {
                    first = false;
                } else {
                    writeln!(actual).unwrap();
                }

                writeln!(actual, "Node: {}", node.text_trimmed()).unwrap();

                let comments = [
                    ("leading", leading),
                    ("dangling", dangling),
                    ("trailing", trailing),
                ];

                for (label, comment) in comments {
                    match comment {
                        None => {}
                        Some(Ok(element)) => {
                            let printed = Formatted::new(
                                // Insert a fake __SOF and __EOF tokens at the start/end to force the printer to print
                                // all whitespace characters at the start/end
                                format_elements![token("__SOL"), element, token("__EOL")],
                                PrinterOptions::default(),
                            )
                            .print();

                            let cleaned = printed
                                .as_code()
                                .trim_start_matches("__SOL")
                                .trim_end_matches("__EOL")
                                .replace(' ', &SPACE.to_string())
                                .replace('\n', "LB");

                            writeln!(actual, "\t{label}: {cleaned}\n").unwrap();
                        }
                        Some(Err(err)) => writeln!(actual, "    {label}: {err}\n").unwrap(),
                    }
                }
            }
        }

        let mut expected = Vec::new();
        first = true;

        for expected_comment in expected_comments {
            if first {
                first = false;
            } else {
                writeln!(expected, "").unwrap();
            }

            writeln!(expected, "Node: {}", expected_comment.node).unwrap();

            let comments = [
                ("leading", expected_comment.leading),
                ("dangling", expected_comment.dangling),
                ("trailing", expected_comment.trailing),
            ];

            for (label, comment) in comments {
                match comment {
                    None => {}
                    Some(expected_str) => {
                        let cleaned = expected_str
                            .replace(' ', &SPACE.to_string())
                            .replace('\n', "LB");
                        writeln!(expected, "\t{label}: {cleaned}").unwrap();
                    }
                }
            }
        }

        let actual = String::from_utf8(actual).unwrap();
        let expected = String::from_utf8(expected).unwrap();

        assert_eq!(actual.trim(), expected.trim());
    }

    fn empty_to_none(result: FormatResult<FormatElement>) -> Option<FormatResult<FormatElement>> {
        match result {
            Ok(FormatElement::Empty) => None,
            result => Some(result),
        }
    }
}
