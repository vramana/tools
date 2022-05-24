use crate::comments::{CommentKind, FormatPrecedingComments};
use crate::comments::{CommentStyle, PreviousToken};
use crate::group_id::UniqueGroupIdBuilder;
use crate::prelude::*;
#[cfg(debug_assertions)]
use crate::printed_tokens::PrintedTokens;
use crate::{ConcatBuilder, GroupId, TextSize};
use rome_rowan::raw_language::RawLanguageKind;
use rome_rowan::Direction::Prev;
use rome_rowan::{
    Language, RawSyntaxKind, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTokenText,
    SyntaxTriviaPieceComments,
};
use smallvec::SmallVec;
use std::any::TypeId;
#[cfg(debug_assertions)]
use std::cell::RefCell;
use std::collections::HashMap;

/// Handles the formatting of a CST and stores the options how the CST should be formatted (user preferences).
/// The formatter is passed to the [Format] implementation of every node in the CST so that they
/// can use it to format their children.
#[derive(Default)]
pub struct Formatter<Options> {
    options: Options,
    group_id_builder: UniqueGroupIdBuilder,
    // TODO: Explore how much work it would be too to change `Format` to `&mut Formatter`
    prev_token_overrides: RefCell<HashMap<TokenId, TokenKind>>,
    // This is using a RefCell as it only exists in debug mode,
    // the Formatter is still completely immutable in release builds
    #[cfg(debug_assertions)]
    pub printed_tokens: RefCell<PrintedTokens>,
}

#[derive(Hash, Eq, PartialEq)]
struct TokenId {
    language: TypeId,
    kind: RawSyntaxKind,
    offset: TextSize,
}

impl<L> From<&SyntaxToken<L>> for TokenId
where
    L: Language + 'static,
{
    fn from(token: &SyntaxToken<L>) -> Self {
        TokenId {
            language: TypeId::of::<L>(),
            offset: token.text_range().start(),
            kind: token.kind().to_raw(),
        }
    }
}

struct TokenKind {
    language: TypeId,
    kind: RawSyntaxKind,
}

impl TokenKind {
    fn of<L>(kind: L::Kind) -> Self
    where
        L: Language + 'static,
    {
        TokenKind {
            language: TypeId::of::<L>(),
            kind: kind.to_raw(),
        }
    }
}

impl<Options> Formatter<Options> {
    /// Creates a new context that uses the given formatter options
    pub fn new(options: Options) -> Self {
        Self {
            options,
            group_id_builder: Default::default(),
            prev_token_overrides: Default::default(),
            #[cfg(debug_assertions)]
            printed_tokens: Default::default(),
        }
    }

    /// Returns the [FormatOptions] specifying how to format the current CST
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.group_id_builder.group_id(debug_name)
    }

    pub fn override_prev_token<L>(&self, token: &SyntaxToken<L>, kind: L::Kind)
    where
        L: Language + 'static,
    {
        let mut prev_tokens = self.prev_token_overrides.borrow_mut();

        let id = TokenId::from(token);

        prev_tokens.insert(id, TokenKind::of::<L>(kind));
    }

    fn get_prev_token_override<L>(&self, token: &SyntaxToken<L>) -> Option<L::Kind>
    where
        L: Language + 'static,
    {
        let id = TokenId::from(token);

        self.prev_token_overrides
            .borrow()
            .get(&id)
            .and_then(|prev_override| {
                if prev_override.language == TypeId::of::<L>() {
                    Some(L::Kind::from_raw(prev_override.kind))
                } else {
                    None
                }
            })
    }

    pub fn format_preceding_comments<'a, S>(
        &self,
        token: &'a SyntaxToken<S::Language>,
    ) -> FormatPrecedingComments<'a, S, Options>
    where
        S: CommentStyle,
        S::Language: 'static,
    {
        let prev_token = if let Some(token_override) = self.get_prev_token_override(token) {
            PreviousToken::Override {
                kind: token_override,
            }
        } else {
            PreviousToken::Token(token.prev_token())
        };

        FormatPrecedingComments::new(token, prev_token)
    }

    /// Tracks the given token as formatted
    #[inline]
    pub fn track_token<L: Language>(&self, #[allow(unused_variables)] token: &SyntaxToken<L>) {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                self.printed_tokens.borrow_mut().track_token(token);
            }
        }
    }

    #[inline]
    pub fn assert_formatted_all_tokens<L: Language>(
        &self,
        #[allow(unused_variables)] root: &SyntaxNode<L>,
    ) {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                let printed_tokens = self.printed_tokens.borrow();
                printed_tokens.assert_all_tracked(root);
            }
        }
    }

    /// Formats all items of the iterator and returns the formatted result
    ///
    /// Returns the [Err] of the first item that failed to format.
    #[inline]
    pub fn format_all<T: Format<Options = Options>>(
        &self,
        nodes: impl IntoIterator<Item = T>,
    ) -> FormatResult<impl Iterator<Item = FormatElement>> {
        let mut result = Vec::new();

        for node in nodes {
            match node.format(self) {
                Ok(formatted) => {
                    result.push(formatted);
                }
                Err(err) => return Err(err),
            }
        }

        Ok(result.into_iter())
    }
}

impl<Options> Formatter<Options> {
    /// Take a snapshot of the state of the formatter
    #[inline]
    pub fn snapshot(&self) -> FormatterSnapshot {
        FormatterSnapshot {
            #[cfg(debug_assertions)]
            printed_tokens: self.printed_tokens.borrow().clone(),
        }
    }

    #[inline]
    /// Restore the state of the formatter to a previous snapshot
    pub fn restore(&self, #[allow(unused)] snapshot: FormatterSnapshot) {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                *self.printed_tokens.borrow_mut() = snapshot.printed_tokens;
            }
        }
    }
}

/// Snapshot of the formatter state  used to handle backtracking if
/// errors are encountered in the formatting process and the formatter
/// has to fallback to printing raw tokens
///
/// In practice this only saves the set of printed tokens in debug
/// mode and compiled to nothing in release mode
pub struct FormatterSnapshot {
    #[cfg(debug_assertions)]
    printed_tokens: PrintedTokens,
}
