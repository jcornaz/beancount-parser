use version_check as rustc;

fn main() {
    if let Some(true) = rustc::supports_feature("doc_auto_cfg") {
        println!("cargo:rustc-cfg=has_doc_auto_cfg");
    }
}
