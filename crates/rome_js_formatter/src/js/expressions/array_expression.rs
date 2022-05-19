use crate::prelude::*;

use crate::generated::FormatJsArrayElementList;
use crate::{print_dangling_comments, FormatNodeFields};
use rome_js_syntax::JsArrayExpression;
use rome_js_syntax::JsArrayExpressionFields;

impl FormatNodeFields<JsArrayExpression> for FormatNodeRule<JsArrayExpression> {
    fn format_fields(
        node: &JsArrayExpression,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let JsArrayExpressionFields {
            l_brack_token,
            elements,
            r_brack_token,
        } = node.as_fields();

        let group_id = formatter.group_id("array");

        let elements = format_elements![
            dbg!(print_dangling_comments(&elements.syntax(), formatter)?),
            FormatJsArrayElementList::format_with_group_id(&elements, formatter, Some(group_id))?
        ];

        formatter
            .delimited(&l_brack_token?, elements, &r_brack_token?)
            .soft_block_indent_with_group_id(Some(group_id))
            .finish()
    }
}
