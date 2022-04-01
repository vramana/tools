use crate::formatter_traits::FormatTokenAndNode;
use crate::{format_elements, FormatElement, FormatResult, Formatter, ToFormatElement};
use rome_js_syntax::{JsxClosingFragment, JsxClosingFragmentFields};

impl ToFormatElement for JsxClosingFragment {
    fn to_format_element(&self, formatter: &Formatter) -> FormatResult<FormatElement> {
        let JsxClosingFragmentFields {
            l_angle_token,
            slash_token,
            r_angle_token,
        } = self.as_fields();

        Ok(format_elements![
            l_angle_token.format(formatter)?,
            slash_token.format(formatter)?,
            r_angle_token.format(formatter)?
        ])
    }
}
