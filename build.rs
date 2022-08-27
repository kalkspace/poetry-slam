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
use std::collections::BTreeSet;
lazy_static! {
    pub static ref TRAINING_SETS: BTreeSet<&'static str> =
        BTreeSet::from([
"#
            .as_bytes(),
        )
        .unwrap();
    for entry in fs::read_dir("assets/data").unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_string_lossy().ends_with(".txt")
        {
            let filename = entry.file_name().to_string_lossy().to_string();
            target
                .write(
                    format!(
                        "// {}\nstd::str::from_utf8({:?}.as_slice()).unwrap(),\n",
                        filename,
                        &filename[..filename.len() - 4].as_bytes(),
                    )
                    .as_bytes(),
                )
                .unwrap();
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

    println!("cargo:rerun-if-changed=data");
    println!("cargo:rerun-if-changed=build.rs");
}
