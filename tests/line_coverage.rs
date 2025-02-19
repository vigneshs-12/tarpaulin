use crate::utils::get_test_path;
use cargo_tarpaulin::config::{Color, Config};
use cargo_tarpaulin::traces::CoverageStat;
use cargo_tarpaulin::{launch_tarpaulin, setup_logging};
use rusty_fork::rusty_fork_test;
use std::env;
use std::time::Duration;

rusty_fork_test! {

#[test]
fn simple_project_coverage() {
    setup_logging(Color::Never, false, false);
    let mut config = Config::default();
    config.set_clean(false);
    config.test_timeout = Duration::from_secs(60);
    let restore_dir = env::current_dir().unwrap();
    let test_dir = get_test_path("simple_project");
    env::set_current_dir(&test_dir).unwrap();
    let mut manifest = test_dir.clone();
    manifest.push("Cargo.toml");
    config.set_manifest(manifest);

    let (res, ret) = launch_tarpaulin(&config, &None).unwrap();
    assert_eq!(ret, 0);
    env::set_current_dir(restore_dir).unwrap();
    let unused_file = test_dir.join("src/unused.rs");
    let unused_hits = res.covered_in_path(&unused_file);
    let unused_lines = res.coverable_in_path(&unused_file);
    assert_eq!(unused_hits, 0);
    assert_eq!(unused_lines, 3);
    let unused_hits = res
        .get_child_traces(&unused_file)
        .map(|x| x.line)
        .collect::<Vec<_>>();

    assert_eq!(unused_hits.len(), 3);
    assert!(unused_hits.contains(&4));
    assert!(unused_hits.contains(&5));
    assert!(unused_hits.contains(&6));

    let lib_file = test_dir.join("src/lib.rs");
    let lib_traces = res.get_child_traces(&lib_file);
    for l in lib_traces {
        if l.line == 6 {
            assert_eq!(CoverageStat::Line(0), l.stats);
        } else if l.line == 8 {
            assert!(matches!(l.stats, CoverageStat::Line(c) if c > 0 ));
        }
    }
}

#[test]
fn debug_info_0() {
    setup_logging(Color::Never, false, false);
    // From issue #601
    let mut config = Config::default();
    config.set_clean(false);
    let restore_dir = env::current_dir().unwrap();
    let test_dir = get_test_path("simple_project");
    env::set_current_dir(&test_dir).unwrap();
    let mut manifest = test_dir;
    manifest.push("Cargo.toml");
    config.set_manifest(manifest);
    let backup_flag = env::var("RUSTFLAGS").ok();
    env::set_var("RUSTFLAGS", "-Cdebuginfo=0");
    let (res, ret) = launch_tarpaulin(&config, &None).unwrap();
    match backup_flag {
        None => env::remove_var("RUSTFLAGS"),
        Some(s) => env::set_var("RUSTFLAGS", s),
    };
    assert_eq!(ret, 0);
    assert!(!res.is_empty());
    env::set_current_dir(restore_dir).unwrap();
}

#[test]
fn test_threads_1() {
    setup_logging(Color::Never, false, false);
    let mut config = Config::default();
    config.set_clean(false);
    let restore_dir = env::current_dir().unwrap();
    let test_dir = get_test_path("simple_project");
    env::set_current_dir(&test_dir).unwrap();
    let mut manifest = test_dir.clone();
    manifest.push("Cargo.toml");
    config.set_manifest(manifest);
    config.varargs.push("--test-threads".to_string());
    config.varargs.push("1".to_string());

    let (res, ret) = launch_tarpaulin(&config, &None).unwrap();
    assert_eq!(ret, 0);
    assert!(!res.is_empty());
    env::set_current_dir(restore_dir).unwrap();

    let lib_file = test_dir.join("src/lib.rs");
    let lib_traces = res.get_child_traces(&lib_file);
    for l in lib_traces {
        if l.line == 6 {
            assert_eq!(CoverageStat::Line(0), l.stats);
        } else if l.line == 8 {
            assert_eq!(CoverageStat::Line(1), l.stats);
        }
    }
}

}
