mod instruction;
mod x86_64_fasm;
pub use instruction::*;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::fs::File;
pub use x86_64_fasm::*;

pub trait CodeGen {
    fn encode_name(label: &str) -> String;

    fn instruction(
        out: &mut BufWriter<File>,
        label_hash: &str,
        instruction: &Instruction,
    ) -> Result<(), std::io::Error>;

    fn open(out: &mut BufWriter<File>) -> Result<(), std::io::Error>;
    fn generate_function(
        out: &mut BufWriter<File>,
        label: &str,
        instructions: &[Instruction],
    ) -> Result<(), std::io::Error>;
    fn entry(out: &mut BufWriter<File>) -> Result<(), std::io::Error>;
    fn init_data(
        out: &mut BufWriter<File>,
        data: &HashMap<String, InitData>,
    ) -> Result<(), std::io::Error>;
    fn uninit_data(
        out: &mut BufWriter<File>,
        data: &HashMap<String, UninitData>,
    ) -> Result<(), std::io::Error>;
}

pub fn compile<B: CodeGen>(
    file: &str,
    functions: &Vec<(&str, Vec<Instruction>)>,
    init_data: &HashMap<String, InitData>,
    uninit_data: &HashMap<String, UninitData>,
) -> Result<(), std::io::Error> {
    let file = std::fs::File::create(file).unwrap();
    let mut out = std::io::BufWriter::new(file);

    B::open(&mut out)?;
    for (label, instructions) in functions {
        B::generate_function(&mut out, label, instructions)?;
    }
    B::entry(&mut out)?;
    B::init_data(&mut out, init_data)?;
    B::uninit_data(&mut out, uninit_data)?;

    out.flush()?;

    Ok(())
}
