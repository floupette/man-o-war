use std::fs;

use rustdoc_parser::process_html;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/home/floupette/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/std/result/enum.Result.html";
    // let path = "/home/floupette/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/std/iter/trait.Iterator.html";
    // let path = "/home/floupette/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/std/marker/trait.Sized.html";
    // let path = "//home/floupette/Projects/rocketman/target/doc/rocket/struct.Catcher.html";
    let file = fs::read_to_string(path)?;
    process_html(&file)?;

    Ok(())
}
