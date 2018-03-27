extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/signal.c")
        .compile("signal");
}
