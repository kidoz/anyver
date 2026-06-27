use super::*;

// --- Debian ---

#[test]
fn test_debian_openssl() {
    assert_chain(
        &[
            "1.0.2g-1",
            "1.0.2g-1+deb8u1",
            "1.0.2g-1+deb8u2",
            "1.0.2l-2",
            "1.0.2l-2+deb9u1",
            "1.0.2l-2+deb9u3",
            "1.1.0g-2",
            "1.1.0j-1",
            "1.1.1a-1",
            "1.1.1d-0",
            "1.1.1d-0+deb10u1",
            "1.1.1d-0+deb10u7",
            "1.1.1n-0",
            "1.1.1n-0+deb10u6",
            "1.1.1w-0",
            "3.0.11-1",
            "3.0.13-1",
            "3.0.14-1",
        ],
        "debian",
    );
}

#[test]
fn test_debian_curl() {
    assert_chain(
        &[
            "7.64.0-4",
            "7.64.0-4+deb10u1",
            "7.64.0-4+deb10u7",
            "7.74.0-1.3",
            "7.74.0-1.3+deb11u1",
            "7.74.0-1.3+deb11u12",
            "7.88.1-10",
            "7.88.1-10+deb12u1",
            "7.88.1-10+deb12u8",
            "8.5.0-2",
            "8.11.1-1",
        ],
        "debian",
    );
}

#[test]
fn test_debian_python3() {
    assert_chain(
        &[
            "3.9.2-3",
            "3.11.2-1",
            "3.11.2-1+b1",
            "3.11.4-5",
            "3.11.4-5+deb12u1",
            "3.12.3-1",
            "3.12.8-1",
        ],
        "debian",
    );
}

#[test]
fn test_debian_nginx() {
    assert_chain(
        &[
            "1.14.0-0",
            "1.14.0-0+deb9u1",
            "1.14.0-0+deb9u5",
            "1.14.2-2",
            "1.14.2-2+deb10u1",
            "1.14.2-2+deb10u5",
            "1.18.0-6",
            "1.18.0-6.1",
            "1.22.1-9",
            "1.24.0-2",
            "1.26.0-3",
        ],
        "debian",
    );
}

#[test]
fn test_debian_tilde_prerelease() {
    // Debian pre-release convention: ~ sorts before everything
    assert_chain(
        &[
            "1.0~alpha1",
            "1.0~alpha2",
            "1.0~beta1",
            "1.0~beta2",
            "1.0~rc1",
            "1.0~rc2",
            "1.0",
            "1.0-1",
            "1.0-2",
        ],
        "debian",
    );
}

#[test]
fn test_debian_epoch_ordering() {
    assert_chain(&["0:1.0-1", "0:2.0-1", "1:0.1-1", "1:0.2-1", "2:0.1-1"], "debian");
}

// --- RPM ---

#[test]
fn test_rpm_kernel() {
    // RHEL 9 kernel versions
    assert_chain(
        &[
            "5.14.0-362.13.1.el9_3",
            "5.14.0-362.18.1.el9_3",
            "5.14.0-362.24.1.el9_3",
            "5.14.0-427.13.1.el9_4",
            "5.14.0-427.20.1.el9_4",
            "5.14.0-427.31.1.el9_4",
            "5.14.0-503.11.1.el9_5",
            "5.14.0-503.14.1.el9_5",
            "5.14.0-503.19.1.el9_5",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_systemd() {
    // Fedora systemd versions
    assert_chain(
        &[
            "252-14.fc38",
            "252-18.fc38",
            "253-1.fc39",
            "253-7.fc39",
            "253-14.fc39",
            "254-1.fc40",
            "254-10.fc40",
            "255-1.fc40",
            "256-1.fc41",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_httpd() {
    // RHEL Apache httpd versions
    assert_chain(
        &[
            "2.4.37-47.el8",
            "2.4.37-62.el8",
            "2.4.37-65.el8",
            "2.4.51-7.el9_0",
            "2.4.53-7.el9",
            "2.4.57-5.el9",
            "2.4.57-8.el9",
            "2.4.62-1.el9",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_gcc() {
    // GCC versions in Fedora
    assert_chain(
        &[
            "13.2.1-6.fc39",
            "13.2.1-7.fc39",
            "13.3.1-2.fc40",
            "13.3.1-3.fc40",
            "14.0.1-0.6.fc40",
            "14.1.1-1.fc40",
            "14.1.1-6.fc41",
            "14.2.1-1.fc41",
            "14.2.1-3.fc41",
            "14.2.1-6.fc41",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_tilde_caret() {
    // RPM pre/post-release semantics with ~ and ^
    assert_chain(&["1.0~alpha", "1.0~beta", "1.0~rc1", "1.0", "1.0^git1", "1.0^git2"], "rpm");
}

// --- Ruby Gems ---

#[test]
fn test_ruby_rails() {
    // Ruby on Rails with pre-releases
    assert_chain(
        &[
            "7.0.0.alpha1",
            "7.0.0.alpha2",
            "7.0.0.rc1",
            "7.0.0.rc2",
            "7.0.0.rc3",
            "7.0.0",
            "7.0.1",
            "7.0.2",
            "7.0.4",
            "7.0.8",
            "7.1.0.beta1",
            "7.1.0.rc1",
            "7.1.0.rc2",
            "7.1.0",
            "7.1.1",
            "7.1.2",
            "7.1.3",
            "7.1.5",
            "7.2.0.beta1",
            "7.2.0.beta2",
            "7.2.0.beta3",
            "7.2.0.rc1",
            "7.2.0",
            "7.2.1",
            "7.2.2",
        ],
        "ruby",
    );
}

#[test]
fn test_ruby_nokogiri() {
    assert_chain(
        &[
            "1.15.0", "1.15.1", "1.15.4", "1.15.6", "1.16.0", "1.16.2", "1.16.5", "1.16.7",
            "1.17.0", "1.17.2", "1.18.0", "1.18.2", "1.18.3",
        ],
        "ruby",
    );
}

#[test]
fn test_ruby_bundler() {
    assert_chain(
        &[
            "2.4.0", "2.4.1", "2.4.10", "2.4.22", "2.5.0", "2.5.1", "2.5.10", "2.5.23", "2.6.0",
            "2.6.1", "2.6.2", "2.6.3",
        ],
        "ruby",
    );
}

#[test]
fn test_ruby_sinatra() {
    assert_chain(
        &[
            "3.0.0", "3.0.1", "3.0.2", "3.0.3", "3.0.4", "3.0.5", "3.0.6", "3.1.0", "3.2.0",
            "4.0.0", "4.1.0", "4.1.1",
        ],
        "ruby",
    );
}

// --- Go modules ---

#[test]
fn test_go_gin() {
    assert_chain(
        &["v1.7.0", "v1.7.7", "v1.8.0", "v1.8.1", "v1.8.2", "v1.9.0", "v1.9.1", "v1.10.0"],
        "go",
    );
}

#[test]
fn test_go_cobra() {
    assert_chain(&["v1.7.0", "v1.8.0", "v1.8.1"], "go");
}

#[test]
fn test_go_prometheus() {
    // prometheus/client_golang
    assert_chain(
        &[
            "v1.17.0", "v1.18.0", "v1.19.0", "v1.19.1", "v1.20.0", "v1.20.1", "v1.20.2", "v1.20.3",
            "v1.20.4", "v1.20.5",
        ],
        "go",
    );
}

#[test]
fn test_go_incompatible_suffix() {
    // Go modules with +incompatible suffix (pre-module v2+ packages)
    assert_eq!(compare_str_with_ecosystem("v2.0.0+incompatible", "v2.0.0", "go").unwrap(), Equal,);
    assert_eq!(
        compare_str_with_ecosystem("v2.0.0+incompatible", "v2.0.1+incompatible", "go").unwrap(),
        Less,
    );
}

// --- NuGet ---

#[test]
fn test_nuget_newtonsoft() {
    assert_chain(&["12.0.1", "12.0.2", "12.0.3", "13.0.1", "13.0.2", "13.0.3"], "nuget");
}

#[test]
fn test_nuget_xunit() {
    assert_chain(
        &[
            "2.5.0", "2.5.1", "2.5.3", "2.6.0", "2.6.1", "2.6.6", "2.7.0", "2.7.1", "2.8.0",
            "2.8.1", "2.9.0", "2.9.2", "2.9.3",
        ],
        "nuget",
    );
}

#[test]
fn test_nuget_nunit() {
    assert_chain(
        &["3.13.3", "3.14.0", "4.0.0", "4.0.1", "4.1.0", "4.2.0", "4.2.2", "4.3.0", "4.3.2"],
        "nuget",
    );
}

#[test]
fn test_nuget_efcore() {
    // Entity Framework Core
    assert_chain(
        &[
            "7.0.0", "7.0.1", "7.0.5", "7.0.11", "7.0.20", "8.0.0", "8.0.1", "8.0.4", "8.0.8",
            "8.0.11", "9.0.0", "9.0.1", "9.0.3", "9.0.4",
        ],
        "nuget",
    );
}

// --- Composer (PHP) ---

#[test]
fn test_composer_laravel() {
    assert_chain(
        &[
            "10.0.0", "10.0.1", "10.10.0", "10.20.0", "10.40.0", "10.48.0", "11.0.0", "11.10.0",
            "11.20.0", "11.30.0", "11.40.0", "12.0.0", "12.1.0", "12.2.0", "12.3.0", "12.4.0",
        ],
        "composer",
    );
}

#[test]
fn test_composer_symfony() {
    assert_chain(
        &[
            "6.4.0", "6.4.1", "6.4.5", "6.4.10", "6.4.15", "6.4.21", "7.0.0", "7.0.1", "7.0.7",
            "7.0.12", "7.1.0", "7.1.1", "7.1.6", "7.1.12", "7.2.0", "7.2.1", "7.2.6",
        ],
        "composer",
    );
}

#[test]
fn test_composer_phpunit() {
    assert_chain(
        &[
            "10.5.0", "10.5.10", "10.5.20", "10.5.38", "11.0.0", "11.0.11", "11.1.0", "11.1.6",
            "11.2.0", "11.2.6", "11.3.0", "11.3.6", "11.4.0", "11.4.4", "11.5.0", "11.5.6",
            "12.0.0", "12.0.6",
        ],
        "composer",
    );
}

#[test]
fn test_composer_prerelease() {
    // Composer stability flags: alpha < beta < RC < stable
    assert_chain(
        &[
            "1.0.0-alpha1",
            "1.0.0-alpha2",
            "1.0.0-beta1",
            "1.0.0-beta2",
            "1.0.0-rc1",
            "1.0.0-rc2",
            "1.0.0",
            "1.0.1",
        ],
        "composer",
    );
}

// --- CalVer ---

#[test]
fn test_calver_ubuntu() {
    // Ubuntu release versions (YY.MM format)
    assert_chain(
        &[
            "20.04", "20.10", "21.04", "21.10", "22.04", "22.10", "23.04", "23.10", "24.04",
            "24.10", "25.04",
        ],
        "calver",
    );
}

#[test]
fn test_calver_pip() {
    // pip uses CalVer (YY.N format)
    assert_chain(
        &[
            "23.0", "23.0.1", "23.1", "23.1.2", "23.2", "23.2.1", "23.3", "23.3.2", "24.0", "24.1",
            "24.1.2", "24.2", "24.3", "24.3.1", "25.0", "25.0.1",
        ],
        "calver",
    );
}

#[test]
fn test_calver_black() {
    // black Python formatter (YYYY.M.D format)
    assert_chain(
        &[
            "23.1.0", "23.3.0", "23.7.0", "23.9.0", "23.9.1", "23.10.0", "23.10.1", "23.11.0",
            "23.12.0", "23.12.1", "24.1.0", "24.1.1", "24.2.0", "24.3.0", "24.4.0", "24.4.2",
            "24.8.0", "24.10.0", "25.1.0",
        ],
        "calver",
    );
}

#[test]
fn test_calver_yyyymmdd() {
    // YYYYMMDD format (e.g., database snapshots, Commons IO timestamp)
    assert_chain(
        &["20230115", "20230201", "20230315", "20240101", "20240615", "20241231", "20250101"],
        "calver",
    );
}

// --- Alpine ---

#[test]
fn test_alpine_prerelease_lifecycle() {
    // Alpine uses _alpha, _beta, _rc for pre-release
    assert_chain(
        &[
            "1.0_alpha1",
            "1.0_alpha2",
            "1.0_beta1",
            "1.0_beta2",
            "1.0_rc1",
            "1.0_rc2",
            "1.0",
            "1.0.1",
            "1.1.0",
        ],
        "alpine",
    );
}

#[test]
fn test_alpine_curl() {
    assert_chain(
        &[
            "8.5.0", "8.6.0", "8.7.1", "8.8.0", "8.9.0", "8.9.1", "8.10.0", "8.10.1", "8.11.0",
            "8.11.1",
        ],
        "alpine",
    );
}

#[test]
fn test_alpine_openssl() {
    assert_chain(
        &[
            "3.1.4", "3.1.5", "3.1.6", "3.1.7", "3.2.0", "3.2.1", "3.2.2", "3.2.3", "3.3.0",
            "3.3.1", "3.3.2", "3.4.0", "3.4.1",
        ],
        "alpine",
    );
}

#[test]
fn test_alpine_busybox() {
    assert_chain(&["1.35.0", "1.36.0", "1.36.1", "1.37.0"], "alpine");
}

// --- Docker ---

#[test]
fn test_docker_nginx() {
    assert_chain(
        &[
            "1.24.0", "1.25.0", "1.25.1", "1.25.2", "1.25.3", "1.25.4", "1.25.5", "1.26.0",
            "1.26.1", "1.26.2", "1.27.0", "1.27.1", "1.27.2", "1.27.3",
        ],
        "docker",
    );
}

#[test]
fn test_docker_postgres() {
    assert_chain(
        &[
            "14.0", "14.5", "14.10", "14.13", "15.0", "15.4", "15.8", "16.0", "16.2", "16.4",
            "17.0", "17.1", "17.2",
        ],
        "docker",
    );
}

#[test]
fn test_docker_redis() {
    assert_chain(
        &[
            "7.0.0", "7.0.5", "7.0.10", "7.0.15", "7.2.0", "7.2.3", "7.2.6", "7.4.0", "7.4.1",
            "7.4.2", "8.0.0", "8.0.1", "8.0.2",
        ],
        "docker",
    );
}

#[test]
fn test_docker_node() {
    // Node.js Docker image tags
    assert_chain(
        &[
            "18.0.0", "18.12.0", "18.17.0", "18.20.0", "20.0.0", "20.9.0", "20.11.0", "20.18.0",
            "22.0.0", "22.6.0", "22.11.0",
        ],
        "docker",
    );
}
