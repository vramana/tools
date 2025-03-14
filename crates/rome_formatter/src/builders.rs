use crate::prelude::*;
use crate::{
    format_element, write, Argument, Arguments, BufferSnapshot, FormatState, GroupId,
    PreambleBuffer, TextRange, TextSize,
};
use crate::{Buffer, VecBuffer};
use rome_rowan::{Language, SyntaxNode, SyntaxToken, SyntaxTokenText, TextLen};
use std::borrow::Cow;
use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::Deref;

/// A line break that only gets printed if the enclosing `Group` doesn't fit on a single line.
/// It's omitted if the enclosing `Group` fits on a single line.
/// A soft line break is identical to a hard line break when not enclosed inside of a `Group`.
///
/// # Examples
///
/// Soft line breaks are omitted if the enclosing `Group` fits on a single line
///
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![token("a,"), soft_line_break(), token("b")])
/// ]).unwrap();
///
/// assert_eq!(
///     "a,b",
///     elements.print().as_code()
/// );
/// ```
/// See [soft_line_break_or_space] if you want to insert a space between the elements if the enclosing
/// `Group` fits on a single line.
///
/// Soft line breaks are emitted if the enclosing `Group` doesn't fit on a single line
/// ```
/// use rome_formatter::{format, format_args, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(10).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let elements = format!(context, [
///     group_elements(&format_args![
///         token("a long word,"),
///         soft_line_break(),
///         token("so that the group doesn't fit on a single line"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "a long word,\nso that the group doesn't fit on a single line",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub const fn soft_line_break() -> Line {
    Line::new(LineMode::Soft)
}

/// A forced line break that are always printed. A hard line break forces any enclosing `Group`
/// to be printed over multiple lines.
///
/// # Examples
///
/// It forces a line break, even if the enclosing `Group` would otherwise fit on a single line.
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("a,"),
///         hard_line_break(),
///         token("b"),
///         hard_line_break()
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "a,\nb\n",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub const fn hard_line_break() -> Line {
    Line::new(LineMode::Hard)
}

/// A forced empty line. An empty line inserts enough line breaks in the output for
/// the previous and next element to be separated by an empty line.
///
/// # Examples
///
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(
///     SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("a,"),
///         empty_line(),
///         token("b"),
///         empty_line()
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "a,\n\nb\n\n",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub const fn empty_line() -> Line {
    Line::new(LineMode::Empty)
}

/// A line break if the enclosing `Group` doesn't fit on a single line, a space otherwise.
///
/// # Examples
///
/// The line breaks are emitted as spaces if the enclosing `Group` fits on a a single line:
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("a,"),
///         soft_line_break_or_space(),
///         token("b"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "a, b",
///     elements.print().as_code()
/// );
/// ```
///
/// The printer breaks the lines if the enclosing `Group` doesn't fit on a single line:
/// ```
/// use rome_formatter::{format_args, format, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(10).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let elements = format!(context, [
///     group_elements(&format_args![
///         token("a long word,"),
///         soft_line_break_or_space(),
///         token("so that the group doesn't fit on a single line"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "a long word,\nso that the group doesn't fit on a single line",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub const fn soft_line_break_or_space() -> Line {
    Line::new(LineMode::SoftOrSpace)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Line {
    mode: LineMode,
}

impl Line {
    const fn new(mode: LineMode) -> Self {
        Self { mode }
    }
}

impl<Context> Format<Context> for Line {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::Line(self.mode))
    }
}

impl std::fmt::Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Line").field(&self.mode).finish()
    }
}

/// Creates a token that gets written as is to the output. Make sure to properly escape the text if
/// it's user generated (e.g. a string and not a language keyword).
///
/// # Line feeds
/// Tokens may contain line breaks but they must use the line feeds (`\n`).
/// The [crate::Printer] converts the line feed characters to the character specified in the [crate::PrinterOptions].
///
/// # Examples
///
/// ```
/// use rome_formatter::format;
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [token("Hello World")]).unwrap();
///
/// assert_eq!(
///     "Hello World",
///     elements.print().as_code()
/// );
/// ```
///
/// Printing a string literal as a literal requires that the string literal is properly escaped and
/// enclosed in quotes (depending on the target language).
///
/// ```
/// use rome_formatter::format;
/// use rome_formatter::prelude::*;
///
/// // the tab must be encoded as \\t to not literally print a tab character ("Hello{tab}World" vs "Hello\tWorld")
/// let elements = format!(SimpleFormatContext::default(), [token("\"Hello\\tWorld\"")]).unwrap();
///
/// assert_eq!(r#""Hello\tWorld""#, elements.print().as_code());
/// ```
#[inline]
pub fn token(text: &'static str) -> StaticToken {
    debug_assert_no_newlines(text);

    StaticToken { text }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StaticToken {
    text: &'static str,
}

impl<Context> Format<Context> for StaticToken {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::Token(Token::Static { text: self.text }))
    }
}

impl std::fmt::Debug for StaticToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "StaticToken({})", self.text)
    }
}

/// Create a token from a dynamic string and a range of the input source
pub fn dynamic_token(text: &str, position: TextSize) -> DynamicToken {
    debug_assert_no_newlines(text);

    DynamicToken { text, position }
}

#[derive(Eq, PartialEq)]
pub struct DynamicToken<'a> {
    text: &'a str,
    position: TextSize,
}

impl<Context> Format<Context> for DynamicToken<'_> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::Token(Token::Dynamic {
            text: self.text.to_string().into_boxed_str(),
            source_position: self.position,
        }))
    }
}

impl std::fmt::Debug for DynamicToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "DynamicToken({})", self.text)
    }
}

/// String that is the same as in the input source text if `text` is [`Cow::Borrowed`] or
/// some replaced content if `text` is [`Cow::Owned`].
pub fn syntax_token_cow_slice<'a, L: Language>(
    text: Cow<'a, str>,
    token: &'a SyntaxToken<L>,
    start: TextSize,
) -> SyntaxTokenCowSlice<'a, L> {
    debug_assert_no_newlines(&text);

    SyntaxTokenCowSlice { text, token, start }
}

pub struct SyntaxTokenCowSlice<'a, L: Language> {
    text: Cow<'a, str>,
    token: &'a SyntaxToken<L>,
    start: TextSize,
}

impl<L: Language, Context> Format<Context> for SyntaxTokenCowSlice<'_, L> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        match &self.text {
            Cow::Borrowed(text) => {
                let range = TextRange::at(self.start, text.text_len());
                debug_assert_eq!(
                    *text,
                    &self.token.text()[range - self.token.text_range().start()],
                    "The borrowed string doesn't match the specified token substring. Does the borrowed string belong to this token and range?"
                );

                let relative_range = range - self.token.text_range().start();
                let slice = self.token.token_text().slice(relative_range);

                f.write_element(FormatElement::Token(Token::SyntaxTokenSlice {
                    slice,
                    source_position: self.start,
                }))
            }
            Cow::Owned(text) => f.write_element(FormatElement::Token(Token::Dynamic {
                text: text.to_string().into_boxed_str(),
                source_position: self.start,
            })),
        }
    }
}

impl<L: Language> std::fmt::Debug for SyntaxTokenCowSlice<'_, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "SyntaxTokenCowSlice({})", self.text)
    }
}

/// Copies a source text 1:1 into the output text.
pub fn syntax_token_text_slice<L: Language>(
    token: &SyntaxToken<L>,
    range: TextRange,
) -> SyntaxTokenTextSlice {
    let relative_range = range - token.text_range().start();
    let slice = token.token_text().slice(relative_range);

    debug_assert_no_newlines(&slice);

    SyntaxTokenTextSlice {
        text: slice,
        source_position: range.start(),
    }
}

pub struct SyntaxTokenTextSlice {
    text: SyntaxTokenText,
    source_position: TextSize,
}

impl<Context> Format<Context> for SyntaxTokenTextSlice {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::Token(Token::SyntaxTokenSlice {
            slice: self.text.clone(),
            source_position: self.source_position,
        }))
    }
}

impl std::fmt::Debug for SyntaxTokenTextSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "SyntaxTokenTextSlice({})", self.text)
    }
}

fn debug_assert_no_newlines(text: &str) {
    debug_assert!(!text.contains('\r'), "The content '{}' contains an unsupported '\\r' line terminator character but string tokens must only use line feeds '\\n' as line separator. Use '\\n' instead of '\\r' and '\\r\\n' to insert a line break in strings.", text);
}

/// Pushes some content to the end of the current line
///
/// ## Examples
///
/// ```
/// use rome_formatter::{format};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     token("a"),
///     line_suffix(&token("c")),
///     token("b")
/// ]).unwrap();
///
/// assert_eq!(
///     "abc",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub fn line_suffix<Content, Context>(inner: &Content) -> LineSuffix<Context>
where
    Content: Format<Context>,
{
    LineSuffix {
        content: Argument::new(inner),
    }
}

#[derive(Copy, Clone)]
pub struct LineSuffix<'a, Context> {
    content: Argument<'a, Context>,
}

impl<Context> Format<Context> for LineSuffix<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());
        buffer.write_fmt(Arguments::from(&self.content))?;

        let content = buffer.into_vec();
        f.write_element(FormatElement::LineSuffix(content.into_boxed_slice()))
    }
}

impl<Context> std::fmt::Debug for LineSuffix<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LineSuffix").field(&"{{content}}").finish()
    }
}

/// Inserts a boundary for line suffixes that forces the printer to print all pending line suffixes.
/// Helpful if a line sufix shouldn't pass a certain point.
///
/// ## Examples
///
/// Forces the line suffix "c" to be printed before the token `d`.
/// ```
/// use rome_formatter::format;
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     token("a"),
///     line_suffix(&token("c")),
///     token("b"),
///     line_suffix_boundary(),
///     token("d")
/// ]).unwrap();
///
/// assert_eq!(
///     "abc\nd",
///     elements.print().as_code()
/// );
/// ```
pub const fn line_suffix_boundary() -> LineSuffixBoundary {
    LineSuffixBoundary
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LineSuffixBoundary;

impl<Context> Format<Context> for LineSuffixBoundary {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::LineSuffixBoundary)
    }
}

/// Marks some content as a comment trivia.
///
/// This does not directly influence how this content will be printed, but some
/// parts of the formatter may chose to handle this element in a specific way
///
/// ## Examples
///
/// ```
/// use rome_formatter::{format, write, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(
///     SimpleFormatContext::default(),
///     [
///         group_elements(&format_args![
///             comment(&format_args![token("// test"), hard_line_break()]),
///             format_with(|f| {
///                 write!(f, [
///                     comment(&format_args![token("/* inline */"), hard_line_break()]).memoized(),
///                     token("a"),
///                     soft_line_break_or_space(),
///                 ])
///             }).memoized(),
///             token("b"),
///             soft_line_break_or_space(),
///             token("c")
///         ])
///     ]
/// ).unwrap();
///
/// assert_eq!(
///     "// test\n/* inline */\na b c",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub fn comment<Content, Context>(content: &Content) -> FormatComment<Context>
where
    Content: Format<Context>,
{
    FormatComment {
        content: Argument::new(content),
    }
}

#[derive(Copy, Clone)]
pub struct FormatComment<'a, Context> {
    content: Argument<'a, Context>,
}

impl<Context> Format<Context> for FormatComment<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());

        buffer.write_fmt(Arguments::from(&self.content))?;
        let content = buffer.into_vec();

        f.write_element(FormatElement::Comment(content.into_boxed_slice()))
    }
}

impl<Context> std::fmt::Debug for FormatComment<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Comment").field(&"{{content}}").finish()
    }
}

/// Marks some content with a label.
///
/// This does not directly influence how this content will be printed, but some
/// parts of the formatter may inspect the [labelled element](FormatElement::Label)
/// using `inspect_is_labelled` buffer extension and choose a different formatting layout.
///
/// See [inspect_is_labelled](BufferExtensions::inspect_is_labelled) for documentation.
#[inline]
pub fn labelled<Content, Context>(label_id: LabelId, content: &Content) -> FormatLabelled<Context>
where
    Content: Format<Context>,
{
    FormatLabelled {
        label_id,
        content: Argument::new(content),
    }
}

#[derive(Copy, Clone)]
pub struct FormatLabelled<'a, Context> {
    label_id: LabelId,
    content: Argument<'a, Context>,
}

impl<Context> Format<Context> for FormatLabelled<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());

        buffer.write_fmt(Arguments::from(&self.content))?;
        let content = buffer.into_vec();

        let label = Label::new(self.label_id, content);

        f.write_element(FormatElement::Label(label))
    }
}

impl<Context> std::fmt::Debug for FormatLabelled<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Label")
            .field(&self.label_id)
            .field(&"{{content}}")
            .finish()
    }
}

/// Inserts a single space. Allows to separate different tokens.
///
/// # Examples
///
/// ```
/// use rome_formatter::format;
/// use rome_formatter::prelude::*;
///
/// // the tab must be encoded as \\t to not literally print a tab character ("Hello{tab}World" vs "Hello\tWorld")
/// let elements = format!(SimpleFormatContext::default(), [token("a"), space_token(), token("b")]).unwrap();
///
/// assert_eq!("a b", elements.print().as_code());
/// ```
#[inline]
pub const fn space_token() -> Space {
    Space
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Space;

impl<Context> Format<Context> for Space {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::Space)
    }
}

/// It adds a level of indentation to the given content
///
/// It doesn't add any line breaks at the edges of the content, meaning that
/// the line breaks have to be manually added.
///
/// This helper should be used only in rare cases, instead you should rely more on
/// [block_indent] and [soft_block_indent]
///
/// # Examples
///
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let block = format!(SimpleFormatContext::default(), [
///     token("switch {"),
///     block_indent(&format_args![
///         token("default:"),
///         indent(&format_args![
///             // this is where we want to use a
///             hard_line_break(),
///             token("break;"),
///         ])
///     ]),
///     token("}"),
/// ]).unwrap();
///
/// assert_eq!(
///     "switch {\n\tdefault:\n\t\tbreak;\n}",
///     block.print().as_code()
/// );
/// ```
#[inline]
pub fn indent<Content, Context>(content: &Content) -> Indent<Context>
where
    Content: Format<Context>,
{
    Indent {
        content: Argument::new(content),
    }
}

#[derive(Copy, Clone)]
pub struct Indent<'a, Context> {
    content: Argument<'a, Context>,
}

impl<Context> Format<Context> for Indent<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());

        buffer.write_fmt(Arguments::from(&self.content))?;

        if buffer.is_empty() {
            return Ok(());
        }

        let content = buffer.into_vec();
        f.write_element(FormatElement::Indent(content.into_boxed_slice()))
    }
}

impl<Context> std::fmt::Debug for Indent<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Indent").field(&"{{content}}").finish()
    }
}

/// Inserts a hard line break before and after the content and increases the indention level for the content by one.
///
/// Block indents indent a block of code, such as in a function body, and therefore insert a line
/// break before and after the content.
///
/// Doesn't create an indention if the passed in content is [FormatElement.is_empty].
///
/// # Examples
///
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let block = format![
///     SimpleFormatContext::default(),
///     [
///         token("{"),
///         block_indent(&format_args![
///             token("let a = 10;"),
///             hard_line_break(),
///             token("let c = a + 5;"),
///         ]),
///         token("}"),
///     ]
/// ].unwrap();
///
/// assert_eq!(
///     "{\n\tlet a = 10;\n\tlet c = a + 5;\n}",
///     block.print().as_code()
/// );
/// ```
#[inline]
pub fn block_indent<Context>(content: &impl Format<Context>) -> BlockIndent<Context> {
    BlockIndent {
        content: Argument::new(content),
        mode: IndentMode::Block,
    }
}

/// Indents the content by inserting a line break before and after the content and increasing
/// the indention level for the content by one if the enclosing group doesn't fit on a single line.
/// Doesn't change the formatting if the enclosing group fits on a single line.
///
/// # Examples
///
/// Indents the content by one level and puts in new lines if the enclosing `Group` doesn't fit on a single line
///
/// ```
/// use rome_formatter::{format, format_args, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(10).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let elements = format!(context, [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("'First string',"),
///             soft_line_break_or_space(),
///             token("'second string',"),
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[\n\t'First string',\n\t'second string',\n]",
///     elements.print().as_code()
/// );
/// ```
///
/// Doesn't change the formatting if the enclosing `Group` fits on a single line
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("5,"),
///             soft_line_break_or_space(),
///             token("10"),
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[5, 10]",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub fn soft_block_indent<Context>(content: &impl Format<Context>) -> BlockIndent<Context> {
    BlockIndent {
        content: Argument::new(content),
        mode: IndentMode::Soft,
    }
}

/// If the enclosing `Group` doesn't fit on a single line, inserts a line break and indent.
/// Otherwise, just inserts a space.
///
/// Line indents are used to break a single line of code, and therefore only insert a line
/// break before the content and not after the content.
///
/// # Examples
///
/// Indents the content by one level and puts in new lines if the enclosing `Group` doesn't
/// fit on a single line. Otherwise, just inserts a space.
///
/// ```
/// use rome_formatter::{format, format_args, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(10).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let elements = format!(context, [
///     group_elements(&format_args![
///         token("name"),
///         space_token(),
///         token("="),
///         soft_line_indent_or_space(&format_args![
///             token("firstName"),
///             space_token(),
///             token("+"),
///             space_token(),
///             token("lastName"),
///         ]),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "name =\n\tfirstName + lastName",
///     elements.print().as_code()
/// );
/// ```
///
/// Only adds a space if the enclosing `Group` fits on a single line
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("a"),
///         space_token(),
///         token("="),
///         soft_line_indent_or_space(&token("10")),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "a = 10",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub fn soft_line_indent_or_space<Context>(content: &impl Format<Context>) -> BlockIndent<Context> {
    BlockIndent {
        content: Argument::new(content),
        mode: IndentMode::SoftLineOrSpace,
    }
}

#[derive(Copy, Clone)]
pub struct BlockIndent<'a, Context> {
    content: Argument<'a, Context>,
    mode: IndentMode,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum IndentMode {
    Soft,
    Block,
    SoftLineOrSpace,
}

impl<Context> Format<Context> for BlockIndent<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());

        match self.mode {
            IndentMode::Soft => write!(buffer, [soft_line_break()])?,
            IndentMode::Block => write!(buffer, [hard_line_break()])?,
            IndentMode::SoftLineOrSpace => write!(buffer, [soft_line_break_or_space()])?,
        };

        buffer.write_fmt(Arguments::from(&self.content))?;

        // Don't create an indent if the content is empty
        if buffer.len() == 1 {
            return Ok(());
        }

        let content = buffer.into_vec();

        f.write_element(FormatElement::Indent(content.into_boxed_slice()))?;

        match self.mode {
            IndentMode::Soft => write!(f, [soft_line_break()])?,
            IndentMode::Block => write!(f, [hard_line_break()])?,
            IndentMode::SoftLineOrSpace => {}
        }

        Ok(())
    }
}

impl<Context> std::fmt::Debug for BlockIndent<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.mode {
            IndentMode::Soft => "SoftBlockIndent",
            IndentMode::Block => "HardBlockIndent",
            IndentMode::SoftLineOrSpace => "SoftLineIndentOrSpace",
        };

        f.debug_tuple(name).field(&"{{content}}").finish()
    }
}

/// Creates a logical `Group` around the content that should either consistently be printed on a single line
/// or broken across multiple lines.
///
/// The printer will try to print the content of the `Group` on a single line, ignoring all soft line breaks and
/// emitting spaces for soft line breaks or spaces. The printer tracks back if it isn't successful either
/// because it encountered a hard line break, or because printing the `Group` on a single line exceeds
/// the configured line width, and thus it must print all its content on multiple lines,
/// emitting line breaks for all line break kinds.
///
/// # Examples
///
/// `Group` that fits on a single line
///
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("1,"),
///             soft_line_break_or_space(),
///             token("2,"),
///             soft_line_break_or_space(),
///             token("3"),
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[1, 2, 3]",
///     elements.print().as_code()
/// );
/// ```
///
/// The printer breaks the `Group` over multiple lines if its content doesn't fit on a single line
/// ```
/// use rome_formatter::{format, format_args, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(20).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let elements = format!(context, [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("'Good morning! How are you today?',"),
///             soft_line_break_or_space(),
///             token("2,"),
///             soft_line_break_or_space(),
///             token("3"),
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[\n\t'Good morning! How are you today?',\n\t2,\n\t3\n]",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub fn group_elements<Context>(content: &impl Format<Context>) -> GroupElements<Context> {
    GroupElements {
        content: Argument::new(content),
        group_id: None,
    }
}

#[derive(Copy, Clone)]
pub struct GroupElements<'a, Context> {
    content: Argument<'a, Context>,
    group_id: Option<GroupId>,
}

impl<Context> GroupElements<'_, Context> {
    pub fn with_group_id(mut self, group_id: Option<GroupId>) -> Self {
        self.group_id = group_id;
        self
    }
}

impl<Context> Format<Context> for GroupElements<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = GroupElementsBuffer::new(f);
        buffer.write_fmt(Arguments::from(&self.content))?;
        let content = buffer.into_vec();

        if content.is_empty() && self.group_id.is_none() {
            return Ok(());
        }

        let group = Group::new(content).with_id(self.group_id);

        f.write_element(FormatElement::Group(group))?;

        Ok(())
    }
}

impl<Context> std::fmt::Debug for GroupElements<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupElements")
            .field("group_id", &self.group_id)
            .field("content", &"{{content}}")
            .finish()
    }
}

/// Custom buffer implementation for `GroupElements` that moves the leading comments out of the group
/// to prevent that a leading line comment expands the token's enclosing group.
///
/// # Examples
///
/// ```javascript
/// /* a comment */
/// [1]
/// ```
///
/// The `/* a comment */` belongs to the `[` group token that is part of a group wrapping the whole
/// `[1]` expression. It's important that the comment `/* a comment */` gets moved out of the group element
/// to avoid that the `[1]` group expands because of the line break inserted by the comment.
struct GroupElementsBuffer<'inner, Context> {
    inner: &'inner mut dyn Buffer<Context = Context>,

    /// The group inner content
    content: Vec<FormatElement>,
}

impl<'inner, Context> GroupElementsBuffer<'inner, Context> {
    fn new(inner: &'inner mut dyn Buffer<Context = Context>) -> Self {
        Self {
            inner,
            content: Vec::new(),
        }
    }

    fn into_vec(self) -> Vec<FormatElement> {
        self.content
    }

    fn write_interned(&mut self, interned: Interned) -> FormatResult<()> {
        debug_assert!(self.content.is_empty());

        match interned.deref() {
            FormatElement::Comment(_) => {
                self.inner.write_element(FormatElement::Interned(interned))
            }
            FormatElement::List(list) => {
                let mut content_start = 0;

                for element in list.iter() {
                    match element {
                        element @ FormatElement::Comment(_) => {
                            content_start += 1;
                            // Cloning comments should be alright as they are rarely nested
                            // and the case where all elements of an interned data structure are comments
                            // are rare
                            self.inner.write_element(element.clone())?;
                        }
                        FormatElement::Interned(interned) => {
                            self.write_interned(interned.clone())?;
                            content_start += 1;

                            if !self.content.is_empty() {
                                // Interned struct contained non-comment
                                break;
                            }
                        }
                        _ => {
                            // Found the first non-comment / nested interned element
                            break;
                        }
                    }
                }

                // No leading comments, this group has no comments
                if content_start == 0 {
                    self.content.push(FormatElement::Interned(interned));
                    return Ok(());
                }

                let content = &list[content_start..];

                // It is necessary to mutate the interned elements, write cloned elements
                self.write_elements(content.iter().cloned())
            }
            FormatElement::Interned(interned) => self.write_interned(interned.clone()),
            _ => {
                self.content.push(FormatElement::Interned(interned));
                Ok(())
            }
        }
    }
}

impl<Context> Buffer for GroupElementsBuffer<'_, Context> {
    type Context = Context;

    fn write_element(&mut self, element: FormatElement) -> FormatResult<()> {
        if self.content.is_empty() {
            match element {
                FormatElement::List(list) => {
                    self.write_elements(list.into_vec())?;
                }
                FormatElement::Interned(interned) => match Interned::try_unwrap(interned) {
                    Ok(owned) => self.write_element(owned)?,
                    Err(interned) => self.write_interned(interned)?,
                },
                comment @ FormatElement::Comment { .. } => {
                    self.inner.write_element(comment)?;
                }
                element => self.content.push(element),
            }
        } else {
            match element {
                FormatElement::List(list) => {
                    self.content.extend(list.into_vec());
                }
                element => self.content.push(element),
            }
        }

        Ok(())
    }

    fn state(&self) -> &FormatState<Self::Context> {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<Self::Context> {
        self.inner.state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        BufferSnapshot::Any(Box::new(GroupElementsBufferSnapshot {
            inner: self.inner.snapshot(),
            content_len: self.content.len(),
        }))
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        let snapshot = snapshot.unwrap_any::<GroupElementsBufferSnapshot>();
        self.inner.restore_snapshot(snapshot.inner);
        self.content.truncate(snapshot.content_len);
    }
}

struct GroupElementsBufferSnapshot {
    inner: BufferSnapshot,
    content_len: usize,
}

/// IR element that forces the parent group to print in expanded mode.
///
/// Has no effect if used outside of a group or element that introduce implicit groups (fill element).
///
/// ## Examples
///
/// ```
/// use rome_formatter::{format, format_args, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("'Good morning! How are you today?',"),
///             soft_line_break_or_space(),
///             token("2,"),
///             expand_parent(), // Forces the parent to expand
///             soft_line_break_or_space(),
///             token("3"),
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[\n\t'Good morning! How are you today?',\n\t2,\n\t3\n]",
///     elements.print().as_code()
/// );
/// ```
///
/// # Prettier
/// Equivalent to Prettier's `break_parent` IR element
pub const fn expand_parent() -> ExpandParent {
    ExpandParent
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExpandParent;

impl<Context> Format<Context> for ExpandParent {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        f.write_element(FormatElement::ExpandParent)
    }
}

/// Adds a conditional content that is emitted only if it isn't inside an enclosing `Group` that
/// is printed on a single line. The element allows, for example, to insert a trailing comma after the last
/// array element only if the array doesn't fit on a single line.
///
/// The element has no special meaning if used outside of a `Group`. In that case, the content is always emitted.
///
/// If you're looking for a way to only print something if the `Group` fits on a single line see [crate::if_group_fits_on_line].
///
/// # Examples
///
/// Omits the trailing comma for the last array element if the `Group` fits on a single line
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let elements = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("1,"),
///             soft_line_break_or_space(),
///             token("2,"),
///             soft_line_break_or_space(),
///             token("3"),
///             if_group_breaks(&token(","))
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[1, 2, 3]",
///     elements.print().as_code()
/// );
/// ```
///
/// Prints the trailing comma for the last array element if the `Group` doesn't fit on a single line
/// ```
/// use rome_formatter::{format_args, format, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(20).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let elements = format!(context, [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("'A somewhat longer string to force a line break',"),
///             soft_line_break_or_space(),
///             token("2,"),
///             soft_line_break_or_space(),
///             token("3"),
///             if_group_breaks(&token(","))
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// let options = PrinterOptions {
///     print_width: LineWidth::try_from(20).unwrap(),
///     ..PrinterOptions::default()
/// };
/// assert_eq!(
///     "[\n\t'A somewhat longer string to force a line break',\n\t2,\n\t3,\n]",
///     elements.print().as_code()
/// );
/// ```
#[inline]
pub fn if_group_breaks<Content, Context>(content: &Content) -> IfGroupBreaks<Context>
where
    Content: Format<Context>,
{
    IfGroupBreaks {
        content: Argument::new(content),
        group_id: None,
        mode: PrintMode::Expanded,
    }
}

/// Adds a conditional content specific for `Group`s that fit on a single line. The content isn't
/// emitted for `Group`s spanning multiple lines.
///
/// See [if_group_breaks] if you're looking for a way to print content only for groups spanning multiple lines.
///
/// # Examples
///
/// Adds the trailing comma for the last array element if the `Group` fits on a single line
/// ```
/// use rome_formatter::{format, format_args};
/// use rome_formatter::prelude::*;
///
/// let formatted = format!(SimpleFormatContext::default(), [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("1,"),
///             soft_line_break_or_space(),
///             token("2,"),
///             soft_line_break_or_space(),
///             token("3"),
///             if_group_fits_on_line(&token(","))
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[1, 2, 3,]",
///     formatted.print().as_code()
/// );
/// ```
///
/// Omits the trailing comma for the last array element if the `Group` doesn't fit on a single line
/// ```
/// use rome_formatter::{format, format_args, LineWidth};
/// use rome_formatter::prelude::*;
///
/// let context = SimpleFormatContext {
///     line_width: LineWidth::try_from(20).unwrap(),
///     ..SimpleFormatContext::default()
/// };
///
/// let formatted = format!(context, [
///     group_elements(&format_args![
///         token("["),
///         soft_block_indent(&format_args![
///             token("'A somewhat longer string to force a line break',"),
///             soft_line_break_or_space(),
///             token("2,"),
///             soft_line_break_or_space(),
///             token("3"),
///             if_group_fits_on_line(&token(","))
///         ]),
///         token("]"),
///     ])
/// ]).unwrap();
///
/// assert_eq!(
///     "[\n\t'A somewhat longer string to force a line break',\n\t2,\n\t3\n]",
///     formatted.print().as_code()
/// );
/// ```
#[inline]
pub fn if_group_fits_on_line<Content, Context>(flat_content: &Content) -> IfGroupBreaks<Context>
where
    Content: Format<Context>,
{
    IfGroupBreaks {
        mode: PrintMode::Flat,
        group_id: None,
        content: Argument::new(flat_content),
    }
}

#[derive(Copy, Clone)]
pub struct IfGroupBreaks<'a, Context> {
    content: Argument<'a, Context>,
    group_id: Option<GroupId>,
    mode: PrintMode,
}

impl<Context> IfGroupBreaks<'_, Context> {
    /// Inserts some content that the printer only prints if the group with the specified `group_id`
    /// is printed in multiline mode. The referred group must appear before this element in the document
    /// but doesn't have to one of its ancestors.
    ///
    /// # Examples
    ///
    /// Prints the trailing comma if the array group doesn't fit. The `group_id` is necessary
    /// because `fill` creates an implicit group around each item and tries to print the item in flat mode.
    /// The item `[4]` in this example fits on a single line but the trailing comma should still be printed
    ///
    /// ```
    /// use rome_formatter::{format, format_args, write, LineWidth};
    /// use rome_formatter::prelude::*;
    ///
    /// let context = SimpleFormatContext {
    ///     line_width: LineWidth::try_from(20).unwrap(),
    ///     ..SimpleFormatContext::default()
    /// };
    ///
    /// let formatted = format!(context, [format_with(|f| {
    ///     let group_id = f.group_id("array");
    ///
    ///     write!(f, [
    ///         group_elements(
    ///             &format_args![
    ///                 token("["),
    ///                 soft_block_indent(&format_with(|f| {
    ///                     f.fill(soft_line_break_or_space())
    ///                         .entry(&token("1,"))
    ///                         .entry(&token("234568789,"))
    ///                         .entry(&token("3456789,"))
    ///                         .entry(&format_args!(
    ///                             token("["),
    ///                             soft_block_indent(&token("4")),
    ///                             token("]"),
    ///                             if_group_breaks(&token(",")).with_group_id(Some(group_id))
    ///                         ))
    ///                         .finish()
    ///                 })),
    ///                 token("]")
    ///             ],
    ///         ).with_group_id(Some(group_id))
    ///     ])
    /// })]).unwrap();
    ///
    /// assert_eq!(
    ///     "[\n\t1, 234568789,\n\t3456789, [4],\n]",
    ///     formatted.print().as_code()
    /// );
    /// ```
    pub fn with_group_id(mut self, group_id: Option<GroupId>) -> Self {
        self.group_id = group_id;
        self
    }
}

impl<Context> Format<Context> for IfGroupBreaks<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());

        buffer.write_fmt(Arguments::from(&self.content))?;

        if buffer.is_empty() {
            return Ok(());
        }

        let content = buffer.into_vec();
        f.write_element(FormatElement::ConditionalGroupContent(
            ConditionalGroupContent::new(content.into_boxed_slice(), self.mode)
                .with_group_id(self.group_id),
        ))
    }
}

impl<Context> std::fmt::Debug for IfGroupBreaks<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.mode {
            PrintMode::Flat => "IfGroupFitsOnLine",
            PrintMode::Expanded => "IfGroupBreaks",
        };

        f.debug_struct(name)
            .field("group_id", &self.group_id)
            .field("content", &"{{content}}")
            .finish()
    }
}

/// Utility for formatting some content with an inline lambda function.
#[derive(Copy, Clone)]
pub struct FormatWith<Context, T> {
    formatter: T,
    context: PhantomData<Context>,
}

impl<Context, T> Format<Context> for FormatWith<Context, T>
where
    T: Fn(&mut Formatter<Context>) -> FormatResult<()>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        (self.formatter)(f)
    }
}

impl<Context, T> std::fmt::Debug for FormatWith<Context, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormatWith").field(&"{{formatter}}").finish()
    }
}

/// Creates an object implementing `Format` that calls the passed closure to perform the formatting.
///
/// # Examples
///
/// ```
/// use rome_formatter::prelude::*;
/// use rome_formatter::{SimpleFormatContext, format, write};
/// use rome_rowan::TextSize;
///
/// struct MyFormat {
///     items: Vec<&'static str>,
/// }
///
/// impl Format<SimpleFormatContext> for MyFormat {
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>) -> FormatResult<()> {
///         write!(f, [
///             token("("),
///             block_indent(&format_with(|f| {
///                 let separator = space_token();
///                 let mut join = f.join_with(&separator);
///
///                 for item in &self.items {
///                     join.entry(&format_with(|f| write!(f, [dynamic_token(item, TextSize::default())])));
///                 }
///                 join.finish()
///             })),
///             token(")")
///         ])
///     }
/// }
///
/// let formatted = format!(SimpleFormatContext::default(), [MyFormat { items: vec!["a", "b", "c"]}]).unwrap();
///
/// assert_eq!("(\n\ta b c\n)", formatted.print().as_code());
/// ```
pub const fn format_with<Context, T>(formatter: T) -> FormatWith<Context, T>
where
    T: Fn(&mut Formatter<Context>) -> FormatResult<()>,
{
    FormatWith {
        formatter,
        context: PhantomData,
    }
}

/// Creates an inline `Format` object that can only be formatted once.
///
/// This can be useful in situation where the borrow checker doesn't allow you to use [`format_with`]
/// because the code formatting the content consumes the value and cloning the value is too expensive.
/// An example of this is if you want to nest a `FormatElement` or non-cloneable `Iterator` inside of a
/// `block_indent` as shown can see in the examples section.
///
/// # Panics
///
/// Panics if the object gets formatted more than once.
///
/// # Example
///
/// ```
/// use rome_formatter::prelude::*;
/// use rome_formatter::{SimpleFormatContext, format, write, Buffer};
///
/// struct MyFormat;
///
/// fn generate_values() -> impl Iterator<Item=StaticToken> {
///     vec![token("1"), token("2"), token("3"), token("4")].into_iter()
/// }
///
/// impl Format<SimpleFormatContext> for MyFormat {
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>) -> FormatResult<()> {
///         let mut values = generate_values();
///
///         let first = values.next();
///
///         // Formats the first item outside of the block and all other items inside of the block,
///         // separated by line breaks
///         write!(f, [
///             first,
///             block_indent(&format_once(|f| {
///                 // Using format_with isn't possible here because the iterator gets consumed here
///                 f.join_with(&hard_line_break()).entries(values).finish()
///             })),
///         ])
///     }
/// }
///
/// let formatted = format!(SimpleFormatContext::default(), [MyFormat]).unwrap();
///
/// assert_eq!("1\n\t2\n\t3\n\t4\n", formatted.print().as_code());
/// ```
///
/// Formatting the same value twice results in a panic.
///
/// ```panics
/// use rome_formatter::prelude::*;
/// use rome_formatter::{SimpleFormatContext, format, write, Buffer};
/// use rome_rowan::TextSize;
///
/// let mut count = 0;
///
/// let value = format_once(|f| {
///     write!(f, [dynamic_token(&std::format!("Formatted {count}."), TextSize::default())])
/// });
///
/// format!(SimpleFormatContext::default(), [value]).expect("Formatting once works fine");
///
/// // Formatting the value more than once panics
/// format!(SimpleFormatContext::default(), [value]);
/// ```
pub const fn format_once<T, Context>(formatter: T) -> FormatOnce<T, Context>
where
    T: FnOnce(&mut Formatter<Context>) -> FormatResult<()>,
{
    FormatOnce {
        formatter: Cell::new(Some(formatter)),
        context: PhantomData,
    }
}

pub struct FormatOnce<T, Context> {
    formatter: Cell<Option<T>>,
    context: PhantomData<Context>,
}

impl<T, Context> Format<Context> for FormatOnce<T, Context>
where
    T: FnOnce(&mut Formatter<Context>) -> FormatResult<()>,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let formatter = self.formatter.take().expect("Tried to format a `format_once` at least twice. This is not allowed. You may want to use `format_with` or `format.memoized` instead.");

        (formatter)(f)
    }
}

impl<T, Context> std::fmt::Debug for FormatOnce<T, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormatOnce").field(&"{{formatter}}").finish()
    }
}

/// Builder to join together a sequence of content.
/// See [Formatter::join]
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct JoinBuilder<'fmt, 'buf, Separator, Context> {
    result: FormatResult<()>,
    fmt: &'fmt mut Formatter<'buf, Context>,
    with: Option<Separator>,
    has_elements: bool,
}

impl<'fmt, 'buf, Separator, Context> JoinBuilder<'fmt, 'buf, Separator, Context>
where
    Separator: Format<Context>,
{
    /// Creates a new instance that joins the elements without a separator
    pub(super) fn new(fmt: &'fmt mut Formatter<'buf, Context>) -> Self {
        Self {
            result: Ok(()),
            fmt,
            has_elements: false,
            with: None,
        }
    }

    /// Creates a new instance that prints the passed separator between every two entries.
    pub(super) fn with_separator(fmt: &'fmt mut Formatter<'buf, Context>, with: Separator) -> Self {
        Self {
            result: Ok(()),
            fmt,
            has_elements: false,
            with: Some(with),
        }
    }

    /// Adds a new entry to the join output.
    pub fn entry(&mut self, entry: &dyn Format<Context>) -> &mut Self {
        self.result = self.result.and_then(|_| {
            if let Some(with) = &self.with {
                if self.has_elements {
                    with.fmt(self.fmt)?;
                }
            }
            self.has_elements = true;

            entry.fmt(self.fmt)
        });

        self
    }

    /// Adds the contents of an iterator of entries to the join output.
    pub fn entries<F, I>(&mut self, entries: I) -> &mut Self
    where
        F: Format<Context>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.entry(&entry);
        }

        self
    }

    /// Finishes the output and returns any error encountered.
    pub fn finish(&mut self) -> FormatResult<()> {
        self.result
    }
}

/// Builder to join together nodes that ensures that nodes separated by empty lines continue
/// to be separated by empty lines in the formatted output.
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct JoinNodesBuilder<'fmt, 'buf, Separator, Context> {
    result: FormatResult<()>,
    /// The separator to insert between nodes. Either a soft or hard line break
    separator: Separator,
    fmt: &'fmt mut Formatter<'buf, Context>,
    has_elements: bool,
}

impl<'fmt, 'buf, Separator, Context> JoinNodesBuilder<'fmt, 'buf, Separator, Context>
where
    Separator: Format<Context>,
{
    pub(super) fn new(separator: Separator, fmt: &'fmt mut Formatter<'buf, Context>) -> Self {
        Self {
            result: Ok(()),
            separator,
            fmt,
            has_elements: false,
        }
    }

    /// Adds a new node with the specified formatted content to the output, respecting any new lines
    /// that appear before the node in the input source.
    pub fn entry<L: Language>(&mut self, node: &SyntaxNode<L>, content: &dyn Format<Context>) {
        self.result = self.result.and_then(|_| {
            let mut buffer = PreambleBuffer::new(
                self.fmt,
                format_with(|f| {
                    if self.has_elements {
                        if get_lines_before(node) > 1 {
                            write!(f, [empty_line()])?;
                        } else {
                            self.separator.fmt(f)?;
                        }
                    }

                    Ok(())
                }),
            );

            write!(buffer, [content])?;

            self.has_elements = self.has_elements || buffer.did_write_preamble();

            Ok(())
        });
    }

    /// Adds an iterator of entries to the output. Each entry is a `(node, content)` tuple.
    pub fn entries<L, F, I>(&mut self, entries: I) -> &mut Self
    where
        L: Language,
        F: Format<Context>,
        I: IntoIterator<Item = (SyntaxNode<L>, F)>,
    {
        for (node, content) in entries {
            self.entry(&node, &content)
        }

        self
    }

    pub fn finish(&mut self) -> FormatResult<()> {
        self.result
    }
}

/// Get the number of line breaks between two consecutive SyntaxNodes in the tree
pub fn get_lines_before<L: Language>(next_node: &SyntaxNode<L>) -> usize {
    // Count the newlines in the leading trivia of the next node
    if let Some(leading_trivia) = next_node.first_leading_trivia() {
        leading_trivia
            .pieces()
            .take_while(|piece| {
                // Stop at the first comment piece, the comment printer
                // will handle newlines between the comment and the node
                !piece.is_comments()
            })
            .filter(|piece| piece.is_newline())
            .count()
    } else {
        0
    }
}

/// Builder to fill as many elements as possible on a single line.
#[must_use = "must eventually call `finish()` on Format builders"]
pub struct FillBuilder<'fmt, 'buf, Context> {
    result: FormatResult<()>,
    fmt: &'fmt mut Formatter<'buf, Context>,

    /// The separator to use to join the elements
    separator: FormatElement,
    items: Vec<FormatElement>,
}

impl<'a, 'buf, Context> FillBuilder<'a, 'buf, Context> {
    pub(crate) fn new<Separator>(
        fmt: &'a mut Formatter<'buf, Context>,
        separator: Separator,
    ) -> Self
    where
        Separator: Format<Context>,
    {
        let mut buffer = VecBuffer::new(fmt.state_mut());
        let result = write!(buffer, [separator]);
        let separator = buffer.into_element();

        Self {
            result,
            fmt,
            separator,
            items: vec![],
        }
    }

    /// Adds an iterator of entries to the fill output.
    pub fn entries<F, I>(&mut self, entries: I) -> &mut Self
    where
        F: Format<Context>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.entry(&entry);
        }

        self
    }

    /// Adds an iterator of entries to the fill output, flattening any [FormatElement::List]
    /// entries by adding the list's elements to the fill output.
    ///
    /// ## Warning
    ///
    /// The usage of this method is highly discouraged and it's better to use
    /// other APIs on ways: for example progressively format the items based on their type.
    pub fn flatten_entries<F, I>(&mut self, entries: I) -> &mut Self
    where
        F: Format<Context>,
        I: IntoIterator<Item = F>,
    {
        for entry in entries {
            self.flatten_entry(&entry);
        }

        self
    }

    /// Adds a new entry to the fill output. If the entry is a [FormatElement::List],
    /// then adds the list's entries to the fill output instead of the list itself.
    pub fn flatten_entry(&mut self, entry: &dyn Format<Context>) -> &mut Self {
        self.result = self.result.and_then(|_| {
            let mut buffer = VecBuffer::new(self.fmt.state_mut());
            write!(buffer, [entry])?;

            self.items.extend(buffer.into_vec());

            Ok(())
        });

        self
    }

    /// Adds a new entry to the fill output.
    pub fn entry(&mut self, entry: &dyn Format<Context>) -> &mut Self {
        self.result = self.result.and_then(|_| {
            let mut buffer = VecBuffer::new(self.fmt.state_mut());
            write!(buffer, [entry])?;

            let item = buffer.into_element();

            if !item.is_empty() {
                self.items.push(item);
            }

            Ok(())
        });

        self
    }

    /// Finishes the output and returns any error encountered
    pub fn finish(&mut self) -> FormatResult<()> {
        self.result.and_then(|_| {
            let mut items = std::mem::take(&mut self.items);

            match items.len() {
                0 => Ok(()),
                1 => self.fmt.write_element(items.pop().unwrap()),
                _ => self.fmt.write_element(FormatElement::Fill(Fill {
                    content: items.into_boxed_slice(),
                    separator: Box::new(self.separator.clone()),
                })),
            }
        })
    }
}

/// The first variant is the most flat, and the last is the most expanded variant.
/// See [`best_fitting!`] macro for a more in-detail documentation
#[derive(Copy, Clone)]
pub struct BestFitting<'a, Context> {
    variants: Arguments<'a, Context>,
}

impl<'a, Context> BestFitting<'a, Context> {
    /// Creates a new best fitting IR with the given variants. The method itself isn't unsafe
    /// but it is to discourage people from using it because the printer will panic if
    /// the slice doesn't contain at least the least and most expanded variants.
    ///
    /// You're looking for a way to create a `BestFitting` object, use the `best_fitting![least_expanded, most_expanded]` macro.
    ///
    /// ## Safety
    /// The slice must contain at least two variants.
    pub unsafe fn from_arguments_unchecked(variants: Arguments<'a, Context>) -> Self {
        debug_assert!(
            variants.0.len() >= 2,
            "Requires at least the least expanded and most expanded variants"
        );

        Self { variants }
    }
}

impl<Context> Format<Context> for BestFitting<'_, Context> {
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        let mut buffer = VecBuffer::new(f.state_mut());
        let variants = self.variants.items();

        let mut formatted_variants = Vec::with_capacity(variants.len());

        for variant in variants {
            buffer.write_fmt(Arguments::from(&*variant))?;

            formatted_variants.push(buffer.take_element());
        }

        // SAFETY: The constructor guarantees that there are always at least two variants. It's, therefore,
        // safe to call into the unsafe `from_vec_unchecked` function
        let element = unsafe {
            FormatElement::BestFitting(format_element::BestFitting::from_vec_unchecked(
                formatted_variants,
            ))
        };

        f.write_element(element)?;

        Ok(())
    }
}
