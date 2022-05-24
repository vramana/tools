use crate::prelude::*;
use crate::ConcatBuilder;
use rome_rowan::syntax::SyntaxTrivia;
use rome_rowan::{Language, SyntaxToken, SyntaxTokenText, SyntaxTriviaPieceComments};
use smallvec::SmallVec;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CommentKind {
    /// A comment that should be on its own line, e.g. a `/* comment\n test */` in JavaScript
    Block,

    /// An inline comment that can share a line with other code, e.g. a `/* comment */` in JavaScript
    InlineBlock,

    /// A line comment, e.g. `// test` in JS
    Line,
}

impl CommentKind {
    pub const fn is_block(&self) -> bool {
        matches!(self, CommentKind::Block)
    }

    pub const fn is_inline_block(&self) -> bool {
        matches!(self, CommentKind::InlineBlock)
    }

    pub const fn is_line(&self) -> bool {
        matches!(self, CommentKind::Line)
    }
}

pub trait CommentStyle {
    type Language: Language;

    /// Determines the kind of the passed comment
    fn comment_kind(comment: &SyntaxTriviaPieceComments<Self::Language>) -> CommentKind;

    /// Tests if the passed in token is the end token grouping a set of nodes. Common grouping tokens are:
    /// * `)`, `]`, `}`: Group arguments, array elements, or members
    /// * `;`: Group statements
    /// * `,`: Group array elements, arguments
    fn is_end_grouping_token(token: <Self::Language as Language>::Kind) -> bool;

    /// Tests if this token is the start of a group of nodes. Common grouping tokens are:
    /// * `(`, `[`, and `{`
    fn is_start_grouping_token(token: <Self::Language as Language>::Kind) -> bool;
}

pub struct FormatPrecedingComments<'a, S, O>
where
    S: CommentStyle,
{
    // TODO replace with enum?
    token: &'a SyntaxToken<S::Language>,
    prev_token: PreviousToken<S::Language>,
    style: PhantomData<S>,
    options: PhantomData<O>,
}

impl<'a, S, O> FormatPrecedingComments<'a, S, O>
where
    S: CommentStyle,
{
    pub(super) fn new(
        token: &'a SyntaxToken<S::Language>,
        prev_token: PreviousToken<S::Language>,
    ) -> Self {
        Self {
            token,
            prev_token,
            style: PhantomData,
            options: PhantomData,
        }
    }
}

#[derive(Debug)]
pub(crate) enum PreviousToken<L: Language> {
    Token(Option<SyntaxToken<L>>),
    Override { kind: L::Kind },
}

impl<L: Language> PreviousToken<L> {
    fn trailing_trivia(&self) -> Option<SyntaxTrivia<L>> {
        match self {
            PreviousToken::Token(token) => token.as_ref().map(|t| t.trailing_trivia()),
            PreviousToken::Override { .. } => None,
        }
    }

    fn kind(&self) -> Option<L::Kind>
    where
        L::Kind: Copy,
    {
        match self {
            PreviousToken::Token(token) => token.as_ref().map(|t| t.kind()),
            PreviousToken::Override { kind } => Some(*kind),
        }
    }
}

impl<'a, CommentStyle, O> FormatPrecedingComments<'a, CommentStyle, O>
where
    CommentStyle: crate::comments::CommentStyle,
{
    fn format_comments(
        &self,
        comments: &[Comment<CommentStyle::Language>],
        lines_before_token: u32,
    ) -> FormatResult<FormatElement> {
        if comments.is_empty() {
            return Ok(empty_element());
        }

        let mut result = ConcatBuilder::new();

        for (i, comment) in comments.iter().enumerate() {
            let mut formatted_comment = ConcatBuilder::new();
            let mut move_before_line_break = false;

            // Test if the comment should either be printed on the same line...
            if comment.lines_before == 0 || comment.kind.is_inline_block() {
                move_before_line_break = self.should_move_comment_before_line_break(&comment);

                if move_before_line_break {}

                // Comment is on same line as content, insert any necessary space
                if self.needs_space_before_comment(&comment) {
                    formatted_comment.entry(space_token());
                }
            } else {
                // or be placed on its own line
                formatted_comment.entry(match comment.lines_before {
                    1 => hard_line_break(),
                    _ => empty_line(),
                });
            }

            // Write the text
            formatted_comment.entry(FormatElement::from(Token::from(&comment.piece)));

            // ```
            // a // test
            //      ,
            // ```
            if comment.lines_before == 0
                && comment.kind.is_line()
                && CommentStyle::is_end_grouping_token(self.token.kind())
            {
                let inner = formatted_comment.take();
                formatted_comment.entry(format_elements![line_suffix(inner), expand_parent()]);
            } else {
                let mut lines_after = comments
                    .get(i + 1)
                    .map_or(lines_before_token, |next| next.lines_before);

                // Force a line break after block comments
                if comment.kind.is_block() {
                    lines_after = lines_after.max(1);
                }

                // Preserve new lines after comment
                match lines_after {
                    0 => {
                        if i == comments.len() - 1
                            && self.needs_space_after_last_comment(&comment, lines_before_token)
                        {
                            formatted_comment.entry(space_token());
                        }
                    }
                    1 => {
                        formatted_comment.entry(hard_line_break());
                    }
                    _ => {
                        formatted_comment.entry(empty_line());
                    }
                };
            }

            // TODO, wrapping in comment still necessary?
            let mut formatted_comment =
                crate::comment(formatted_comment.finish(), move_before_line_break);

            result.entry(formatted_comment);
        }

        dbg!(self.token);

        Ok(dbg!(result.finish()))
    }

    fn should_move_comment_before_line_break(
        &self,
        comment: &Comment<CommentStyle::Language>,
    ) -> bool {
        // No point of moving if there's a line break before
        // ```
        // a
        // // test
        // ```
        if comment.lines_before != 0 {
            return false;
        }

        // Always put block comments on the same line
        if comment.kind.is_block() {
            return false;
        }

        // It looks better to move comments after an opening grouping character into the group
        // ```javascript
        // class A  { /* test */ // or inline
        // ```
        // becomes
        // ```javascript
        // class A {
        //   /* test */ // test
        // ```
        if let Some(prev_token_kind) = self.prev_token.kind() {
            !CommentStyle::is_start_grouping_token(prev_token_kind)
        } else {
            true
        }
    }

    fn needs_space_before_comment(&self, comment: &Comment<CommentStyle::Language>) -> bool {
        if comment.kind.is_line() && comment.lines_before == 0 {
            return true;
        }

        if let Some(prev_token_kind) = self.prev_token.kind() {
            !CommentStyle::is_start_grouping_token(prev_token_kind)
        } else {
            true
        }
    }

    fn needs_space_after_last_comment(
        &self,
        comment: &Comment<CommentStyle::Language>,
        lines_before_token: u32,
    ) -> bool {
        match comment.kind {
            CommentKind::Block => false,
            CommentKind::InlineBlock | CommentKind::Line => {
                if lines_before_token == 0 {
                    // `/* test */)` -> Don't add a space
                    !CommentStyle::is_end_grouping_token(self.token.kind())
                } else {
                    false
                }
            }
        }
    }
}

impl<CommentStyle, O> Format for FormatPrecedingComments<'_, CommentStyle, O>
where
    CommentStyle: crate::comments::CommentStyle,
{
    type Options = O;

    fn format(&self, _: &Formatter<Self::Options>) -> FormatResult<FormatElement> {
        // Two cases: If there's an override, then use the prev kind to determine if it requires a leading whitespace
        // Otherwise: take the leading
        let pieces = self
            .prev_token
            .trailing_trivia()
            .map(|trivia| trivia.pieces())
            .into_iter()
            .flatten()
            .chain(self.token.leading_trivia().pieces());

        // Lines before the next comment or the token.
        let mut lines_before = 0u32;
        let mut comments: SmallVec<[Comment<CommentStyle::Language>; 3]> = SmallVec::new();

        for piece in pieces {
            if piece.is_newline() {
                lines_before += 1;
            } else if let Some(comment) = piece.as_comments() {
                let kind = CommentStyle::comment_kind(&comment);
                let comment = Comment {
                    piece: comment,
                    lines_before,
                    kind,
                };
                comments.push(comment);
                lines_before = 0;
            } else if piece.is_skipped() {
                return Err(FormatError::SyntaxError);
            }
        }

        dbg!(&comments);

        self.format_comments(comments.as_slice(), lines_before)
    }
}

#[derive(Debug)]
struct Comment<L>
where
    L: Language,
{
    piece: SyntaxTriviaPieceComments<L>,
    lines_before: u32,
    kind: CommentKind,
}
