use std::process::Command;

pub fn build_patches(tag: String, ver: String) {
    println!("Building patch. tag: {} ver: {}", tag, ver);
    let mut cmd = Command::new("git")
        .args(["checkout", &tag])
        .current_dir("fp")
        .spawn()
        .expect("Failed to execute command");

    cmd.wait().unwrap();

    let mut cmd = Command::new("bash")
        .args(["./makerom", &format!("../roms/{}.z64", ver.to_lowercase())])
        .current_dir("fp")
        .spawn()
        .expect("Failed to execute command");

    cmd.wait().unwrap();

    std::fs::create_dir_all(format!("./patches/{}/", tag)).unwrap();

    let mut cmd = Command::new("xdelta3")
        .args([
            "-e",
            "-S",
            "none",
            "-s",
            &format!("./roms/{}.z64", ver.to_lowercase()),
            &format!("./fp/fp-{}.z64", ver.to_lowercase()),
            &format!("./patches/{}/{}.xdelta", tag, ver.to_lowercase()),
        ])
        .spawn()
        .expect("Failed to execute command");
    cmd.wait().unwrap();
}
