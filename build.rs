use std::{ env, fs };

fn copy_mermaid_js() {
    let mut docs_dir = env::var("OUT_DIR").unwrap();
    docs_dir.push_str("/../../../../doc");

    fs::create_dir_all(&docs_dir).unwrap();

    let mut mermaid_dst_file = docs_dir;
    mermaid_dst_file.push_str("/mermaid.min.js");

    fs::copy(&"./doc/js/mermaid.min.js", &mermaid_dst_file).unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=./doc/**");
    copy_mermaid_js();
}
