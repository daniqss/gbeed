use gbeed::prelude::*;

fn main() -> Result<()> {
    let file_name = std::env::args()
        .nth(1)
        .ok_or_else(|| Error::Generic("First argument must be a Gameboy ROM".to_owned()))?;

    let file = std::fs::File::open(file_name)?;

    gbeed::run(file)
}
