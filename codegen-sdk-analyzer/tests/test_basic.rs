use std::{path::PathBuf, thread::sleep, time::Duration};

use codegen_sdk_analyzer::Codebase;
use codegen_sdk_resolution::{CodebaseContext, File};
use rfs_test_macro::rfs_test;
#[cfg(feature = "python")]
#[test_log::test(rfs_test(
    config = r#"
    - !directory
        name: testa
        content:
          - !file
              name: test.py
              content:
                !inline_text "import codegen_sdk_pink"
    "#,
    start_point = "./test_data/"
))]
fn test_basic(dirname: &str) -> Result<(), Error> {
    let codebase = Codebase::new(PathBuf::from(dirname));
    assert_eq!(codebase.files().len(), 1);
    assert_eq!(codebase.files()[0].name(codebase.db()), "test.py");
    Ok(())
}
#[cfg(feature = "python")]
#[test_log::test(rfs_test(
    config = r#"
    - !directory
        name: testb
        content:
          - !file
              name: test.py
              content:
                !inline_text "import codegen_sdk_pink"
    "#,
    start_point = "./test_data/"
))]
fn test_add_file(dirname: &str) -> Result<(), Error> {
    use std::thread::sleep;

    let dir = PathBuf::from(dirname);
    let mut codebase = Codebase::new(dir.clone());
    let new_file = dir.join("test2.py");
    std::fs::write(&new_file, "import codegen_sdk_pink").unwrap();
    log::info!("Added file at {}", new_file.display());
    assert_eq!(codebase.files().len(), 1);
    sleep(Duration::from_secs(5));
    log::info!("Checking update");
    codebase.check_update().unwrap();
    log::info!("Checking update done");
    assert_eq!(codebase.files().len(), 2);
    assert_eq!(codebase.files()[0].name(codebase.db()), "test2.py");
    assert_eq!(codebase.files()[1].name(codebase.db()), "test.py");

    log::info!("Finished!");
    Ok(())
}
#[cfg(feature = "python")]
#[test_log::test(rfs_test(
    config = r#"
    - !directory
        name: testc
        content:
          - !file
              name: test.py
              content:
                !inline_text "import codegen_sdk_pink"
    "#,
    start_point = "./test_data/"
))]
fn test_remove_file(dirname: &str) -> Result<(), Error> {
    let dir = PathBuf::from(dirname);
    let mut codebase = Codebase::new(dir.clone());
    let new_file = dir.join("test.py");
    assert_eq!(codebase.files().len(), 1);
    std::fs::remove_file(new_file).unwrap();
    log::info!("Removed file");
    sleep(Duration::from_secs(5));
    log::info!("Checking update");
    codebase.check_update().unwrap();
    log::info!("Checking update done");
    assert_eq!(codebase.files().len(), 0);
    assert_eq!(codebase.db().files().len(), 0);
    log::info!("Finished!");
    Ok(())
}
