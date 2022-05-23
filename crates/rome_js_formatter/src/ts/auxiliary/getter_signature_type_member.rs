use crate::prelude::*;
use crate::utils::FormatTypeMemberSeparator;
use crate::FormatNodeFields;
use rome_js_syntax::{TsGetterSignatureTypeMember, TsGetterSignatureTypeMemberFields};

impl FormatNodeFields<TsGetterSignatureTypeMember> for FormatNodeRule<TsGetterSignatureTypeMember> {
    fn format_fields(
        node: &TsGetterSignatureTypeMember,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let TsGetterSignatureTypeMemberFields {
            get_token,
            name,
            l_paren_token,
            r_paren_token,
            type_annotation,
            separator_token,
        } = node.as_fields();

        formatted![
            formatter,
            [
                get_token.format(),
                space_token(),
                name.format(),
                l_paren_token.format(),
                r_paren_token.format(),
                type_annotation.format(),
                FormatTypeMemberSeparator::new(separator_token)
            ]
        ]
    }
}
