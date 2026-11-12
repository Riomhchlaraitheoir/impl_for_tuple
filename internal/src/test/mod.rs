use std::error::Error;
use crate::impl_for_tuple;
use proc_macro2::TokenStream;
use syn::File;

fn test_case(input: &str, expected: &str) {
    let input: TokenStream = input.parse().expect("failed to parse input");
    let expected: TokenStream = expected.parse().expect("failed to parse expected output");
    let expected: File = syn::parse2(expected).expect("failed to parse expected output");
    let actual = impl_for_tuple(input).map_err(render_error).expect("failed to generate output");
    let actual: File = syn::parse2(actual).expect("failed to parse actual output");
    let expected = prettyplease::unparse(&expected);
    let actual = prettyplease::unparse(&actual);
    difference::assert_diff!(&expected, &actual, "\n", 0);
}

fn render_error(error: syn::Error) -> String {
    let start = error.span().start();
    let end = error.span().end();
    let source = error.span().source_text().unwrap_or_default();
    format!("[{}:{}]-[{}:{}]:: {error}\nsource:\n{source}", start.line, start.column, end.line, end.column)
}

macro_rules! test_cases {
    ($($name:ident),*) => {
        $(
        #[test]
        fn $name() {
            test_case(include_str!(concat!("inputs/", stringify!($name), ".rs")), include_str!(concat!("outputs/", stringify!($name), ".rs")));
        }
        )*
    };
}

test_cases![marker, notify, validate, validator];