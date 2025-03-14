use rome_js_syntax::kind::JsSyntaxKind;
use rome_js_syntax::{JsLanguage, JsxAnyChild, JsxChildList};
use rome_rowan::{AstNodeList, SyntaxNode};

/// Checks if the children of an element contain meaningful text. See [is_meaningful_jsx_text] for
/// definition of meaningful JSX text.
pub fn contains_meaningful_jsx_text(children: &JsxChildList) -> bool {
    children.iter().any(|child| {
        if let JsxAnyChild::JsxText(jsx_text) = child {
            if let Ok(token) = jsx_text.value_token() {
                if is_meaningful_jsx_text(token.text()) {
                    return true;
                }
            }
        }

        false
    })
}

pub static JSX_WHITESPACE_CHARS: [char; 4] = [' ', '\n', '\t', '\r'];

/// Meaningful JSX text is defined to be text that has either non-whitespace
/// characters, or does not contain a newline. Whitespace is defined as ASCII
/// whitespace.
///
/// ```
/// use rome_js_formatter::utils::jsx_utils::is_meaningful_jsx_text;
///
/// assert_eq!(is_meaningful_jsx_text("     \t\r   "), true);
/// assert_eq!(is_meaningful_jsx_text("     \n\r   "), false);
/// assert_eq!(is_meaningful_jsx_text("  Alien   "), true);
/// assert_eq!(is_meaningful_jsx_text("\n  Alien   "), true);
/// assert_eq!(is_meaningful_jsx_text("  Alien   \n"), true);
/// assert_eq!(is_meaningful_jsx_text(""), true);
/// ```
pub fn is_meaningful_jsx_text(text: &str) -> bool {
    let mut has_newline = false;
    for c in text.chars() {
        // If there is a non-whitespace character
        if !JSX_WHITESPACE_CHARS.contains(&c) {
            return true;
        } else if c == '\n' {
            has_newline = true;
        }
    }

    !has_newline
}

/// Indicates that an element should always be wrapped in parentheses, should be wrapped
/// only when it's line broken, or should not be wrapped at all.
pub enum WrapState {
    /// For a JSX element that is never wrapped in parentheses.
    /// For instance, a JSX element that is another element's attribute
    /// should never be wrapped:
    /// ```jsx
    ///  <Route path="/" component={<HomePage />} />
    /// ```
    NoWrap,
    /// For a JSX element that must be wrapped in parentheses when line broken.
    /// For instance, a JSX element nested in a let binding is wrapped on line break:
    /// ```jsx
    ///  let component = <div> La Haine dir. Mathieu Kassovitz </div>;
    ///
    ///  let component = (
    ///   <div> Uncle Boonmee Who Can Recall His Past Lives dir. Apichatpong Weerasethakul </div>
    ///  );
    /// ```
    WrapOnBreak,
    /// For a JSX element that must always be wrapped in parentheses.
    /// For instance, a JSX element inside a static member expression
    /// should always be wrapped:
    /// ```jsx
    /// (<div>Badlands</div>).property
    /// ```
    AlwaysWrap,
}

/// Checks if a JSX Element should be wrapped in parentheses. Returns a [WrapState] which
/// indicates when the element should be wrapped in parentheses.
pub fn get_wrap_state(node: &SyntaxNode<JsLanguage>) -> WrapState {
    // We skip the first item because the first item in ancestors is the node itself, i.e.
    // the JSX Element in this case.
    let mut ancestors = node.ancestors().skip(1);

    ancestors
        .next()
        .map(|parent| {
            let parent_kind = parent.kind();
            if parent_kind == JsSyntaxKind::JS_PARENTHESIZED_EXPRESSION {
                return get_wrap_state(&parent);
            }

            match parent_kind {
                JsSyntaxKind::JS_ARRAY_EXPRESSION
                | JsSyntaxKind::JSX_ATTRIBUTE
                | JsSyntaxKind::JSX_ELEMENT
                | JsSyntaxKind::JSX_EXPRESSION_CHILD
                | JsSyntaxKind::JSX_FRAGMENT
                | JsSyntaxKind::JS_EXPRESSION_STATEMENT
                | JsSyntaxKind::JS_CALL_ARGUMENT_LIST => WrapState::NoWrap,
                JsSyntaxKind::JS_STATIC_MEMBER_EXPRESSION
                | JsSyntaxKind::JS_COMPUTED_MEMBER_EXPRESSION => WrapState::AlwaysWrap,
                _ => WrapState::WrapOnBreak,
            }
        })
        .unwrap_or(WrapState::NoWrap)
}

/// This is a very special situation where we're returning a JsxElement
/// from an arrow function that's passed as an argument to a function,
/// which is itself inside a JSX expression child.
///
/// If you're wondering why this is the only other case, it's because
/// Prettier defines it to be that way.
///
/// ```jsx
///  let bar = <div>
///    {foo(() => <div> the quick brown fox jumps over the lazy dog </div>)}
///  </div>;
/// ```
pub fn is_jsx_inside_arrow_function_inside_call_inside_expression_child(
    node: &SyntaxNode<JsLanguage>,
) -> bool {
    // We skip the first item because the first item in ancestors is the node itself, i.e.
    // the JSX Element in this case.
    let mut ancestors = node.ancestors().skip(2).peekable();

    // This matching should work with or without parentheses around the JSX element
    // therefore we ignore parenthesized expressions.
    if ancestors
        .peek()
        .map(|a| a.kind() == JsSyntaxKind::JS_PARENTHESIZED_EXPRESSION)
        .unwrap_or(false)
    {
        ancestors.next();
    }

    let required_ancestors = [
        JsSyntaxKind::JS_ARROW_FUNCTION_EXPRESSION,
        JsSyntaxKind::JS_CALL_ARGUMENT_LIST,
        JsSyntaxKind::JS_CALL_ARGUMENTS,
        JsSyntaxKind::JS_CALL_EXPRESSION,
        JsSyntaxKind::JSX_EXPRESSION_CHILD,
    ];

    for required_ancestor in required_ancestors {
        let is_required_ancestor = ancestors
            .next()
            .map(|ancestor| ancestor.kind() == required_ancestor)
            .unwrap_or(false);
        if !is_required_ancestor {
            return false;
        }
    }

    true
}
