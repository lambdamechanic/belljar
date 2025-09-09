use proptest::prelude::*;
use tempfile::TempDir;

fn init_temp_data_dir() -> TempDir {
    let td = TempDir::new().expect("tempdir");
    par_core::set_data_dir_override_for_testing(td.path());
    td
}

proptest! {
    #[test]
    fn registry_roundtrip(labels in proptest::collection::vec("[a-zA-Z0-9_-]{1,16}", 1..5)) {
        let _td = init_temp_data_dir();

        let repo_root = TempDir::new().unwrap();
        let repo = repo_root.path();
        std::fs::create_dir_all(repo).unwrap();

        // create sessions
        for (i, label) in labels.iter().enumerate() {
            let branch = if i % 2 == 0 { Some(format!("feat_{}", i)) } else { None };
            let s = par_core::create_session(label, repo, branch, vec![]).expect("create session");
            assert_eq!(&s.label, label);
        }

        let reg = par_core::load_registry().expect("load");
        // uniqueness of labels in property is not guaranteed; dedup the input
        use std::collections::HashSet;
        let set: HashSet<_> = labels.iter().cloned().collect();
        assert_eq!(reg.sessions.len(), set.len());

        // find and remove
        if let Some(first) = labels.first() {
            let found = par_core::find_session(first).expect("find");
            assert!(found.is_some());
            let removed = par_core::remove_session(first).expect("remove");
            assert!(removed.is_some());
            let found_again = par_core::find_session(first).expect("find");
            assert!(found_again.is_none());
        }
    }
}

