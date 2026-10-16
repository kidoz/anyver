"""Regressions for version, range, and security interoperability contracts."""

from itertools import permutations, product

import pytest

import anyver
from anyver import Version


def sign(value):
    return (value > 0) - (value < 0)


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
