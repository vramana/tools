use crate::formatter_traits::FormatTokenAndNode;
use crate::{
    block_indent, format_elements, group_elements, FormatElement, FormatResult, Formatter,
    ToFormatElement,
};
use rome_js_syntax::{JsxFragment, JsxFragmentFields};

impl ToFormatElement for JsxFragment {
    fn to_format_element(&self, formatter: &Formatter) -> FormatResult<FormatElement> {
        let JsxFragmentFields {
            opening_fragment,
            children,
            closing_fragment,
        } = self.as_fields();
        Ok(group_elements(format_elements![
            opening_fragment.format(formatter)?,
            block_indent(children.format(formatter)?),
            closing_fragment.format(formatter)?,
        ]))
    }
}
