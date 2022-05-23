use crate::prelude::*;
use crate::utils::FormatTypeMemberSeparator;
use crate::FormatNodeFields;
use rome_js_syntax::{TsMethodSignatureTypeMember, TsMethodSignatureTypeMemberFields};

impl FormatNodeFields<TsMethodSignatureTypeMember> for FormatNodeRule<TsMethodSignatureTypeMember> {
    fn format_fields(
        node: &TsMethodSignatureTypeMember,
        formatter: &Formatter<JsFormatOptions>,
    ) -> FormatResult<FormatElement> {
        let TsMethodSignatureTypeMemberFields {
            name,
            optional_token,
            type_parameters,
            parameters,
            return_type_annotation,
            separator_token,
        } = node.as_fields();

        formatted![
            formatter,
            [
                name.format(),
                optional_token.format(),
                type_parameters.format(),
                parameters.format(),
                return_type_annotation.format(),
                FormatTypeMemberSeparator::new(separator_token)
            ]
        ]
    }
}
