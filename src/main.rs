mod convert;

use convert::convert;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    println!("{}", convert(path));
}
