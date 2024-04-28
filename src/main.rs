fn main() {
    "Hello, world!"
        .chars()
        .chain(std::iter::once('\n'))
        .for_each(|c| print!("{c}"));
}
