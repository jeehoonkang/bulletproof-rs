extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/impl.c")
        .compile("impl");
}
