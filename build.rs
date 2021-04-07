use std::{ env, fs };
use fs_extra::{copy_items, dir::CopyOptions};

fn copy_js_folder() {
    let mut copy_options = CopyOptions::new();
    let out_dir = env::var("OUT_DIR").unwrap();

    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    let mut docs_dir = out_dir;
    docs_dir.push_str("/../../../../doc");

    fs::create_dir_all(&docs_dir).unwrap();
    copy_items(&vec!["./doc/js/mermaid.min.js"], &docs_dir, &copy_options).unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=./doc/**");
    copy_js_folder();
}
