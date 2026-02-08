// src/diag.rs

use ariadne::{Color, Label, Report, ReportKind, Source};

use crate::analysis::AError;
use crate::parser::ParseDiag;

pub fn render_parse_error(src: &str, file: &str, d: &ParseDiag) {
    let report = Report::build(ReportKind::Error, (file, d.span.start..d.span.end))
        .with_code("A_PARSE")
        .with_message(&d.message)
        .with_label(
            Label::new((file, d.span.start..d.span.end))
                .with_message("I got confused parsing here")
                .with_color(Color::Red),
        )
        .with_help(
            r#"Example:
Func main() {
    x = 1
    y = x + 2
    Print(y)
}"#,
        )
        .with_note(
            "The parser couldn't match the code here to A's grammar.\n\
Double-check braces `{}` and parentheses `()` are balanced, and statements are valid.",
        )
        .finish();

    report.print((file, Source::from(src))).unwrap();
}

pub fn render_lesson_error(src: &str, file: &str, e: &AError) {
    let mut rep = Report::build(ReportKind::Error, (file, e.span.start..e.span.end))
        .with_code(&e.code)
        .with_message(&e.title)
        .with_label(
            Label::new((file, e.span.start..e.span.end))
                .with_message("This is where the problem shows up")
                .with_color(Color::Red),
        )
        .with_note(format!("Why: {}", e.mental_model));

    for (i, h) in e.help.iter().enumerate() {
        rep = rep.with_help(format!("Help {}: {}", i + 1, h));
    }

    rep = rep.with_help(format!("Example fix:\n{}", e.example));

    if let Some(backend) = &e.backend {
        rep = rep.with_note(format!("Backend: {}", backend));
    }

    rep.finish().print((file, Source::from(src))).unwrap();
}
