use crate::prelude::*;

use crate::FormatNodeFields;
use rome_js_syntax::{JsAnyExpression, JsTemplateFields};
use rome_js_syntax::{JsAnyTemplateElement, JsTemplate};

impl FormatNodeFields<JsTemplate> for FormatNodeRule<JsTemplate> {
    fn format_fields(
        node: &JsTemplate,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        println!("IS_SIMPLE: {}", is_simple_template_literal(node)?);

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

    for element in elements {
        if element.syntax().has_comments_descendants() {
            println!("1");
            return Ok(false);
        }

        if let JsAnyTemplateElement::JsTemplateElement(element) = element {
            let expression = element.expression()?;
            if matches!(
                expression,
                JsAnyExpression::JsIdentifierExpression(_) | JsAnyExpression::JsThisExpression(_)
            ) {
                continue;
            }

            let mut head = expression;
            loop {
                match head {
                    JsAnyExpression::JsStaticMemberExpression(static_member_expression) => {
                        head = static_member_expression.object()?;
                    }
                    JsAnyExpression::JsComputedMemberExpression(computed_member_expression) => {
                        let member = computed_member_expression.member()?;
                        if !matches!(member, JsAnyExpression::JsAnyLiteralExpression(_)) {
                            println!("2");
                            return Ok(false);
                        }
                        head = computed_member_expression.object()?;
                    }
                    _ => {
                        break;
                    }
                }
            }

            if matches!(
                head,
                JsAnyExpression::JsAnyLiteralExpression(_) | JsAnyExpression::JsThisExpression(_)
            ) {
                continue;
            }
        }
    }

    Ok(true)
}
