pub struct X86_64;
use std::io::{BufWriter, Write};
use std::fs::File;

use crate::lex::token::Operator;

use super::{InitData, Instruction, UninitData};

impl super::CodeGen for X86_64 {
    fn encode_name(label: &str) -> String {
        let mut s = String::from("fn_");
        label.chars().for_each(|c| match c {
            '.' => s.push_str("_dot_"),
            '<' => s.push('_'),
            '>' => s.push('_'),
            '+' => s.push_str("_plus_"),
            ' ' => s.push('_'),
            '*' => s.push_str("_star_"),
            '&' => s.push_str("_amp_"),
            '_' => s.push_str("__"),
            ':' => s.push('_'),
            '[' => s.push_str("_lbrace_"),
            ']' => s.push_str("_rbrace_"),
            c => s.push(c),
        });
        s
    }

    fn instruction(
        out: &mut BufWriter<File>,
        hash: &str,
        instruction: &Instruction,
    ) -> Result<(), std::io::Error> {
        writeln!(out, "  ; -- {instruction:?}")?;
        match instruction {
            Instruction::FrameReserve { bytes } => {
                writeln!(out, "  sub qword [frame_end_ptr], {bytes}")?;
            }
            Instruction::PushFromFrame {
                offset_from_end,
                bytes,
            } => {
                writeln!(out, "  mov rax, [frame_end_ptr]")?;
                writeln!(out, "  add rax, {}", offset_from_end * 8)?;
                for _ in 0..*bytes {
                    writeln!(out, "  mov rbx, [rax]")?;
                    writeln!(out, "  push rbx")?;
                    writeln!(out, "  add rax, 8")?;
                }
            }
            Instruction::PushPtrToFrame {
                offset_from_end, ..
            } => {
                writeln!(out, "  mov rax, [frame_end_ptr]")?;
                writeln!(out, "  add rax, {}", offset_from_end * 8)?;
                writeln!(out, "  push rax")?;
            }
            Instruction::PushToFrame { quad_words } => {
                for _ in 0..*quad_words {
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  sub  qword [frame_end_ptr], 8")?;
                    writeln!(out, "  mov  rbx, [frame_end_ptr]")?;
                    writeln!(out, "  mov  [rbx], rax")?;
                }
            }
            Instruction::FramePtrToFrameReserve {
                offset,
                size,
                width,
            } => {
                writeln!(out, "  mov  rax, [frame_start_ptr]")?;
                if *size > 0 {
                    writeln!(out, "  sub  rax, {}", offset + 16 + ((size - 1) * width))?;
                }

                writeln!(out, "  sub  qword [frame_end_ptr], 8")?;
                writeln!(out, "  mov  rbx, [frame_end_ptr]")?;
                writeln!(out, "  mov  [rbx], rax")?;
            }
            Instruction::StartBlock => (),
            Instruction::EndBlock { bytes_to_free } => {
                writeln!(out, "  add qword [frame_end_ptr], {bytes_to_free}")?;
            }
            Instruction::Operator { op, size: None } => match op {
                Operator::Plus => {
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  add  rax, rbx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Minus => {
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  sub  rax, rbx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Star => {
                    writeln!(out, "  pop  rcx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  mul  rcx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Slash => {
                    writeln!(out, "  mov  rdx, 0")?;
                    writeln!(out, "  pop  rcx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  div  rcx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Modulo => {
                    writeln!(out, "  mov  rdx, 0")?;
                    writeln!(out, "  pop  rcx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  div  rcx")?;
                    writeln!(out, "  push rdx")?;
                }
                Operator::GreaterEqual => {
                    writeln!(out, "  mov  rcx, 0")?;
                    writeln!(out, "  mov  rdx, 1")?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  cmp  rax, rbx")?;
                    writeln!(out, "  cmovge rcx, rdx")?;
                    writeln!(out, "  push rcx")?;
                }
                Operator::GreaterThan => {
                    writeln!(out, "  mov  rcx, 0")?;
                    writeln!(out, "  mov  rdx, 1")?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  cmp  rax, rbx")?;
                    writeln!(out, "  cmovg rcx, rdx")?;
                    writeln!(out, "  push rcx")?;
                }
                Operator::LessThan => {
                    writeln!(out, "  mov  rcx, 0")?;
                    writeln!(out, "  mov  rdx, 1")?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  cmp  rax, rbx")?;
                    writeln!(out, "  cmovl rcx, rdx")?;
                    writeln!(out, "  push rcx")?;
                }
                Operator::LessEqual => {
                    writeln!(out, "  mov  rcx, 0")?;
                    writeln!(out, "  mov  rdx, 1")?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  cmp  rax, rbx")?;
                    writeln!(out, "  cmovle rcx, rdx")?;
                    writeln!(out, "  push rcx")?;
                }
                Operator::Equal => {
                    writeln!(out, "  mov  rcx, 0")?;
                    writeln!(out, "  mov  rdx, 1")?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  cmp  rax, rbx")?;
                    writeln!(out, "  cmove rcx, rdx")?;
                    writeln!(out, "  push rcx")?;
                }
                Operator::BangEqual => {
                    writeln!(out, "  mov  rcx, 0")?;
                    writeln!(out, "  mov  rdx, 1")?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  cmp  rax, rbx")?;
                    writeln!(out, "  cmovne rcx, rdx")?;
                    writeln!(out, "  push rcx")?;
                }
                Operator::Ampersand => {
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  and  rax, rbx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Pipe => {
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  or   rax, rbx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Caret => {
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  xor  rax, rbx")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::ShiftLeft => {
                    writeln!(out, "  pop  rcx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  shl  rax, cl")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::ShiftRight => {
                    writeln!(out, "  pop  rcx")?;
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  shr  rax, cl")?;
                    writeln!(out, "  push rax")?;
                }
                Operator::Unary { .. } => unreachable!(
                    "Unary expressions should have been converted into other instructions",
                ),
                Operator::Read | Operator::Write => unreachable!(),
            },
            Instruction::Operator {
                op,
                size: Some((size, width)),
            } => match op {
                Operator::Read => {
                    if *size == 0 {
                        return Ok(());
                    }

                    let register = match width {
                        1 => "bl",
                        2 => "bx",
                        3 => "ebx",
                        8 => "rbx",
                        w => unreachable!("n: {w}"),
                    };
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  mov  rbx, 0")?;
                    writeln!(out, "  mov  {register}, [rax]")?;
                    writeln!(out, "  push rbx")?;
                    for _ in 1..*size {
                        writeln!(out, "  add  rax, {width}")?;
                        writeln!(out, "  mov  rbx, 0")?;
                        writeln!(out, "  mov  {register}, [rax]")?;
                        writeln!(out, "  push rbx")?;
                    }
                }
                Operator::Write => {
                    if *size == 0 {
                        return Ok(());
                    }

                    let register = match width {
                        1 => "bl",
                        2 => "bx",
                        4 => "ebx",
                        8 => "rbx",
                        _ => unreachable!(),
                    };
                    writeln!(out, "  pop  rax")?;
                    writeln!(out, "  add  rax, {}", (*size - 1) * width)?;
                    writeln!(out, "  pop  rbx")?;
                    writeln!(out, "  mov  [rax], {register}")?;
                    for _ in 1..*size {
                        writeln!(out, "  sub  rax, {width}")?;
                        writeln!(out, "  pop  rbx")?;
                        writeln!(out, "  mov  [rax], {register}")?;
                    }
                }
                _ => unreachable!(),
            },
            Instruction::Jump { dest_id } => writeln!(out, "  jmp {hash}_jmp_dest_{dest_id}")?,
            Instruction::JumpFalse { dest_id } => {
                writeln!(out, "  pop  rax")?;
                writeln!(out, "  test rax, rax")?;
                writeln!(out, "  jz {hash}_jmp_dest_{dest_id}")?;
            }
            Instruction::JumpDest { id } => writeln!(out, "{hash}_jmp_dest_{id}:")?,
            Instruction::PushU64(n) => writeln!(out, "  push {n}")?,
            Instruction::PushGlobal { id } => writeln!(out, "  push {id}")?,
            Instruction::Call(func) => {
                let hash = X86_64::encode_name(func);
                writeln!(out, "  mov rax, [frame_start_ptr]")?;
                writeln!(out, "  sub  qword [frame_end_ptr], 8")?;
                writeln!(out, "  mov  rbx, [frame_end_ptr]")?;
                writeln!(out, "  mov  [rbx], rax")?;
                writeln!(out, "  mov  rax, [frame_end_ptr]")?;
                writeln!(out, "  mov  qword [frame_start_ptr], rax")?;
                writeln!(out, "  call {hash} ; -- {func}")?;
            }
            Instruction::InitLocalVarArr {
                offset_to_var,
                offset_to_data,
                data_size,
                data_width,
            } => {
                writeln!(out, "  mov rax, [frame_start_ptr]")?;
                writeln!(out, "  sub rax, {}", offset_to_var + 16)?;
                writeln!(out, "  mov rbx, [frame_start_ptr]")?;
                writeln!(
                    out,
                    "  sub rbx, {}",
                    offset_to_data + 8 + data_size * data_width
                )?;
                writeln!(out, "  mov [rax], rbx")?;
                writeln!(out, "  sub rax, 8")?;
                writeln!(out, "  mov rbx, {data_size}")?;
                writeln!(out, "  mov [rax], rbx")?;
            }
            Instruction::Syscall(n) => {
                let order = ["rax", "rdi", "rsi", "rdx", "r10", "r8", "r9"];
                for register in order.iter().take(*n + 1) {
                    writeln!(out, "  pop  {register}").unwrap();
                }
                writeln!(out, "  syscall").unwrap();
                writeln!(out, "  push rax").unwrap();
            }
            Instruction::Return => {
                writeln!(out, "  mov  rax, [frame_start_ptr]")?;
                writeln!(out, "  push qword [rax-8]")?;
                writeln!(out, "  mov  [frame_end_ptr], rax")?;
                writeln!(out, "  add  qword [frame_end_ptr], 8")?;
                writeln!(out, "  mov  rax, [rax]")?;
                writeln!(out, "  mov  qword [frame_start_ptr], rax")?;
                writeln!(out, "  ret")?;
            }
        }

        Ok(())
    }

    fn generate_function(
        out: &mut BufWriter<File>,
        label: &str,
        instructions: &[super::Instruction],
    ) -> Result<(), std::io::Error> {
        let hash = X86_64::encode_name(label);
        writeln!(out, "; fn {label}")?;
        writeln!(out, "{hash}:")?;
        // Save the return address
        writeln!(out, "  pop rax")?;
        writeln!(out, "  sub  qword [frame_end_ptr], 8")?;
        writeln!(out, "  mov  rbx, [frame_end_ptr]")?;
        writeln!(out, "  mov  [rbx], rax")?;

        for i in instructions {
            X86_64::instruction(out, hash.as_str(), i)?;
        }

        Ok(())
    }

    fn open(out: &mut BufWriter<File>) -> Result<(), std::io::Error> {
        writeln!(out, "format ELF64 executable")?;
        writeln!(out, "segment readable executable")
    }

    fn entry(out: &mut BufWriter<File>) -> Result<(), std::io::Error> {
        writeln!(out, "entry _start")?;
        writeln!(out, "_start: ")?;
        writeln!(out, "  mov  qword [frame_start_ptr], frame_stack_end")?;
        writeln!(out, "  mov  qword [frame_end_ptr], frame_stack_end")?;

        let call = Instruction::Call(String::from("main"));
        X86_64::instruction(out, "", &call)?;

        writeln!(out, "exit:")?;
        writeln!(out, "  mov  rax, 60")?;
        writeln!(out, "  mov  rdi, 0")?;
        writeln!(out, "  syscall")?;

        Ok(())
    }

    fn init_data(
        out: &mut BufWriter<File>,
        data: &std::collections::HashMap<String, InitData>,
    ) -> Result<(), std::io::Error> {
        writeln!(out)?;
        writeln!(out, "segment readable writeable")?;
        for (id, data) in data {
            match data {
                InitData::String(s) => {
                    write!(out, "  {id}: db ")?;
                    if !s.is_empty() {
                        for (i, b) in s.as_bytes().iter().enumerate() {
                            if i > 0 { write!(out, ", ")? }
                            write!(out, "{b:#x}")?;
                        }
                    } else {
                        write!(out, "0x00")?;
                    }
                }
                InitData::Arr { size, pointer } => {
                    write!(out, "  {id}: dq {size}, {pointer}").unwrap()
                }
            }
            writeln!(out)?;
        }

        Ok(())
    }

    fn uninit_data(
        out: &mut BufWriter<File>,
        data: &std::collections::HashMap<String, UninitData>,
    ) -> Result<(), std::io::Error> {
        writeln!(out, "segment readable writeable")?;
        writeln!(out, "  frame_start_ptr: rq 1")?;
        writeln!(out, "  frame_end_ptr: rq 1")?;
        writeln!(out, "  frame_stack: rq 1048576")?;
        writeln!(out, "  frame_stack_end:")?;

        for (id, data) in data {
            match data {
                UninitData::Region(size) => writeln!(out, "  {id}: rq {size}")?,
            }
        }

        Ok(())
    }
}
