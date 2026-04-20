"""anyver — high-performance version parsing & comparison across ecosystems."""

from ._anyver import (
    Version,
    batch_compare,
    bump_major,
    bump_minor,
    bump_patch,
    bump_prerelease,
    compare,
    compare_semver_strict,
    eq,
    ge,
    gt,
    gte,
    latest_stable,
    le,
    lt,
    lte,
    max_version,
    min_version,
    ne,
    next_stable,
    satisfies,
    sort_versions,
    stable_versions,
    version,
)

try:
    from importlib.metadata import version as _pkg_version

    __version__ = _pkg_version("anyver")
except Exception:
    __version__ = "0.0.0+unknown"

__all__ = [
    "Version",
    "__version__",
    "batch_compare",
    "bump_major",
    "bump_minor",
    "bump_patch",
    "bump_prerelease",
    "compare",
    "compare_semver_strict",
    "eq",
    "ge",
    "gt",
    "gte",
    "latest_stable",
    "le",
    "lt",
    "lte",
    "max_version",
    "min_version",
    "ne",
    "next_stable",
    "satisfies",
    "sort_versions",
    "stable_versions",
    "version",
]
