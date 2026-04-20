pub mod parser;
pub mod python;
pub mod strategies;

#[cfg(test)]
mod tests;

// Benchmark-only facade over internal parser/comparator APIs.
// Uses opaque wrapper types so the crate's `pub(crate)` internals stay
// non-public. Hidden from docs; only compiled with `--features bench`.
#[cfg(feature = "bench")]
#[doc(hidden)]
pub mod __bench {
    use std::cmp::Ordering;

    #[derive(Clone, Copy)]
    pub struct EcoHandle(crate::parser::Ecosystem);

    #[derive(Clone)]
    pub struct Parsed(crate::strategies::ParsedRepr);

    pub fn eco_from_str(s: &str) -> Result<EcoHandle, String> {
        crate::parser::Ecosystem::from_str(s).map(EcoHandle)
    }

    pub fn autodetect(input: &str) -> EcoHandle {
        EcoHandle(crate::parser::autodetect_ecosystem(input))
    }

    pub fn parse_generic(input: &str) -> Parsed {
        Parsed(crate::strategies::ParsedRepr::Generic(crate::parser::parse(input)))
    }

    pub fn parse_for(eco: EcoHandle, input: &str) -> Result<Parsed, String> {
        crate::strategies::parse_for_ecosystem(eco.0, input).map(Parsed)
    }

    pub fn compare(eco: EcoHandle, a: &Parsed, b: &Parsed) -> i32 {
        match crate::strategies::compare_for_ecosystem(eco.0, &a.0, &b.0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}
