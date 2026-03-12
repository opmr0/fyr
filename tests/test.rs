use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;

fn fyr() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_missing_watch_flag() {
    let output = fyr()
        .args(["-r", "echo hello"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("please provide paths with -w"));
}

#[test]
fn test_missing_run_flag() {
    let output = fyr().args(["-w", "src"]).output().expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("please provide a command with -r"));
}

#[test]
fn test_path_does_not_exist() {
    let output = fyr()
        .args(["-w", "this_path_does_not_exist", "-r", "echo hello"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("doesn't exist"));
}

#[test]
fn test_command_does_not_exist() {
    let output = fyr()
        .args(["-w", "src", "-r", "this_command_does_not_exist_xyz"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found"));
}

#[test]
fn test_task_add_and_list() {
    let add = fyr()
        .args([
            "task",
            "add",
            "test_task_list",
            "-w",
            "src",
            "-r",
            "echo hello",
        ])
        .output()
        .expect("failed to run");

    assert!(add.status.success());
    assert!(stdout(&add).contains("saved"));

    let list = fyr()
        .args(["task", "list"])
        .output()
        .expect("failed to run");

    assert!(list.status.success());
    assert!(stdout(&list).contains("test_task_list"));

    fyr()
        .args(["task", "remove", "test_task_list"])
        .output()
        .expect("failed to run");
}

#[test]
fn test_task_add_missing_watch() {
    let output = fyr()
        .args(["task", "add", "bad_task", "-r", "echo hello"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
}

#[test]
fn test_task_add_missing_run() {
    let output = fyr()
        .args(["task", "add", "bad_task", "-w", "src"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
}

#[test]
fn test_task_remove_existing() {
    fyr()
        .args([
            "task",
            "add",
            "test_task_remove",
            "-w",
            "src",
            "-r",
            "echo hi",
        ])
        .output()
        .expect("failed to run");

    let remove = fyr()
        .args(["task", "remove", "test_task_remove"])
        .output()
        .expect("failed to run");

    assert!(remove.status.success());
    assert!(stdout(&remove).contains("removed"));
}

#[test]
fn test_task_remove_not_found() {
    let output = fyr()
        .args(["task", "remove", "task_that_does_not_exist_xyz"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found"));
}

#[test]
fn test_task_edit_run() {
    fyr()
        .args([
            "task",
            "add",
            "test_task_edit",
            "-w",
            "src",
            "-r",
            "echo old",
        ])
        .output()
        .expect("failed to run");

    let edit = fyr()
        .args(["task", "edit", "test_task_edit", "-r", "echo new"])
        .output()
        .expect("failed to run");

    assert!(edit.status.success());
    assert!(stdout(&edit).contains("updated"));

    fyr()
        .args(["task", "remove", "test_task_edit"])
        .output()
        .expect("failed to run");
}

#[test]
fn test_task_edit_no_fields() {
    let output = fyr()
        .args(["task", "edit", "some_task"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
}

#[test]
fn test_task_edit_not_found() {
    let output = fyr()
        .args([
            "task",
            "edit",
            "task_that_does_not_exist_xyz",
            "-r",
            "echo hi",
        ])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found"));
}

#[test]
fn test_task_list_empty_message() {
    let output = fyr()
        .args(["task", "list"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
}

#[test]
fn test_run_task_not_found() {
    let output = fyr()
        .args(["run", "task_that_does_not_exist_xyz"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found"));
}

#[test]
fn test_run_global_flag() {
    let output = fyr()
        .args(["run", "task_that_does_not_exist_xyz", "--global"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found"));
}

#[test]
#[serial]
fn test_init_creates_file() {
    let existed = Path::new("fyr.toml").exists();
    if existed {
        fs::rename("fyr.toml", "fyr.toml.bak").ok();
    }

    let output = fyr().args(["init"]).output().expect("failed to run");

    assert!(output.status.success());
    assert!(Path::new("fyr.toml").exists());
    assert!(stdout(&output).contains("created"));

    fs::remove_file("fyr.toml").ok();
    if existed {
        fs::rename("fyr.toml.bak", "fyr.toml").ok();
    }
}

#[test]
#[serial]

fn test_init_already_exists() {
    fs::write("fyr.toml", "[tasks]").ok();

    let output = fyr().args(["init"]).output().expect("failed to run");

    assert!(output.status.success());
    assert!(stdout(&output).contains("already exists"));

    fs::remove_file("fyr.toml").ok();
}

#[test]
#[serial]
fn test_no_args_no_toml_no_global_flag() {
    let existed = Path::new("fyr.toml").exists();
    if existed {
        fs::rename("fyr.toml", "fyr.toml.bak").ok();
    }

    let output = fyr().output().expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("no 'fyr.toml' found"));

    if existed {
        fs::rename("fyr.toml.bak", "fyr.toml").ok();
    }
}

#[test]
fn test_task_rename() {
    fyr()
        .args([
            "task",
            "add",
            "test_task_rename",
            "-w",
            "src",
            "-r",
            "echo hi",
        ])
        .output()
        .expect("failed to run");

    let rename = fyr()
        .args(["task", "rename", "test_task_rename", "test_task_renamed"])
        .output()
        .expect("failed to run");

    assert!(rename.status.success());
    assert!(stdout(&rename).contains("renamed"));

    let list = fyr()
        .args(["task", "list"])
        .output()
        .expect("failed to run");

    assert!(stdout(&list).contains("test_task_renamed"));
    assert!(!stdout(&list).contains("test_task_rename "));

    fyr()
        .args(["task", "remove", "test_task_renamed"])
        .output()
        .expect("failed to run");
}

#[test]
fn test_task_rename_not_found() {
    let output = fyr()
        .args(["task", "rename", "task_that_does_not_exist_xyz", "new_name"])
        .output()
        .expect("failed to run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("not found"));
}
