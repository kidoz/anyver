//! Lightweight input validators for ecosystems that parse via the generic
//! strategy but enforce a "must start with a digit" (or similar) precondition.

use crate::parser::strip_v;

/// Go module versions are *always* a lowercase `v` prefix followed by a digit
/// (spec: "Always `v` prefix"). Reject a missing prefix and an uppercase `V`,
/// both of which strict `SemVer`'s `strip_v` would otherwise accept silently.
pub(crate) fn validate_go(input: &str) -> Result<(), String> {
    let b = input.trim().as_bytes();
    if b.len() < 2 || b[0] != b'v' || !b[1].is_ascii_digit() {
        return Err(format!("Go version must start with a lowercase 'v' prefix: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_ruby(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Ruby gem version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Ruby gem version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_maven(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Maven version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Maven version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_nuget(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty NuGet version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("NuGet version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_composer(input: &str) -> Result<(), String> {
    let s = strip_v(input.trim());
    if s.is_empty() {
        return Err("empty Composer version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Composer version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_calver(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty CalVer version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("CalVer version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_alpine(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Alpine version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Alpine version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_docker(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Docker tag".to_string());
    }
    Ok(())
}

pub(crate) fn validate_nonempty(input: &str, label: &str) -> Result<(), String> {
    if input.trim().is_empty() {
        return Err(format!("empty {label} version"));
    }
    Ok(())
}

pub(crate) fn validate_digit_start(input: &str, label: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err(format!("empty {label} version"));
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("{label} version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_conda(input: &str) -> Result<(), String> {
    validate_digit_start(input, "Conda")
}

pub(crate) fn validate_arch(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Arch version".to_string());
    }
    let after_epoch = match s.split_once(':') {
        Some((epoch, rest)) if epoch.bytes().all(|b| b.is_ascii_digit()) => rest,
        _ => s,
    };
    if after_epoch.is_empty() || !after_epoch.as_bytes()[0].is_ascii_alphanumeric() {
        return Err(format!("invalid Arch version: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_gentoo(input: &str) -> Result<(), String> {
    validate_digit_start(input, "Gentoo")
}

pub(crate) fn validate_cran(input: &str) -> Result<(), String> {
    let s = input.trim();
    validate_digit_start(s, "R/CRAN")?;
    if !s.bytes().all(|b| b.is_ascii_digit() || b == b'.' || b == b'-') {
        return Err(format!("R/CRAN version must contain only digits, '.', or '-': '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_spack(input: &str) -> Result<(), String> {
    validate_nonempty(input, "Spack")
}

pub(crate) fn validate_terraform(input: &str) -> Result<(), String> {
    validate_digit_start(input, "Terraform/OpenTofu")
}

pub(crate) fn validate_cpan(input: &str) -> Result<(), String> {
    validate_nonempty(input, "CPAN")
}

pub(crate) fn validate_nix(input: &str) -> Result<(), String> {
    validate_nonempty(input, "Nix")
}

pub(crate) fn validate_homebrew(input: &str) -> Result<(), String> {
    validate_nonempty(input, "Homebrew")
}

pub(crate) fn validate_freebsd(input: &str) -> Result<(), String> {
    validate_nonempty(input, "FreeBSD pkg")
}
