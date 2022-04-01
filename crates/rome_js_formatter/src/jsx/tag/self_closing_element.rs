use crate::formatter_traits::{FormatOptionalTokenAndNode, FormatTokenAndNode};
use crate::{
    format_elements, space_token, FormatElement, FormatResult, Formatter, ToFormatElement,
};
use rome_js_syntax::{JsxSelfClosingElement, JsxSelfClosingElementFields};

impl ToFormatElement for JsxSelfClosingElement {
    fn to_format_element(&self, formatter: &Formatter) -> FormatResult<FormatElement> {
        let JsxSelfClosingElementFields {
            l_angle_token,
            name,
            attributes,
            type_arguments,
            slash_token,
            r_angle_token,
        } = self.as_fields();

        Ok(format_elements![
            l_angle_token.format(formatter)?,
            name.format(formatter)?,
            type_arguments.format_or_empty(formatter)?,
            space_token(),
            attributes.format(formatter)?,
            space_token(),
            slash_token.format(formatter)?,
            r_angle_token.format(formatter)?
        ])
    }
}
