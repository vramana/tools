use crate::prelude::*;

use crate::FormatNodeFields;
use rome_js_syntax::JsAnyExpression::JsIdentifierExpression;
use rome_js_syntax::{JsAnyExpression, JsTemplateFields};
use rome_js_syntax::{JsAnyTemplateElement, JsTemplate};

impl FormatNodeFields<JsTemplate> for FormatNodeRule<JsTemplate> {
    fn format_fields(
        node: &JsTemplate,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let JsTemplateFields {
            tag,
            type_arguments,
            l_tick_token,
            elements,
            r_tick_token,
        } = node.as_fields();

        let l_tick = l_tick_token.format();
        let r_tick = r_tick_token.format();

        formatted![
            formatter,
            [
                tag.format(),
                type_arguments.format(),
                line_suffix_boundary(),
                l_tick,
                concat_elements(formatter.format_all(elements.iter().formatted())?),
                r_tick
            ]
        ]
    }
}

/// A simple template literal contains expressions with only
fn is_simple_template_literal(literal: &JsTemplate) -> FormatResult<bool> {
    let elements = literal.elements();
    if elements.is_empty() {
        return Ok(false);
    }

    let mut is_simple = false;

    for element in elements {
        if let JsAnyTemplateElement::JsTemplateElement(element) = element {
            if element.syntax().has_comments_direct() {
                return Ok(false);
            }

            let expr = element.expression()?;
            if matches!(
                expr,
                JsAnyExpression::JsIdentifierExpression(_) | JsAnyExpression::JsThisExpression(_)
            ) {
                is_simple = true;
            }

            let mut head = expr;
            while let JsAnyExpression::JsStaticMemberExpression(member_expr) = head {
                head = member_expr.object()?;
            }
        }
    }

    Ok(is_simple)
}
