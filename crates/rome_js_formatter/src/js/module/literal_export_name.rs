use crate::prelude::*;
use crate::utils::FormatStringLiteralToken;
use crate::FormatNodeFields;
use rome_js_syntax::JsLiteralExportName;
use rome_js_syntax::JsLiteralExportNameFields;

impl FormatNodeFields<JsLiteralExportName> for FormatNodeRule<JsLiteralExportName> {
    fn format_fields(
        node: &JsLiteralExportName,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let JsLiteralExportNameFields { value } = node.as_fields();

        formatted![formatter, [FormatStringLiteralToken::new(value?)]]
    }
}
