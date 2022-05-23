use crate::prelude::*;

use crate::FormatNodeFields;
use rome_js_syntax::JsEmptyClassMember;
use rome_js_syntax::JsEmptyClassMemberFields;

impl FormatNodeFields<JsEmptyClassMember> for FormatNodeRule<JsEmptyClassMember> {
    fn format_fields(
        node: &JsEmptyClassMember,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let JsEmptyClassMemberFields { semicolon_token } = node.as_fields();

        formatted![
            formatter,
            [formatter.replaced_token(&semicolon_token?, empty_element())]
        ]
    }
}
