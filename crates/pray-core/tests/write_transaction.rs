use pray_core::{transaction, PrayError};
use std::{fs, process::Command};

fn root() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "pray-transaction-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

#[test]
fn rolls_back_files_and_metadata_after_failure() {
    let root = root();
    fs::write(root.join("Prayfile"), "original").unwrap();
    let result = transaction::run(&root, || {
        transaction::write_file(&root.join("Prayfile"), b"intermediate")?;
        transaction::write_file(&root.join("Prayfile"), b"candidate")?;
        transaction::write_file(&root.join("output"), b"created")?;
        Err::<(), _>(PrayError::Render("late failure".into()))
    });
    assert!(result.is_err());
    assert_eq!(fs::read(root.join("Prayfile")).unwrap(), b"original");
    assert!(!root.join("output").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovers_after_process_exit_and_preserves_later_edits() {
    for edited in [false, true] {
        let root = root();
        fs::write(root.join("Prayfile"), "original").unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "crash_child", "--nocapture"])
            .env("PRAY_TRANSACTION_TEST_ROOT", &root)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(91),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if edited {
            fs::write(root.join("Prayfile"), "operator edit").unwrap();
        }
        let result = transaction::run(&root, || Ok(()));
        if edited {
            assert!(result.is_err());
            assert_eq!(fs::read(root.join("Prayfile")).unwrap(), b"operator edit");
            fs::rename(root.join("Prayfile"), root.join("operator.saved")).unwrap();
            transaction::run(&root, || Ok(())).unwrap();
        } else {
            result.unwrap();
        }
        assert_eq!(fs::read(root.join("Prayfile")).unwrap(), b"original");
        assert!(!root.join("output").exists());
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn crash_child() {
    let Some(root) = std::env::var_os("PRAY_TRANSACTION_TEST_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    if std::env::var("PRAY_TRANSACTION_TEST_ACTION").as_deref() == Ok("recover") {
        transaction::run(&root, || Ok(())).unwrap();
        return;
    }
    transaction::run(&root, || -> pray_core::PrayResult<()> {
        transaction::write_file(&root.join("Prayfile"), b"intermediate")?;
        transaction::write_file(&root.join("Prayfile"), b"candidate")?;
        transaction::write_file(&root.join("output"), b"created")?;
        std::process::exit(91);
    })
    .unwrap();
}

#[test]
fn excludes_a_second_writer_until_commit() {
    let root = root();
    transaction::run(&root, || {
        transaction::write_file(&root.join("output"), b"first")?;
        std::thread::scope(|scope| {
            let result = scope
                .spawn(|| {
                    transaction::run(&root, || {
                        transaction::write_file(&root.join("output"), b"second")
                    })
                })
                .join()
                .unwrap();
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("another pray command"));
        });
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read(root.join("output")).unwrap(), b"first");
    fs::remove_dir_all(root).unwrap();
}
