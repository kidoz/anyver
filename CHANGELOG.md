# Changelog 

## [Unreleased]

### Added
- `Version == str` and all rich comparisons now accept a plain string on the
  right-hand side (`Version("1.0") == "1.0.0"`). Comparing against unrelated
  types returns `NotImplemented` for `==`/`!=` instead of raising.
- Pickle and `copy.deepcopy` support via `__reduce__`, preserving both `raw`
  and the detected/assigned ecosystem.
- `Version.parse(s, ecosystem=...)` classmethod (raises) and
  `Version.try_parse(s, ecosystem=...)` staticmethod (returns `None`).
- `Version.from_dict(d)` classmethod reconstructs a `Version` from the dict
  produced by `to_dict()`. `to_dict()` now also emits an `"ecosystem"` key.
- `anyver.bump_prerelease(version, tag="alpha")` — increments the prerelease
  counter or starts a new `-{tag}.0`.
- `anyver.next_stable(version)` — strips prerelease/build metadata.
- `anyver.satisfies` now understands npm-style shorthand:
  - caret ranges (`^1.2.3`) with 0-version semantics
  - tilde ranges (`~1.2.3`, `~1.2`, `~1`)
  - x-ranges (`1.2.x`, `1.x`, `*`)
  - OR branches via `||`
- Type stubs (`anyver/__init__.pyi`) and `py.typed` marker are now shipped
  with every wheel, making the library fully typed for mypy/Pyright.

### Changed
- Switched to a mixed Python + Rust maturin layout. The compiled extension
  is now `anyver._anyver`; `anyver` itself is a Python package that
  re-exports the public API. Existing imports (`from anyver import Version`)
  continue to work unchanged.
- `Version.to_dict()` now includes the `"ecosystem"` key.

## [0.2.0] - 2026-04

### Added
- Per-ecosystem comparison strategies (semver strict, Debian `verrevcmp`,
  RPM `rpmvercmp`, PEP 440, Ruby, Maven, Go, NuGet, Composer, crates,
  Hex, Swift, CalVer, Alpine, Docker).
- `ecosystem="auto"` autodetection with explicit precedence rules.
- `satisfies`, `stable_versions`, `latest_stable`, `bump_major/minor/patch`.
- `Version.sort_key()` DB-friendly tuple encoding isomorphic to `compare`.
- Rust criterion benchmarks and Python pytest-benchmark suites.

## [0.1.0] - Initial release

- Core `Version` class with generic parser + comparator.
- Module-level `compare`, `sort_versions`, `batch_compare`,
  `min_version`/`max_version`, boolean helpers.
- `compare_semver_strict` for strict SemVer 2.0.0 ordering.
