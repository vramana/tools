use rome_diagnostics::{file::SimpleFiles, termcolor, Emitter};
use rome_js_formatter::context::JsFormatContext;
use rome_js_formatter::format_node;
use rome_js_parser::parse;
use rome_js_syntax::{JsSyntaxNode, SourceType};

pub struct CheckReformatParams<'a> {
    pub root: &'a JsSyntaxNode,
    pub text: &'a str,
    pub source_type: SourceType,
    pub file_name: &'a str,
    pub format_context: JsFormatContext,
}

/// Perform a second pass of formatting on a file, printing a diff if the
/// output doesn't match the input
pub fn check_reformat(params: CheckReformatParams) {
    let CheckReformatParams {
        root,
        text,
        source_type,
        file_name,
        format_context: context,
    } = params;

    let re_parse = parse(text, 0, source_type);

    // Panic if the result from the formatter has syntax errors
    if re_parse.has_errors() {
        let mut files = SimpleFiles::new();
        files.add(file_name.into(), text.into());

        let mut buffer = termcolor::Buffer::ansi();
        let mut emitter = Emitter::new(&files);

        for error in re_parse.diagnostics() {
            emitter
                .emit_with_writer(error, &mut buffer)
                .expect("failed to emit diagnostic");
        }

        panic!(
            "formatter output had syntax errors where input had none:\n{}",
            std::str::from_utf8(buffer.as_slice()).expect("non utf8 in error buffer")
        )
    }

    let formatted = format_node(context.clone(), &re_parse.syntax()).unwrap();
    let printed = formatted.print();

    if text != printed.as_code() {
        let input_format_element = format_node(context, root).unwrap();

        // Print a diff of the Formatter IR emitted for the input and the output
        let diff =
            similar_asserts::Diff::from_debug(&input_format_element, &formatted, "input", "output");

        println!("{diff}");

        similar_asserts::assert_str_eq!(text, printed.as_code());
    }
}
