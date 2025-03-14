use crate::buffer::BufferSnapshot;
use crate::builders::{FillBuilder, JoinBuilder, JoinNodesBuilder, Line};
use crate::prelude::*;
use crate::{Arguments, Buffer, FormatState, FormatStateSnapshot, GroupId, VecBuffer};

/// Handles the formatting of a CST and stores the context how the CST should be formatted (user preferences).
/// The formatter is passed to the [Format] implementation of every node in the CST so that they
/// can use it to format their children.
pub struct Formatter<'buf, Context> {
    pub(super) buffer: &'buf mut dyn Buffer<Context = Context>,
}

impl<'buf, Context> Formatter<'buf, Context> {
    /// Creates a new context that uses the given formatter context
    pub fn new(buffer: &'buf mut (dyn Buffer<Context = Context> + 'buf)) -> Self {
        Self { buffer }
    }

    /// Returns the Context specifying how to format the current CST
    pub fn context(&self) -> &Context {
        self.state().context()
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.state().group_id(debug_name)
    }

    /// Joins multiple [Format] together without any separator
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rome_formatter::format;
    /// use rome_formatter::prelude::*;
    ///
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.join()
    ///         .entry(&token("a"))
    ///         .entry(&space_token())
    ///         .entry(&token("+"))
    ///         .entry(&space_token())
    ///         .entry(&token("b"))
    ///         .finish()
    /// })]).unwrap();
    ///
    /// assert_eq!(
    ///     "a + b",
    ///     formatted.print().as_code()
    /// )
    /// ```
    pub fn join<'a>(&'a mut self) -> JoinBuilder<'a, 'buf, (), Context> {
        JoinBuilder::new(self)
    }

    /// Joins the objects by placing the specified separator between every two items.
    ///
    /// ## Examples
    ///
    /// Joining different tokens by separating them with a comma and a space.
    ///
    /// ```
    /// use rome_formatter::{format, format_args};
    /// use rome_formatter::prelude::*;
    ///
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.join_with(&format_args!(token(","), space_token()))
    ///         .entry(&token("1"))
    ///         .entry(&token("2"))
    ///         .entry(&token("3"))
    ///         .entry(&token("4"))
    ///         .finish()
    /// })]).unwrap();
    ///
    /// assert_eq!(
    ///     "1, 2, 3, 4",
    ///     formatted.print().as_code()
    /// );
    /// ```
    pub fn join_with<'a, Joiner>(
        &'a mut self,
        joiner: Joiner,
    ) -> JoinBuilder<'a, 'buf, Joiner, Context>
    where
        Joiner: Format<Context>,
    {
        JoinBuilder::with_separator(self, joiner)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by a space, soft
    /// line break or empty line depending on the input file.
    ///
    /// This functions inspects the input source and separates consecutive elements with either
    /// a [crate::soft_line_break_or_space] or [crate::empty_line] depending on how many line breaks were
    /// separating the elements in the original file.
    pub fn join_nodes_with_soft_line<'a>(
        &'a mut self,
    ) -> JoinNodesBuilder<'a, 'buf, Line, Context> {
        JoinNodesBuilder::new(soft_line_break_or_space(), self)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by one or more
    /// line breaks depending on the input file.
    ///
    /// This functions inspects the input source and separates consecutive elements with either
    /// a [crate::hard_line_break] or [crate::empty_line] depending on how many line breaks were separating the
    /// elements in the original file.
    pub fn join_nodes_with_hardline<'a>(&'a mut self) -> JoinNodesBuilder<'a, 'buf, Line, Context> {
        JoinNodesBuilder::new(hard_line_break(), self)
    }

    /// Concatenates a list of [crate::Format] objects with spaces and line breaks to fit
    /// them on as few lines as possible. Each element introduces a conceptual group. The printer
    /// first tries to print the item in flat mode but then prints it in expanded mode if it doesn't fit.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use rome_formatter::prelude::*;
    /// use rome_formatter::{format, format_args};
    ///
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.fill(soft_line_break_or_space())
    ///         .entry(&token("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
    ///         .entry(&token("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"))
    ///         .entry(&token("cccccccccccccccccccccccccccccc"))
    ///         .entry(&token("dddddddddddddddddddddddddddddd"))
    ///         .finish()
    /// })]).unwrap();
    ///
    /// assert_eq!(
    ///     "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\ncccccccccccccccccccccccccccccc dddddddddddddddddddddddddddddd",
    ///     formatted.print().as_code()
    /// )
    /// ```
    ///
    /// ```rust
    /// use rome_formatter::prelude::*;
    /// use rome_formatter::{format, format_args};
    ///
    /// let entries = vec![
    ///     token("<b>Important: </b>"),
    ///     token("Please do not commit memory bugs such as segfaults, buffer overflows, etc. otherwise you "),
    ///     token("<em>will</em>"),
    ///     token(" be reprimanded")
    /// ];
    ///
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     f.fill(soft_line_break()).entries(entries.iter()).finish()
    /// })]).unwrap();
    ///
    /// assert_eq!(
    ///     &std::format!("<b>Important: </b>\nPlease do not commit memory bugs such as segfaults, buffer overflows, etc. otherwise you \n<em>will</em> be reprimanded"),
    ///     formatted.print().as_code()
    /// )
    /// ```
    pub fn fill<'a, Separator>(&'a mut self, separator: Separator) -> FillBuilder<'a, 'buf, Context>
    where
        Separator: Format<Context>,
    {
        FillBuilder::new(self, separator)
    }

    /// Formats `content` into an interned element without writing it to the formatter's buffer.
    pub fn intern(&mut self, content: &dyn Format<Context>) -> FormatResult<Interned> {
        let mut buffer = VecBuffer::new(self.state_mut());

        crate::write!(&mut buffer, [content])?;

        Ok(buffer.into_element().intern())
    }
}

impl<Context> Formatter<'_, Context> {
    /// Take a snapshot of the state of the formatter
    #[inline]
    pub fn state_snapshot(&self) -> FormatterSnapshot {
        FormatterSnapshot {
            buffer: self.buffer.snapshot(),
            state: self.state().snapshot(),
        }
    }

    #[inline]
    /// Restore the state of the formatter to a previous snapshot
    pub fn restore_state_snapshot(&mut self, snapshot: FormatterSnapshot) {
        self.state_mut().restore_snapshot(snapshot.state);
        self.buffer.restore_snapshot(snapshot.buffer);
    }
}

impl<Context> Buffer for Formatter<'_, Context> {
    type Context = Context;

    fn write_element(&mut self, element: FormatElement) -> FormatResult<()> {
        self.buffer.write_element(element)
    }

    fn write_fmt(&mut self, arguments: Arguments<Self::Context>) -> FormatResult<()> {
        for argument in arguments.items() {
            argument.format(self)?;
        }
        Ok(())
    }

    fn state(&self) -> &FormatState<Self::Context> {
        self.buffer.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<Self::Context> {
        self.buffer.state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        self.buffer.snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        self.buffer.restore_snapshot(snapshot)
    }
}

/// Snapshot of the formatter state  used to handle backtracking if
/// errors are encountered in the formatting process and the formatter
/// has to fallback to printing raw tokens
///
/// In practice this only saves the set of printed tokens in debug
/// mode and compiled to nothing in release mode
pub struct FormatterSnapshot {
    buffer: BufferSnapshot,
    state: FormatStateSnapshot,
}
