use crate::formatter_traits::FormatTokenAndNode;
use crate::{format_elements, FormatElement, FormatResult, Formatter, ToFormatElement};
use rome_js_syntax::{JsxOpeningFragment, JsxOpeningFragmentFields};

impl ToFormatElement for JsxOpeningFragment {
    fn to_format_element(&self, formatter: &Formatter) -> FormatResult<FormatElement> {
        let JsxOpeningFragmentFields {
            l_angle_token,
            r_angle_token,
        } = self.as_fields();

        Ok(format_elements![
            l_angle_token.format(formatter)?,
            r_angle_token.format(formatter)?
        ])
    }
}
