use chrono::NaiveDate;
use chumsky::{label::LabelError, prelude::*, util::Maybe};
use std::fmt::Write;

pub fn parse_date<'a>() -> impl Parser<'a, &'a str, NaiveDate, extra::Err<Rich<'a, char>>> {
    let date_separator = one_of("-/");
    let year = just('-')
        .or_not()
        .then(digits::<4>())
        .labelled("four digit year");
    let month = digits::<2>().labelled("two digit month");
    let day = digits::<2>().labelled("two digit day");

    year.then_ignore(date_separator)
        .then(month)
        .then_ignore(date_separator)
        .then(day)
        .try_map_with(|(((neg, year), month), day), extra| {
            let year = i32::try_from(year).unwrap();
            let year: i32 = if neg.is_some() { -year } else { year };
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
                Rich::custom(
                    extra.span(),
                    format!("{} is not a valid date", extra.slice()),
                )
            })
        })
}

fn digits<'a, const NUM_DIGITS: usize>() -> impl Parser<'a, &'a str, u32, extra::Err<Rich<'a, char>>>
{
    digit()
        .repeated()
        .collect_exactly::<[u8; NUM_DIGITS]>()
        .map(|digits| digits.iter().fold(0, |acc, &d| acc * 10 + u32::from(d)))
}

fn digit<'a>() -> impl Parser<'a, &'a str, u8, extra::Err<Rich<'a, char>>> {
    any().try_map(|c: char, span| {
        let parsed = c.to_digit(10).ok_or_else(|| {
            LabelError::<&'a str, _>::expected_found(["digit"], Some(Maybe::Val(c)), span)
        })?;
        Ok(u8::try_from(parsed).unwrap())
    })
}

pub fn marshal_date(date: &NaiveDate, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "{}", date.format("%Y-%m-%d"))
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;
    use chumsky::error::RichPattern;
    use rstest::rstest;
    use rstest_reuse::*;

    fn error(span: Range<usize>, message: &str) -> Rich<'_, char> {
        Rich::custom(SimpleSpan::from(span), message)
    }

    fn expected_found(
        span: Range<usize>,
        expected: impl IntoIterator<Item = impl Into<RichPattern<'static, char>>>,
        found: impl Into<Option<char>>,
    ) -> Rich<'static, char> {
        LabelError::<&str, _>::expected_found(
            expected,
            found.into().map(Maybe::Val),
            SimpleSpan::from(span),
        )
    }

    #[template]
    #[rstest]
    #[case("2023-01-01", 2023, 1, 1)]
    #[case("2023-12-31", 2023, 12, 31)]
    #[case("2024-02-29", 2024, 2, 29)] // leap year
    #[case("2000-02-29", 2000, 2, 29)] // leap year
    #[case("1900-12-25", 1900, 12, 25)]
    #[case("2099-07-15", 2099, 7, 15)]
    #[case("2023-06-15", 2023, 6, 15)]
    #[case("2023-02-28", 2023, 2, 28)]
    #[case("0023-02-28", 23, 2, 28)] // very early year
    #[case("-5000-02-28", -5000, 2, 28)] // negative year
    fn valid_date_template(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
    }

    #[template]
    #[rstest]
    #[case("2023-13-01", error(0..10, "2023-13-01 is not a valid date"))] // invalid month
    #[case("2023/13/01", error(0..10, "2023/13/01 is not a valid date"))]
    #[case("2023-00-01", error(0..10, "2023-00-01 is not a valid date"))] // invalid month
    #[case("2023/00/01", error(0..10, "2023/00/01 is not a valid date"))]
    #[case("2023-01-32", error(0..10, "2023-01-32 is not a valid date"))] // invalid day
    #[case("2023/01/32", error(0..10, "2023/01/32 is not a valid date"))]
    #[case("2023-01-00", error(0..10, "2023-01-00 is not a valid date"))] // invalid day
    #[case("2023/01/00", error(0..10, "2023/01/00 is not a valid date"))]
    #[case("2023-02-29", error(0..10, "2023-02-29 is not a valid date"))] // not a leap year
    #[case("2023/02/29", error(0..10, "2023/02/29 is not a valid date"))]
    #[case("2023-04-31", error(0..10, "2023-04-31 is not a valid date"))] // April doesn't have 31 days
    #[case("2023/04/31", error(0..10, "2023/04/31 is not a valid date"))]
    #[case("2023-06-31", error(0..10, "2023-06-31 is not a valid date"))] // June doesn't have 31 days
    #[case("2023/06/31", error(0..10, "2023/06/31 is not a valid date"))]
    #[case("2023-09-31", error(0..10, "2023-09-31 is not a valid date"))] // September doesn't have 31 days
    #[case("2023/09/31", error(0..10, "2023/09/31 is not a valid date"))]
    #[case("2023-11-31", error(0..10, "2023-11-31 is not a valid date"))] // November doesn't have 31 days
    #[case("2023/11/31", error(0..10, "2023/11/31 is not a valid date"))]
    fn invalid_date_template(#[case] input: &str, #[case] expected_error: Rich<char>) {}

    #[template]
    #[rstest]
    #[case("23-01-01", expected_found(2..3, ["digit"], '-'))] // wrong year format
    #[case("23/01/01", expected_found(2..3, ["digit"], '/'))]
    #[case("2023-1-01", expected_found(6..7, ["digit"], '-'))] // wrong month format
    #[case("2023/1/01", expected_found(6..7, ["digit"], '/'))]
    #[case("2023-01-1", expected_found(9..9, [RichPattern::Any], None))] // wrong day format
    #[case("2023/01/1", expected_found(9..9, [RichPattern::Any], None))]
    #[case("2023.01.01", expected_found(4..5, ['-', '/'], '.'))] // wrong separator
    #[case("20230101", expected_found(4..5, ['-', '/'], '0'))] // no separators
    #[case("2023-01", expected_found(7..7, ['-', '/'], None))] // missing day
    #[case("2023/01", expected_found(7..7, ['-', '/'], None))]
    #[case("2023", expected_found(4..4, ['-', '/'], None))] // missing month and day
    #[case("01-01-2023", expected_found(2..3, ["digit"], '-'))] // wrong order
    #[case("01/01/2023", expected_found(2..3, ["digit"], '/'))]
    #[case("2023-1-1", expected_found(6..7, ["digit"], '-'))] // single digit month and day
    #[case("2023/1/1", expected_found(6..7, ["digit"], '/'))]
    #[case("", expected_found(0..0, ["four digit year"], None))] // empty string
    #[case("not-a-date", expected_found(0..1, ["four digit year"], 'n'))] // completely invalid
    #[case("not/a/date", expected_found(0..1, ["four digit year"], 'n'))]
    #[case("2023-cd-01", expected_found(5..6, ["two digit month"], 'c'))] // non-numeric month
    #[case("2023/cd/01", expected_found(5..6, ["two digit month"], 'c'))]
    #[case("2023-01-bc", expected_found(8..9,["two digit day"], 'b'))] // non-numeric day
    #[case("2023/01/bc", expected_found(8..9,["two digit day"], 'b'))]
    #[case("abcd-01-01", expected_found(0..1, ["four digit year"], 'a'))] // non-numeric year
    #[case("abcd/01/01", expected_found(0..1, ["four digit year"], 'a'))]
    fn invalid_format_template(#[case] input: &str, #[case] expected_error: Rich<char>) {}

    #[apply(valid_date_template)]
    fn parse_valid(#[case] input: &str, #[case] year: i32, #[case] month: u32, #[case] day: u32) {
        let result = parse_date().parse(input);
        assert!(result.has_output(), "Failed to parse valid date: {}", input);
        let parsed_date = result.into_result().unwrap();
        let expected_date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        assert_eq!(parsed_date, expected_date);
    }

    #[test]
    fn different_separator() {
        let result = parse_date().parse("2020/01/02");
        let parsed_date = result.into_result().unwrap();
        let expected_date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        assert_eq!(parsed_date, expected_date);
    }

    #[apply(invalid_date_template)]
    fn parse_invalid_date(#[case] input: &str, #[case] expected_error: Rich<char>) {
        let result = parse_date().parse(input);
        assert!(
            result.has_errors(),
            "Expected parsing to fail for invalid date: {}",
            input
        );
        let errors = result.into_errors();
        assert_eq!(vec![expected_error], errors);
    }

    #[apply(invalid_format_template)]
    fn parse_invalid_format(#[case] input: &str, #[case] expected_error: Rich<char>) {
        let result = parse_date().parse(input);
        assert!(
            result.has_errors(),
            "Expected parsing to fail for invalid format: {}",
            input
        );
        let errors = result.into_errors();
        assert_eq!(vec![expected_error], errors);
    }

    #[apply(valid_date_template)]
    fn marshal(
        #[case] expected_output: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let mut output = String::new();
        let result = marshal_date(&date, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, expected_output);
    }

    #[apply(valid_date_template)]
    fn marshal_and_parse(
        #[case] _date_str: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let original_date = NaiveDate::from_ymd_opt(year, month, day).unwrap();

        // Marshal to string
        let mut marshalled = String::new();
        marshal_date(&original_date, &mut marshalled).unwrap();

        // Parse back from string
        let result = parse_date().parse(&marshalled);
        assert!(result.has_output());
        let parsed_date = result.into_result().unwrap();

        // Should be equal
        assert_eq!(original_date, parsed_date);
    }

    #[test]
    fn parse_with_partial_input() {
        // Test that parser correctly handles parsing date when followed by other content
        let parser = parse_date()
            .then_ignore(just(' '))
            .then(text::ascii::keyword("transaction"));
        let input = "2023-06-15 transaction";
        let result = parser.parse(input);
        assert!(result.has_output());
        let (date, _) = result.into_result().unwrap();
        let expected_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        assert_eq!(date, expected_date);
    }

    #[test]
    fn parse_stops_at_non_digit() {
        // Test that the parser stops correctly when encountering non-date characters
        let input = "2023-06-15T10:30:00"; // ISO datetime format
        let result = parse_date().parse(input);
        assert!(result.has_errors()); // Should fail because it expects exactly YYYY-MM-DD format
    }

    #[test]
    fn parse_error_message() {
        // Test specific error messages for invalid dates
        let input = "2023-02-30"; // February 30th doesn't exist
        let result = parse_date().parse(input);
        assert!(result.has_errors());
        let errors = result.into_errors();
        assert!(errors.iter().any(|e| e.to_string().contains("valid date")));
    }

    #[test]
    fn marshal_various_dates() {
        // Test marshaling of specific interesting dates
        let test_cases = vec![
            (NaiveDate::from_ymd_opt(1, 1, 1).unwrap(), "0001-01-01"),
            (NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(), "9999-12-31"),
            (NaiveDate::from_ymd_opt(2000, 2, 29).unwrap(), "2000-02-29"), // leap year
        ];

        for (date, expected) in test_cases {
            let mut output = String::new();
            marshal_date(&date, &mut output).unwrap();
            assert_eq!(output, expected);
        }
    }
}
