use crate::green::{GreenToken, GreenTrivia};
use crate::syntax::SyntaxTrivia;
use crate::syntax_token_text::SyntaxTokenText;
use crate::{
    cursor, Direction, Language, NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode, TriviaPiece,
    TriviaPieceKind,
};
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;
use text_size::{TextRange, TextSize};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken<L: Language> {
    raw: cursor::SyntaxToken,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxToken<L> {
    /// Create a new token detached from any tree
    ///
    /// This is mainly useful for creating a small number of individual tokens
    /// when mutating an existing tree, the bulk of the tokens in a given file
    /// should be created through the [crate::TreeBuilder] abstraction instead
    /// as it will efficiently cache and reuse the created tokens
    pub fn new_detached<Leading, Trailing>(
        kind: L::Kind,
        text: &str,
        leading: Leading,
        trailing: Trailing,
    ) -> Self
    where
        Leading: IntoIterator<Item = TriviaPiece>,
        Leading::IntoIter: ExactSizeIterator,
        Trailing: IntoIterator<Item = TriviaPiece>,
        Trailing::IntoIter: ExactSizeIterator,
    {
        Self {
            raw: cursor::SyntaxToken::new_detached(GreenToken::with_trivia(
                kind.to_raw(),
                text,
                GreenTrivia::new(leading),
                GreenTrivia::new(trailing),
            )),
            _p: PhantomData,
        }
    }

    pub(super) fn green_token(&self) -> GreenToken {
        self.raw.green().to_owned()
    }

    pub fn key(&self) -> (NonNull<()>, TextSize) {
        self.raw.key()
    }

    pub fn kind(&self) -> L::Kind {
        L::Kind::from_raw(self.raw.kind())
    }

    pub fn text_range(&self) -> TextRange {
        self.raw.text_range()
    }

    pub fn text_trimmed_range(&self) -> TextRange {
        self.raw.text_trimmed_range()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
    }

    /// Returns the text of the token, including all trivia.
    ///
    /// ```
    /// use rome_rowan::raw_language::{RawLanguage, RawLanguageKind, RawSyntaxTreeBuilder};
    /// use rome_rowan::*;
    /// let mut token = RawSyntaxTreeBuilder::wrap_with_node(RawLanguageKind::ROOT, |builder| {
    ///     builder.token_with_trivia(
    ///         RawLanguageKind::LET_TOKEN,
    ///         "\n\t let \t\t",
    ///         &[TriviaPiece::whitespace(3)],
    ///         &[TriviaPiece::whitespace(3)],
    ///     );
    /// })
    /// .first_token()
    /// .unwrap();
    /// assert_eq!("\n\t let \t\t", token.text());
    /// ```
    pub fn text(&self) -> &str {
        self.raw.text()
    }

    /// Returns the text of a token, including all trivia as an owned value.
    ///  
    /// ```
    /// use rome_rowan::raw_language::{RawLanguage, RawLanguageKind, RawSyntaxTreeBuilder};
    /// use rome_rowan::*;
    /// let mut token = RawSyntaxTreeBuilder::wrap_with_node(RawLanguageKind::ROOT, |builder| {
    ///     builder.token_with_trivia(
    ///         RawLanguageKind::LET_TOKEN,
    ///         "\n\t let \t\t",
    ///         &[TriviaPiece::whitespace(3)],
    ///         &[TriviaPiece::whitespace(3)],
    ///     );
    /// })
    /// .first_token()
    /// .unwrap();
    /// assert_eq!("\n\t let \t\t", token.token_text());
    /// assert_eq!(
    ///     format!("{}", "\n\t let \t\t"),
    ///     format!("{}", token.token_text())
    /// );
    /// assert_eq!(
    ///     format!("{:?}", "\n\t let \t\t"),
    ///     format!("{:?}", token.token_text())
    /// );
    /// ```
    pub fn token_text(&self) -> SyntaxTokenText {
        self.raw.token_text()
    }

    pub fn token_text_trimmed(&self) -> SyntaxTokenText {
        self.raw.token_text_trimmed()
    }

    /// Returns the text of the token, excluding all trivia.
    ///
    /// ```
    /// use rome_rowan::raw_language::{RawLanguage, RawLanguageKind, RawSyntaxTreeBuilder};
    /// use rome_rowan::*;
    /// let mut token = RawSyntaxTreeBuilder::wrap_with_node(RawLanguageKind::ROOT, |builder| {
    ///     builder.token_with_trivia(
    ///         RawLanguageKind::LET_TOKEN,
    ///         "\n\t let \t\t",
    ///         &[TriviaPiece::whitespace(3)],
    ///         &[TriviaPiece::whitespace(3)],
    ///     );
    /// })
    /// .first_token()
    /// .unwrap();
    /// assert_eq!("let", token.text_trimmed());
    /// ```
    pub fn text_trimmed(&self) -> &str {
        self.raw.text_trimmed()
    }

    pub fn parent(&self) -> Option<SyntaxNode<L>> {
        self.raw.parent().map(SyntaxNode::from)
    }

    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> {
        self.raw.ancestors().map(SyntaxNode::from)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.next_sibling_or_token().map(NodeOrToken::from)
    }
    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.prev_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement<L>> {
        self.raw
            .siblings_with_tokens(direction)
            .map(SyntaxElement::from)
    }

    /// Next token in the tree (i.e, not necessary a sibling).
    pub fn next_token(&self) -> Option<SyntaxToken<L>> {
        self.raw.next_token().map(SyntaxToken::from)
    }
    /// Previous token in the tree (i.e, not necessary a sibling).
    pub fn prev_token(&self) -> Option<SyntaxToken<L>> {
        self.raw.prev_token().map(SyntaxToken::from)
    }

    /// Return a new version of this token detached from its parent node
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn detach(self) -> Self {
        Self {
            raw: self.raw.detach(),
            _p: PhantomData,
        }
    }

    /// Return a new version of this token with its leading trivia replaced with `trivia`
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn with_leading_trivia<'a, I>(self, trivia: I) -> Self
    where
        I: Iterator<Item = (TriviaPieceKind, &'a str)> + Clone + ExactSizeIterator,
    {
        let mut text = String::new();
        for (_, piece) in trivia.clone() {
            text.push_str(piece);
        }

        text.push_str(self.text_trimmed());

        for piece in self.trailing_trivia().pieces() {
            text.push_str(piece.text());
        }

        Self {
            raw: cursor::SyntaxToken::new_detached(GreenToken::with_trivia(
                self.kind().to_raw(),
                &text,
                GreenTrivia::new(
                    trivia.map(|(kind, text)| TriviaPiece::new(kind, TextSize::of(text))),
                ),
                self.green_token().trailing_trivia().clone(),
            )),
            _p: PhantomData,
        }
    }

    /// Return a new version of this token with its trailing trivia replaced with `trivia`
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn with_trailing_trivia<'a, I>(self, trivia: I) -> Self
    where
        I: Iterator<Item = (TriviaPieceKind, &'a str)> + Clone + ExactSizeIterator,
    {
        let mut text = String::new();
        for piece in self.leading_trivia().pieces() {
            text.push_str(piece.text());
        }

        text.push_str(self.text_trimmed());

        for (_, piece) in trivia.clone() {
            text.push_str(piece);
        }

        Self {
            raw: cursor::SyntaxToken::new_detached(GreenToken::with_trivia(
                self.kind().to_raw(),
                &text,
                self.green_token().leading_trivia().clone(),
                GreenTrivia::new(
                    trivia.map(|(kind, text)| TriviaPiece::new(kind, TextSize::of(text))),
                ),
            )),
            _p: PhantomData,
        }
    }

    /// Returns the token leading trivia.
    ///
    /// ```
    /// use rome_rowan::raw_language::{RawLanguage, RawLanguageKind, RawSyntaxTreeBuilder};
    /// use rome_rowan::*;
    /// let mut token = RawSyntaxTreeBuilder::wrap_with_node(RawLanguageKind::ROOT, |builder| {
    ///     builder.token_with_trivia(
    ///         RawLanguageKind::LET_TOKEN,
    ///         "\n\t let \t\t",
    ///         &[TriviaPiece::whitespace(3)],
    ///         &[TriviaPiece::whitespace(3)],
    ///     );
    /// })
    /// .first_token()
    /// .unwrap();
    /// assert_eq!("\n\t ", token.leading_trivia().text());
    /// ```
    #[inline]
    pub fn leading_trivia(&self) -> SyntaxTrivia<L> {
        SyntaxTrivia::new(self.raw.leading_trivia())
    }

    /// Returns the token trailing trivia.
    ///
    /// ```
    /// use rome_rowan::raw_language::{RawLanguage, RawLanguageKind, RawSyntaxTreeBuilder};
    /// use rome_rowan::*;
    /// let mut token = RawSyntaxTreeBuilder::wrap_with_node(RawLanguageKind::ROOT, |builder| {
    ///     builder.token_with_trivia(
    ///         RawLanguageKind::LET_TOKEN,
    ///         "\n\t let \t\t",
    ///         &[TriviaPiece::whitespace(3)],
    ///         &[TriviaPiece::whitespace(3)],
    ///     );
    /// })
    /// .first_token()
    /// .unwrap();
    /// assert_eq!(" \t\t", token.trailing_trivia().text());
    /// ```
    #[inline]
    pub fn trailing_trivia(&self) -> SyntaxTrivia<L> {
        SyntaxTrivia::new(self.raw.trailing_trivia())
    }

    /// Checks if the current token has trailing comments
    pub fn has_trailing_comments(&self) -> bool {
        self.trailing_trivia()
            .pieces()
            .into_iter()
            .any(|piece| piece.is_comments())
    }

    /// Checks if the current token has leading comments
    pub fn has_leading_comments(&self) -> bool {
        self.leading_trivia()
            .pieces()
            .into_iter()
            .any(|piece| piece.is_comments())
    }
}

impl<L: Language> fmt::Debug for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}@{:?} {:?} ",
            self.kind(),
            self.text_range(),
            self.text_trimmed()
        )?;

        write!(f, "[")?;
        let mut first_piece = true;
        for piece in self.leading_trivia().pieces() {
            if !first_piece {
                write!(f, ", ")?;
            }
            first_piece = false;
            write!(f, "{:?}", piece)?;
        }
        write!(f, "] [")?;

        let mut first_piece = true;
        for piece in self.trailing_trivia().pieces() {
            if !first_piece {
                write!(f, ", ")?;
            }
            first_piece = false;
            write!(f, "{:?}", piece)?;
        }
        write!(f, "]")
    }
}

impl<L: Language> fmt::Display for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

impl<L: Language> From<SyntaxToken<L>> for cursor::SyntaxToken {
    fn from(token: SyntaxToken<L>) -> cursor::SyntaxToken {
        token.raw
    }
}

impl<L: Language> From<cursor::SyntaxToken> for SyntaxToken<L> {
    fn from(raw: cursor::SyntaxToken) -> SyntaxToken<L> {
        SyntaxToken {
            raw,
            _p: PhantomData,
        }
    }
}
