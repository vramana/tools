use crate::prelude::*;

use crate::utils::FormatStringLiteralToken;
use crate::FormatNodeFields;
use rome_js_syntax::JsModuleSource;
use rome_js_syntax::JsModuleSourceFields;

impl FormatNodeFields<JsModuleSource> for FormatNodeRule<JsModuleSource> {
    fn format_fields(
        node: &JsModuleSource,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let JsModuleSourceFields { value_token } = node.as_fields();

        formatted![formatter, [FormatStringLiteralToken::new(value_token?)]]
    }
}
