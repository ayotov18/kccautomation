use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, space0};
use nom::combinator::{opt, value};
use nom::sequence::{delimited, preceded};

use super::types::{DimensionParseError, DimensionPrefix, ParsedDimension};
use crate::geometry::model::Tolerance;

type IResult<'a, O> = nom::IResult<&'a str, O>;

/// Parse dimension text into a structured representation.
pub fn parse_dimension_text(input: &str) -> Result<ParsedDimension, DimensionParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(DimensionParseError::Empty);
    }

    // Try reference dimension first: (25.00)
    if let Ok((_, parsed)) = parse_reference.parse(input) {
        return Ok(parsed);
    }

    // Try thread: M8x1.25
    if let Ok((_, parsed)) = parse_thread_dim.parse(input) {
        return Ok(parsed);
    }

    // Try full dimension with optional prefix, value, tolerance
    if let Ok((_, parsed)) = parse_full_dimension.parse(input) {
        return Ok(parsed);
    }

    // Fallback: try to extract just a number
    if let Ok(val) = extract_number(input) {
        return Ok(ParsedDimension {
            nominal: val,
            tolerance: None,
            prefix: DimensionPrefix::None,
            count: None,
            is_reference: false,
            is_basic: false,
            raw_text: input.to_string(),
        });
    }

    Err(DimensionParseError::ParseFailed(input.to_string()))
}

fn parse_float(input: &str) -> IResult<'_, f64> {
    let (input, neg) = opt(char('-')).parse(input)?;
    let (input, digits) = take_while1(|c: char| c.is_ascii_digit() || c == '.').parse(input)?;
    let val: f64 = digits.parse().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
    })?;
    Ok((input, if neg.is_some() { -val } else { val }))
}

fn parse_count_prefix(input: &str) -> IResult<'_, usize> {
    let (input, count) = take_while1(|c: char| c.is_ascii_digit()).parse(input)?;
    let (input, _) = (space0, alt((tag("X"), tag("x"))), space0).parse(input)?;
    let n: usize = count.parse().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    Ok((input, n))
}

fn parse_diameter_prefix(input: &str) -> IResult<'_, DimensionPrefix> {
    alt((
        value(DimensionPrefix::Diameter, tag("\u{2300}")), // ⌀
        value(DimensionPrefix::Diameter, tag("%%c")),      // AutoCAD %%c
        value(DimensionPrefix::Diameter, tag("%%C")),
        value(DimensionPrefix::Diameter, tag("\u{00D8}")), // Ø
    ))
    .parse(input)
}

fn parse_radius_prefix(input: &str) -> IResult<'_, DimensionPrefix> {
    value(DimensionPrefix::Radius, alt((tag("R"), tag("r")))).parse(input)
}

fn parse_symmetric_tolerance(input: &str) -> IResult<'_, Tolerance> {
    let (input, _) = space0.parse(input)?;
    let (input, _) = alt((tag("\u{00B1}"), tag("+-"), tag("+/-"))).parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, val) = parse_float.parse(input)?;
    Ok((input, Tolerance::symmetric(val)))
}

fn parse_asymmetric_tolerance(input: &str) -> IResult<'_, Tolerance> {
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('+').parse(input)?;
    let (input, upper) = parse_float.parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char('/').parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, lower) = parse_float.parse(input)?;
    Ok((input, Tolerance::asymmetric(upper, lower)))
}

fn parse_limit_dimension(input: &str) -> IResult<'_, (f64, Tolerance)> {
    let (input, upper) = parse_float.parse(input)?;
    let (input, _) = (space0, char('/'), space0).parse(input)?;
    let (input, lower) = parse_float.parse(input)?;
    let nominal = (upper + lower) / 2.0;
    let tol = Tolerance::asymmetric(upper - nominal, lower - nominal);
    Ok((input, (nominal, tol)))
}

fn parse_reference(input: &str) -> IResult<'_, ParsedDimension> {
    let (input, inner) = delimited(char('('), parse_float, char(')')).parse(input)?;
    Ok((
        input,
        ParsedDimension {
            nominal: inner,
            tolerance: None,
            prefix: DimensionPrefix::None,
            count: None,
            is_reference: true,
            is_basic: false,
            raw_text: format!("({inner})"),
        },
    ))
}

fn parse_thread_dim(input: &str) -> IResult<'_, ParsedDimension> {
    let (input, _) = alt((tag("M"), tag("m"))).parse(input)?;
    let (input, diameter) = parse_float.parse(input)?;
    let (input, pitch) = opt(preceded(alt((char('x'), char('X'))), parse_float)).parse(input)?;

    let designation = if let Some(p) = pitch {
        format!("M{diameter}x{p}")
    } else {
        format!("M{diameter}")
    };

    Ok((
        input,
        ParsedDimension {
            nominal: diameter,
            tolerance: None,
            prefix: DimensionPrefix::Thread(designation.clone()),
            count: None,
            is_reference: false,
            is_basic: false,
            raw_text: designation,
        },
    ))
}

fn parse_full_dimension(input: &str) -> IResult<'_, ParsedDimension> {
    // Optional count prefix: 3X
    let (input, count) = opt(parse_count_prefix).parse(input)?;
    let (input, _) = space0.parse(input)?;

    // Optional type prefix: ⌀, R
    let (input, prefix) = opt(alt((parse_diameter_prefix, parse_radius_prefix))).parse(input)?;
    let (input, _) = space0.parse(input)?;

    // Try limit dimension first
    if let Ok((input2, (nominal, tol))) = parse_limit_dimension.parse(input) {
        return Ok((
            input2,
            ParsedDimension {
                nominal,
                tolerance: Some(tol),
                prefix: prefix.unwrap_or(DimensionPrefix::None),
                count,
                is_reference: false,
                is_basic: false,
                raw_text: input.to_string(),
            },
        ));
    }

    // Nominal value
    let (input, nominal) = parse_float.parse(input)?;

    // Optional degree symbol for angular
    let (input, is_angular) = opt(alt((tag("\u{00B0}"), tag("deg")))).parse(input)?;

    let prefix = if is_angular.is_some() {
        DimensionPrefix::Angular
    } else {
        prefix.unwrap_or(DimensionPrefix::None)
    };

    // Optional tolerance
    let (input, tolerance) =
        opt(alt((parse_symmetric_tolerance, parse_asymmetric_tolerance))).parse(input)?;

    Ok((
        input,
        ParsedDimension {
            nominal,
            tolerance,
            prefix,
            count,
            is_reference: false,
            is_basic: false,
            raw_text: input.to_string(),
        },
    ))
}

fn extract_number(input: &str) -> Result<f64, ()> {
    // Find the first sequence of digits/dots in the string
    let mut start = None;
    let mut end = 0;

    for (i, c) in input.chars().enumerate() {
        if c.is_ascii_digit() || c == '.' || (c == '-' && start.is_none()) {
            if start.is_none() {
                start = Some(i);
            }
            end = i + 1;
        } else if start.is_some() {
            break;
        }
    }

    if let Some(s) = start {
        input[s..end].parse::<f64>().map_err(|_| ())
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nominal_only() {
        let result = parse_dimension_text("25.00").unwrap();
        assert!((result.nominal - 25.0).abs() < 1e-6);
        assert!(result.tolerance.is_none());
    }

    #[test]
    fn test_symmetric_tolerance() {
        let result = parse_dimension_text("25.00 \u{00B1}0.05").unwrap();
        assert!((result.nominal - 25.0).abs() < 1e-6);
        let tol = result.tolerance.unwrap();
        assert!(tol.is_symmetric);
        assert!((tol.upper - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetric_tolerance() {
        let result = parse_dimension_text("25.00 +0.05/-0.02").unwrap();
        assert!((result.nominal - 25.0).abs() < 1e-6);
        let tol = result.tolerance.unwrap();
        assert!(!tol.is_symmetric);
        assert!((tol.upper - 0.05).abs() < 1e-6);
        assert!((tol.lower + 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_diameter() {
        let result = parse_dimension_text("\u{2300}10.00 \u{00B1}0.02").unwrap();
        assert_eq!(result.prefix, DimensionPrefix::Diameter);
        assert!((result.nominal - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_radius() {
        let result = parse_dimension_text("R5.00").unwrap();
        assert_eq!(result.prefix, DimensionPrefix::Radius);
        assert!((result.nominal - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_count_prefix() {
        let result = parse_dimension_text("3X \u{2300}6.5").unwrap();
        assert_eq!(result.count, Some(3));
        assert_eq!(result.prefix, DimensionPrefix::Diameter);
        assert!((result.nominal - 6.5).abs() < 1e-6);
    }

    #[test]
    fn test_thread() {
        let result = parse_dimension_text("M8x1.25").unwrap();
        assert_eq!(
            result.prefix,
            DimensionPrefix::Thread("M8x1.25".to_string())
        );
        assert!((result.nominal - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_reference() {
        let result = parse_dimension_text("(25.00)").unwrap();
        assert!(result.is_reference);
        assert!((result.nominal - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_limit_dimension() {
        let result = parse_dimension_text("25.05/24.98").unwrap();
        assert!((result.nominal - 25.015).abs() < 1e-6);
        assert!(result.tolerance.is_some());
    }

    #[test]
    fn test_garbage_returns_error() {
        let result = parse_dimension_text("hello world");
        assert!(result.is_err());
    }
}
