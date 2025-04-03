fn main() {
    let crate_dir = env!("CARGO_MANIFEST_DIR");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("../include/ksc/_libkslang_autogen.h");
}
