"""Regressions for version, range, and security interoperability contracts."""

from itertools import permutations, product

import pytest

import anyver
from anyver import Version


def sign(value):
    return (value > 0) - (value < 0)


@pytest.mark.parametrize("value", ["1.2.3+", "1.2.3+foo..bar", "1.2.3+foo_bar", "1.2.3+foo+bar"])
def test_semver_build_grammar(value):
    assert Version.try_parse(value, ecosystem="semver") is None
    with pytest.raises(ValueError):
        anyver.compare_semver_strict(value, "1.2.3")


def test_semver_valid_build_ignored():
    assert anyver.compare_semver_strict("1.2.3+001.foo-bar", "1.2.3") == 0


@pytest.mark.parametrize(
    "value", ["1..0", "1.0+", "1!2!3", "1..a1", "1.0a..1", "1.0+abc..1", "1.0wat", "1.0.post1rc1"]
)
def test_pep440_rejects_malformed(value):
    assert Version.try_parse(value, ecosystem="pep440") is None


@pytest.mark.parametrize(
    "a,b,expected",
    [
        ("1.0rc1", "1.0.0rc1", 0),
        ("1.0-1", "1.0.post1", 0),
        ("1.0+abc1", "1.0+abc.1", 1),
        ("1.0+abc.0", "1.0+abc", 1),
        ("1.0a", "1.0alpha0", 0),
        ("1.0r1", "1.0.post1", 0),
        ("1.0.dev1", "1.0a0", -1),
        ("1.0a1.dev1", "1.0a1", -1),
        ("1.0.post1.dev1", "1.0.post1", -1),
        ("1.0+1", "1.0+abc", 1),
    ],
)
def test_pep440_comparison(a, b, expected):
    assert anyver.compare(a, b, ecosystem="pep440") == expected


@pytest.mark.parametrize(
    "eco,a,b",
    [
        ("rpm", "1.0+a", "1.0.a"),
        ("rpm", "1.01", "1.1"),
        ("debian", "1.0", "1.0-0"),
        ("debian", "1.01", "1.1"),
        ("pep440", "1.0rc1", "1.0.0RC1"),
        ("pep440", "1.0-1", "1.0.post1"),
        ("maven", "1-a1", "1-alpha-1"),
        ("generic", "1.0-a", "1-alpha"),
        ("semver", "1.0.0+build", "1.0.0+other"),
    ],
)
def test_equal_versions_have_identical_hashes(eco, a, b):
    left, right = Version(a, ecosystem=eco), Version(b, ecosystem=eco)
    assert left == right
    assert hash(left) == hash(right)
    assert len({left, right}) == 1
    assert {left: "found"}[right] == "found"


def test_equality_domain():
    generic = Version("1.0", ecosystem="generic")
    a, b = (Version("1.0+" + label, ecosystem="pep440") for label in ["a", "b"])
    assert a != b and a != generic and b != generic
    assert generic != "1.0.0" and "1.0.0" != generic
    assert generic.compare("1.0.0") == 0
    with pytest.raises(TypeError):
        _ = a < generic


@pytest.mark.parametrize(
    "eco,values",
    [
        (
            "generic",
            [
                "1",
                "1.0",
                "1.0.0-rc1",
                "1.0-rc1",
                "1-post",
                "1-a",
                "1-alpha",
                "1-final7",
                "1.7",
                "1.18446744073709551615",
                "1.9223372036854775808",
                "1.9223372036854775807",
            ],
        ),
        (
            "semver",
            [
                "1.0.0-alpha",
                "1.0.0-alpha.0",
                "1.0.0-alpha.1",
                "1.0.0-alpha.beta",
                "1.0.0-Alpha",
                "1.0.0",
                "1.0.0+foo",
                "2.0.0",
            ],
        ),
        (
            "pep440",
            [
                "1.0",
                "1.0.0",
                "1.0rc1",
                "1.0.0rc1",
                "1.0.dev0",
                "1.0.post1.dev1",
                "1.0.post1",
                "1.0+abc",
                "1.0+abc1",
                "1.0+abc.1",
                "1.0+1",
                "1!0.1",
            ],
        ),
    ],
)
def test_sort_keys_match_all_pairwise_comparisons(eco, values):
    for a, b in product(values, repeat=2):
        left, right = Version(a, ecosystem=eco), Version(b, ecosystem=eco)
        ka, kb = left.sort_key(), right.sort_key()
        assert sign((ka > kb) - (ka < kb)) == anyver.compare(a, b, ecosystem=eco), (eco, a, b)
        assert (left == right) == (ka == kb)


@pytest.mark.parametrize("eco", ["rpm", "debian", "maven"])
def test_native_keys_fail_explicitly(eco):
    with pytest.raises(ValueError, match="sort_versions"):
        Version("1.0", ecosystem=eco).sort_key()
