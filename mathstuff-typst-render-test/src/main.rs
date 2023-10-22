use std::error::Error;

use std::fs::{self, File};
use std::io::BufWriter;

use mathstuff_typst::{MyWorld, PREAMBLE, write_image};

use mathstuff::parse::parse_into_expression;
use mathstuff::print::print_expr_to_string;

fn new_file(path: &str) -> Result<BufWriter<File>, Box<dyn Error>> {
    Ok(BufWriter::new(File::create(path)?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let eq = parse_into_expression(&fs::read_to_string("./test.txt")?).unwrap();
    dbg!(&eq);
    let str = print_expr_to_string(&eq);
    let mut world = MyWorld::new(format!("{PREAMBLE} ${str}$"))?;

    write_image(&world, &mut new_file("./out.png")?)?;

    let s = eq.simplify().unwrap();
    dbg!(s.as_inner());
    let str = print_expr_to_string(s.as_inner());

    world.set_source(format!("{PREAMBLE} ${str}$"));

    write_image(&world, &mut new_file("./out_simplified.png")?)?;

    Ok(())
}
