use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_litepack"))
}

fn temp(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("litepack-cli-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn pack_list_unpack_zip_roundtrip() {
    let root = temp("zip");
    let data = root.join("data");
    std::fs::create_dir_all(data.join("sub")).unwrap();
    std::fs::write(data.join("a.txt"), "你好 cli").unwrap();
    std::fs::write(data.join("sub/b.txt"), b"nested").unwrap();
    let archive = root.join("out.zip");

    let out = bin().args(["pack", data.to_str().unwrap(), "-o", archive.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(archive.exists());

    let out = bin().args(["list", archive.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("data/a.txt"), "{stdout}");
    assert!(stdout.contains("data/sub/b.txt"), "{stdout}");

    let dest = root.join("x");
    let out = bin().args(["unpack", archive.to_str().unwrap(), "-d", dest.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(std::fs::read_to_string(dest.join("data/a.txt")).unwrap(), "你好 cli");
}

#[test]
fn pack_unpack_7z_with_password() {
    let root = temp("7z");
    let f = root.join("secret.txt");
    std::fs::write(&f, "top secret").unwrap();
    let archive = root.join("enc.7z");

    let out = bin().args(["pack", f.to_str().unwrap(), "-o", archive.to_str().unwrap(), "--password", "pass123"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

    let dest = root.join("ok");
    let out = bin().args(["unpack", archive.to_str().unwrap(), "-d", dest.to_str().unwrap(), "--password", "pass123"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(std::fs::read_to_string(dest.join("secret.txt")).unwrap(), "top secret");

    let dest2 = root.join("bad");
    let out = bin().args(["unpack", archive.to_str().unwrap(), "-d", dest2.to_str().unwrap(), "--password", "wrong"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn errors_use_exit_code_2() {
    let root = temp("err");
    let out = bin().args(["list", root.join("missing.zip").to_str().unwrap()]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));

    let out = bin().args(["pack", root.to_str().unwrap(), "-o", root.join("x.rar").to_str().unwrap()]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
