use super::*;

// ====================================================================
// ECOSYSTEM DATASETS
// ====================================================================
// Each section uses version strings from popular packages,
// tested via ecosystem-specific comparison dispatch.
//
// Sources:
//   npm: registry.npmjs.org  |  crates: crates.io
//   pep440: pypi.org         |  debian: packages.debian.org
//   rpm: koji.fedoraproject  |  ruby: rubygems.org
//   go: pkg.go.dev           |  nuget: nuget.org
//   composer: packagist.org  |  calver: various
//   alpine: pkgs.alpinelinux |  docker: hub.docker.com
//   hex: hex.pm              |  swift: swiftpackageindex.com

// --- npm (strict SemVer) ---

#[test]
fn test_npm_react() {
    assert_chain(
        &[
            "17.0.0", "17.0.1", "17.0.2", "18.0.0", "18.1.0", "18.2.0", "18.3.0", "18.3.1",
            "19.0.0", "19.0.1", "19.1.0",
        ],
        "npm",
    );
}

#[test]
fn test_npm_typescript_prerelease() {
    // TypeScript publishes -beta and -rc pre-releases
    assert_chain(
        &[
            "5.3.0-beta",
            "5.3.0-rc",
            "5.3.0",
            "5.3.3",
            "5.4.0-beta",
            "5.4.0-rc",
            "5.4.0",
            "5.4.5",
            "5.5.0-beta",
            "5.5.0-rc",
            "5.5.0",
            "5.5.4",
        ],
        "npm",
    );
}

#[test]
fn test_npm_nextjs_canary() {
    // Next.js uses -canary and -rc pre-releases
    // SemVer: canary < rc (c < r lexically)
    assert_chain(
        &[
            "14.0.0-canary.0",
            "14.0.0-canary.1",
            "14.0.0-canary.37",
            "14.0.0-rc.0",
            "14.0.0-rc.1",
            "14.0.0",
            "14.0.1",
            "14.0.4",
            "14.1.0",
            "14.2.0",
            "14.2.5",
            "15.0.0-canary.0",
            "15.0.0-rc.0",
            "15.0.0",
            "15.0.1",
            "15.0.4",
            "15.1.0",
            "15.2.0",
            "15.2.4",
        ],
        "npm",
    );
}

#[test]
fn test_npm_express() {
    // Express — pure numeric, major transition 4→5
    assert_chain(
        &[
            "4.17.1", "4.17.2", "4.17.3", "4.18.0", "4.18.1", "4.18.2", "4.18.3", "4.19.0",
            "4.19.2", "4.20.0", "4.21.0", "4.21.2", "5.0.0", "5.0.1",
        ],
        "npm",
    );
}

#[test]
fn test_npm_webpack() {
    assert_chain(
        &[
            "5.88.0", "5.88.2", "5.89.0", "5.90.0", "5.90.3", "5.91.0", "5.92.0", "5.93.0",
            "5.94.0", "5.95.0", "5.96.1", "5.97.1",
        ],
        "npm",
    );
}

// --- Crates.io (strict SemVer) ---

#[test]
fn test_crates_serde() {
    assert_chain(
        &[
            "1.0.195", "1.0.196", "1.0.197", "1.0.198", "1.0.199", "1.0.200", "1.0.201", "1.0.204",
            "1.0.210", "1.0.217",
        ],
        "crates",
    );
}

#[test]
fn test_crates_tokio() {
    assert_chain(
        &[
            "1.35.0", "1.35.1", "1.36.0", "1.37.0", "1.38.0", "1.38.1", "1.39.0", "1.39.2",
            "1.40.0", "1.41.0", "1.42.0", "1.43.0",
        ],
        "crates",
    );
}

#[test]
fn test_crates_clap() {
    assert_chain(
        &["4.4.0", "4.4.1", "4.4.18", "4.5.0", "4.5.1", "4.5.4", "4.5.16", "4.5.20", "4.5.23"],
        "crates",
    );
}

// --- Hex (strict SemVer, Elixir/Erlang) ---

#[test]
fn test_hex_phoenix() {
    // Phoenix Framework pre-releases and releases
    assert_chain(
        &[
            "1.7.0-rc.0",
            "1.7.0-rc.1",
            "1.7.0-rc.2",
            "1.7.0-rc.3",
            "1.7.0",
            "1.7.1",
            "1.7.2",
            "1.7.6",
            "1.7.7",
            "1.7.10",
            "1.7.11",
            "1.7.12",
            "1.7.14",
        ],
        "hex",
    );
}

#[test]
fn test_hex_ecto() {
    assert_chain(
        &["3.11.0", "3.11.1", "3.11.2", "3.12.0", "3.12.1", "3.12.3", "3.12.4", "3.12.5"],
        "hex",
    );
}

// --- Swift PM (strict SemVer) ---

#[test]
fn test_swift_vapor() {
    assert_chain(
        &[
            "4.90.0", "4.91.0", "4.92.0", "4.95.0", "4.99.0", "4.100.0", "4.101.0", "4.105.0",
            "4.107.0",
        ],
        "swift",
    );
}

#[test]
fn test_swift_alamofire() {
    assert_chain(&["5.8.0", "5.8.1", "5.9.0", "5.9.1", "5.10.0", "5.10.1", "5.10.2"], "swift");
}

// --- PEP 440 (Python) ---

#[test]
fn test_pep440_django() {
    assert_chain(
        &[
            "4.2", "4.2.1", "4.2.5", "4.2.10", "4.2.16", "5.0a1", "5.0b1", "5.0rc1", "5.0",
            "5.0.1", "5.0.5", "5.0.10", "5.1a1", "5.1b1", "5.1rc1", "5.1", "5.1.1", "5.1.5",
            "5.1.8", "5.2a1", "5.2b1", "5.2rc1", "5.2",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_cpython() {
    // CPython release cycle: alpha → beta → rc → release
    assert_chain(
        &[
            "3.12.0a1",
            "3.12.0a4",
            "3.12.0a7",
            "3.12.0b1",
            "3.12.0b4",
            "3.12.0rc1",
            "3.12.0rc3",
            "3.12.0",
            "3.12.1",
            "3.12.4",
            "3.12.8",
            "3.13.0a1",
            "3.13.0a6",
            "3.13.0b1",
            "3.13.0b4",
            "3.13.0rc1",
            "3.13.0rc3",
            "3.13.0",
            "3.13.1",
            "3.13.3",
            "3.14.0a1",
            "3.14.0a7",
            "3.14.0b1",
            "3.14.0b2",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_numpy() {
    assert_chain(
        &[
            "1.26.0", "1.26.2", "1.26.4", "2.0.0rc1", "2.0.0rc2", "2.0.0", "2.0.1", "2.0.2",
            "2.1.0rc1", "2.1.0", "2.1.1", "2.1.3", "2.2.0rc1", "2.2.0", "2.2.1", "2.2.4",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_setuptools() {
    assert_chain(
        &[
            "69.0.0", "69.5.1", "70.0.0", "70.3.0", "71.0.0", "71.1.0", "72.0.0", "72.2.0",
            "73.0.0", "73.0.1", "74.0.0", "74.1.3", "75.0.0", "75.6.0",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_full_lifecycle() {
    // Complete PEP 440 lifecycle: dev → alpha → beta → rc → release → post
    assert_chain(
        &[
            "1.0.dev1",
            "1.0.dev2",
            "1.0a1",
            "1.0a2",
            "1.0b1",
            "1.0b2",
            "1.0rc1",
            "1.0rc2",
            "1.0",
            "1.0.post1",
            "1.0.post2",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_epoch() {
    // Epoch resets version ordering
    assert_eq!(compare_str_with_ecosystem("2023.1", "1!0.1", "pep440").unwrap(), Less,);
    assert_eq!(compare_str_with_ecosystem("1!0.1", "1!0.2", "pep440").unwrap(), Less,);
    assert_eq!(compare_str_with_ecosystem("1!0.1", "2!0.1", "pep440").unwrap(), Less,);
}
