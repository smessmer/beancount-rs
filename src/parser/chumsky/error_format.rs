use ariadne::{Report, ReportKind};
use chumsky::{ParseResult, error::Rich};

pub trait ParseResultExt {
    fn get_formatted_errors(&self) -> Vec<Report<'_>>;
}

impl<T> ParseResultExt for ParseResult<T, Rich<'_, char>> {
    fn get_formatted_errors(&self) -> Vec<Report<'_>> {
        self.errors()
            .map(|e| crate::parser::chumsky::error_format::format_error(&e))
            .collect()
    }
}

pub fn format_error<'a>(error: &Rich<'a, char>) -> Report<'a> {
    let mut report = Report::build(ReportKind::Error, error.span().into_range())
        .with_message(error.to_string())
        .with_label(
            ariadne::Label::new(error.span().into_range())
                .with_message(error.reason().to_string())
                .with_color(ariadne::Color::Red),
        );
    for (expected_pattern, span) in error.contexts() {
        report = report.with_label(
            ariadne::Label::new(span.into_range())
                .with_message(expected_pattern.to_string())
                .with_color(ariadne::Color::Yellow),
        );
    }
    report.finish()
}
