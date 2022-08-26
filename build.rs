use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data.rs");

    let mut target = fs::File::create(&dest_path).unwrap();
    target
        .write(
            r#"
use lazy_static::lazy_static;
use std::collections::BTreeMap;
lazy_static! {
    pub static ref TRAINING_SETS: BTreeMap<&'static str, &'static str> =
        BTreeMap::from([
"#
            .as_bytes(),
        )
        .unwrap();
    for entry in fs::read_dir("data").unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry
                .file_name()
                .to_ascii_lowercase()
                .to_string_lossy()
                .ends_with(".txt")
        {
            let filename = entry.file_name().to_string_lossy().to_string();
            let content = fs::read_to_string(entry.path()).unwrap();
            target
                .write(
                    format!(
                        "\n({:?}, std::str::from_utf8({:?}.as_slice()).unwrap()),\n",
                        &filename[..filename.len() - 4],
                        content.as_bytes(),
                    )
                    .as_bytes(),
                )
                .unwrap();

            println!("cargo:rerun-if-changed={}", entry.path().to_string_lossy());
        }
    }
    target
        .write(
            r#"
            ]
        );
    }
"#
            .as_bytes(),
        )
        .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
