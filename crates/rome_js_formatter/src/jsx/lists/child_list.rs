use crate::prelude::*;
use crate::utils::jsx_utils::contains_meaningful_jsx_text;
use crate::JsFormatter;
use rome_formatter::write;
use rome_js_syntax::JsxChildList;

#[derive(Debug, Clone, Default)]
pub struct FormatJsxChildList;

impl FormatRule<JsxChildList> for FormatJsxChildList {
    type Context = JsFormatContext;

    fn fmt(&self, node: &JsxChildList, f: &mut JsFormatter) -> FormatResult<()> {
        let mut forced_break = false;
        let children: Vec<_> = node
            .iter()
            .formatted()
            .into_iter()
            .map(|formatted| formatted.memoized())
            .collect();

        for element in &children {
            let mut null_buffer = f.inspect_null();
            let mut b = null_buffer.inspect_will_break();
            write!(b, [element])?;
            if b.will_break() {
                forced_break = true;
                break;
            }
        }

        let children = format_with(move |f| {
            if contains_meaningful_jsx_text(node) {
                f.fill(soft_line_break())
                    .flatten_entries(children.iter())
                    .finish()
            } else {
                f.join_with(hard_line_break())
                    .entries(children.iter())
                    .finish()
            }
        });

        let multiline = format_with(|f| write!(f, [block_indent(&children)]));

        if forced_break {
            return write!(f, [multiline]);
        }
        write!(f, [&best_fitting![group_elements(&children), multiline]])
    }
}
