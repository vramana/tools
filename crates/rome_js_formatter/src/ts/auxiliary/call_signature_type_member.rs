use crate::prelude::*;
use crate::utils::FormatTypeMemberSeparator;
use crate::FormatNodeFields;
use rome_js_syntax::{TsCallSignatureTypeMember, TsCallSignatureTypeMemberFields};

impl FormatNodeFields<TsCallSignatureTypeMember> for FormatNodeRule<TsCallSignatureTypeMember> {
    fn format_fields(
        node: &TsCallSignatureTypeMember,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let TsCallSignatureTypeMemberFields {
            type_parameters,
            parameters,
            return_type_annotation,
            separator_token,
        } = node.as_fields();

        formatted![
            formatter,
            [
                type_parameters.format(),
                parameters.format(),
                return_type_annotation.format(),
                FormatTypeMemberSeparator::new(separator_token)
            ]
        ]
    }
}
