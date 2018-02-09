extern crate includedir_codegen;

fn main() {
    println!("cargo:warning=\"Running build.rs...\"");
    println!("cargo:rerun-if-changed=data/shaders");
    includedir_codegen::start("FILES")
        .dir("data/shaders", includedir_codegen::Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
