use cc;

fn main() {
    cc::Build::new()
        .file("src/version.c")
        .compile("version");
}
