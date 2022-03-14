use std::fs::File;
use std::io::{Read, Write};
use crate::opcode::Opcode;
use crate::value::{ObjectValue, Value};


#[derive(Debug, Clone)]
pub struct Chunk {
    pub op_codes: Vec<Opcode>,
    pub constants: Vec<Value>
}


impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            op_codes: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn size_to_bytes(s: usize) -> [u8; 8] {
        usize::to_le_bytes(s)
    }

    pub fn bytes_to_usize(reader:   &mut File) -> usize{
        let mut buffer = [0_u8; std::mem::size_of::<usize>()];
        reader.read(&mut buffer);
        usize::from_le_bytes(buffer)
    }


    pub fn to_bytes(&self, file: &mut File) ->std::io::Result<()> {

        // write constant pool

        // size of pool
        file.write(&[self.constants.len() as u8]);

        //
        self.constants.iter().for_each(|c| {

            // write value type (u8)
            // write size of value (usize)
            // write byte values
            match c {
                Value::Boolean(b) => {

                    file.write(&[1]);   // t
                    file.write(&[if *b  { 1} else {0} ]); // bytes
                }
                Value::Nil => {
                    file.write(&[2]);   // type
                }
                Value::Number(d) => {
                    file.write(&[3]);   // type
                    file.write(& d.to_le_bytes());
                }
                Value::Object(obj) => {
                    match obj { ObjectValue::String(s) => {
                        file.write(&[4]);
                        let str_bytes = s.as_bytes();
                        file.write(&Chunk::size_to_bytes(str_bytes.len()));
                        file.write(str_bytes);
                    }
                        ObjectValue::Function(_) => {
                            todo!("serialize funtcion to bytes");
                        }
                    }
                }
            }

        });

        file.flush();



        // write chunks
        self.op_codes.iter().for_each(|opcode|  {
            let v: Vec<u8> =  opcode.into();
            let s = v.as_slice();
            let n = file.write(s);
        });
        Ok(())
    }

    pub fn from_bytes( file: &mut File)-> Chunk {

        let mut buff=  [0u8;1];
        file.read(&mut buff);
        let mut constant_pool_len = buff[0] as i8;

        let mut constants: Vec<Value> =  Vec::new();
        while constant_pool_len > 0 {

            // read type
            file.read(&mut buff);
            let value = match buff[0] {

                // Boolean
                1 => {
                    // read 0 or 1
                    file.read(&mut buff);
                    Value::Boolean(buff[0] == 1)
                }

                // nil
                2 => {
                    Value::Nil
                }
                // number
                3 => {
                    let mut buff_f64=  [0u8;8];
                    file.read(&mut buff_f64);
                    Value::Number(f64::from_le_bytes(buff_f64))
                }
                // string
                4 => {
                    let len =  Chunk::bytes_to_usize(file);
                    let mut buff_f64=  Vec::with_capacity(len);
                    unsafe {
                        buff_f64.set_len(len);
                    }
                    file.read(buff_f64.as_mut_slice());
                    let s = String::from_utf8(buff_f64).ok().unwrap();
                    Value::Object(ObjectValue::String(s))
                }
                x => panic!("Unknown type {}", x)
            };

            constants.push(value);


            constant_pool_len = constant_pool_len -  1;
        }


        let mut op_codes = Vec::new();
        while let Some(opcode) = Opcode::from_file(file) {
            op_codes.push(opcode);
        }
        Chunk {
            op_codes: op_codes,
            constants: constants
        }
    }

}


pub trait ChunkWriterTrait {
    ///
    ///
    fn emit_byte(&mut self, byte: Opcode, line: isize);
    ///
    ///
    fn emit_bytes(&mut self, byte1: Opcode, byte2: Opcode, line: isize);
    ///
    ///
    fn emit_return(&mut self, line: isize);
    ///
    ///
    fn emit_constant(&mut self, value: Value, line: isize);
    fn write_chunk(&mut self, byte: Opcode, _line: isize);
    fn make_constant(&mut self, value: Value) -> usize;
    fn disassemble_chunk(&mut self, writer: &mut Box<dyn Write>);
    fn len(&self) -> usize;
    fn replace_opcode(&mut self, index: usize, bytes: Opcode);
}



pub struct ChunkOpCodeReader<'s> {
    op_codes: &'s[Opcode],
    ip: usize
}

impl<'s> ChunkOpCodeReader<'s>
{
    pub fn new(op_codes: &'s[Opcode]) -> Self {
        Self { op_codes, ip: 0 }
    }
    pub fn prev(&mut self) {
        self.ip -= 1;
    }

    pub fn read_slice(&mut self, n: usize) -> &[Opcode] {
        let start = self.ip + 1;
        let end  = start +  n;
        &self.op_codes[start..end]
    }

}
impl<'s> Iterator for ChunkOpCodeReader<'s> {
    type Item = &'s Opcode;
    fn next(&mut self) -> Option<Self::Item> {
        let ip = self.ip;
        if ip < self.op_codes.len() {
            self.ip += 1;
            self.op_codes.get(ip)
        } else {
            None
        }
    }
}


/*
pub trait WritableChunk {
    fn write_chunk(&mut self, bytes: Opcode);
    fn replace_opcode(&mut self, index: usize, bytes: Opcode);
    fn add_constant(&mut self, value: Value) -> usize;
    fn read_constant(&self, index : usize) -> Option<&Value>;
    fn disassemble_chunk(&mut self, writer: &mut dyn Write);
    fn disassemble_instruction(&mut self, offset: usize, writer: &mut dyn Write) -> usize;
    fn simple_instruction(&mut self, name: &str, offset: usize, writer: &mut dyn Write) -> usize;
    fn constant_instruction(&mut self, name: &str, offset: usize, const_idx: usize, writer: &mut dyn Write) -> usize;
    fn byte_instruction(&mut self, name: &str, offset: usize, const_idx: usize, writer: &mut dyn Write) -> usize;
    fn jump_instruction(&mut self, name: &str, offset: usize, sign: isize, jump: &u16, writer: &mut dyn Write) -> usize;
}
*/


impl Chunk   {

    pub(crate) fn write_chunk(&mut self, bytes: Opcode) {
       self.op_codes.push(bytes);
    }

    pub(crate) fn replace_opcode(&mut self, index: usize, bytes: Opcode) {
        std::mem::replace(&mut self.op_codes[index], bytes);
    }


    pub(crate) fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub(crate) fn read_constant(&self, index : usize) -> Option<&Value>{
        self.constants.get(index)
    }

    pub(crate) fn disassemble_chunk(&mut self, writer: &mut dyn Write) {
        let mut offset: usize = 0;
        while offset < self.op_codes.len() {
            offset = self.disassemble_instruction(offset, writer);
        }
    }

    fn disassemble_instruction(&mut self, offset: usize, writer: &mut dyn Write) -> usize {
        write!(writer, "{:04} ", offset);
        let opcode = self.op_codes.get(offset).unwrap();
        match opcode {
            Opcode::OpReturn => {
                Chunk::simple_instruction("OP_RETURN", offset, writer)
            },
            Opcode::OpNegate => {
                Chunk::simple_instruction("OP_NEGATE", offset, writer)
            },
            Opcode::OpNot => {
                Chunk::simple_instruction("OP_NOT", offset, writer)
            },
            Opcode::OpConstant(size) => {
                self.constant_instruction("OP_CONSTANT", offset, *size, writer)
            },
            Opcode::OpDefineGlobal(size) => {
                self.constant_instruction("OP_DEFINE_GLOBAL", offset, *size, writer)
            },
            Opcode::OpGetGlobal(size) => {
                self.constant_instruction("OP_GET_GLOBAL", offset, *size, writer)
            },
            Opcode::OpSetGlobal(size) => {
                self.constant_instruction("OP_SET_GLOBAL", offset, *size, writer)
            },

            Opcode::OpSetLocal(size) => {
                self.byte_instruction("OP_GET_LOCAL", offset, *size, writer)
            },
            Opcode::OpGetLocal(size) => {
                self.byte_instruction("OP_SET_LOCAL", offset, *size, writer)
            },
            Opcode::OpAdd => {
                Chunk::simple_instruction("OP_ADD", offset, writer)
            },
            Opcode::OPSubtract => {
                Chunk::simple_instruction("OP_SUBTRACT", offset, writer)
            },
            Opcode::OPMultiply => {
                Chunk::simple_instruction("OP_MULTIPLY", offset, writer)
            },
            Opcode::OpDivide => {
                Chunk::simple_instruction("OP_DIVIDE", offset, writer)
            },
            Opcode::OpFalse =>  Chunk::simple_instruction("OP_FALSE", offset, writer),
            Opcode::OpNil=>  Chunk::simple_instruction("OP_NIL", offset, writer),
            Opcode::OpTrue =>  Chunk::simple_instruction("OP_TRUE", offset, writer),

            Opcode::OpEqual =>  Chunk::simple_instruction("OP_EQUAL", offset, writer),
            Opcode::OpGreater =>  Chunk::simple_instruction("OP_GREATER", offset, writer),
            Opcode::OpLess =>  Chunk::simple_instruction("OP_LESS", offset, writer),
            Opcode::OpPrint =>  Chunk::simple_instruction("OP_PRINT", offset, writer),
            Opcode::OpPop =>  Chunk::simple_instruction("OP_POP", offset, writer),


            Opcode::OpJumpIfFalse(jump) => {
                Chunk::jump_instruction("OP_JUMP_IF_FALSE", offset, 1, jump, writer)
            },
            Opcode::OpJump(jump) => {
                Chunk::jump_instruction("OP_JUMP", offset, 1, jump, writer)
            },
            Opcode::OpLoop(jump) => {
                Chunk::jump_instruction("OP_LOOP", offset, -1, jump, writer)
            },
            _ => {
                offset + 1
            }
        }
    }

    fn simple_instruction(name: &str, offset: usize, writer: &mut dyn Write) -> usize {
        write!(writer, "{: <12}\n", name);
        offset + 1
    }

    fn constant_instruction(&mut self, name: &str, offset: usize, const_idx: usize, writer: &mut dyn Write) -> usize {
        let value = self.constants.get(const_idx).unwrap();
        write!(writer, "{: <12} {} '{}' \n", name, const_idx, value);
        offset + 1
    }

    fn byte_instruction(&mut self, name: &str, offset: usize, const_idx: usize, writer: &mut dyn Write) -> usize {
        let op_code = self.op_codes.get(offset ).unwrap();
        match op_code {
            Opcode::OpGetLocal(idx) => {
                write!(writer, "{: <12} {}  \n", name, idx);
            }
            Opcode::OpSetLocal(idx) => {
                write!(writer, "{: <12} {}  \n", name, idx);
            }
            _ => {
                panic!("INVALID")
            }
        }

        offset + 2
    }

    fn jump_instruction(name: &str, offset: usize, sign: isize, jump:&u16, writer: &mut dyn Write) -> usize {
        write!(writer, "{: <12} {} -> {}\n", name, offset, offset as i32  +1 + sign as i32 * (*jump as i32));
        offset + 1
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::{Read, Write};
    use crate::chunk::{Chunk};
    use crate::opcode::Opcode;
    use crate::value::{ObjectValue, Value};
    use crate::vm::VM;

    #[test]
    fn negate() {
        let mut chunk : Chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Number(3.14));

        chunk.write_chunk(Opcode::OpConstant(idx));
        chunk.write_chunk(Opcode::OpNegate);
        chunk.write_chunk(Opcode::OpReturn);

       chunk.disassemble_chunk(&mut (Box::new(io::stdout()) as Box<dyn Write>));

        let mut vm = VM::new();
        vm.run(&chunk);

    }

    #[test]
    fn basic_sum() {
        let mut chunk : Chunk = Chunk::new();

        let mut constant = chunk.add_constant(Value::Number(1.2));
        chunk.write_chunk(Opcode::OpConstant(constant));

        constant = chunk.add_constant(Value::Number(3.4));
        chunk.write_chunk(Opcode::OpConstant(constant));

        chunk.write_chunk(Opcode::OpAdd);

        constant = chunk.add_constant(Value::Number(5.6));
        chunk.write_chunk(Opcode::OpConstant(constant));

        chunk.write_chunk(Opcode::OpDivide);
        chunk.write_chunk(Opcode::OpNegate);
        chunk.write_chunk(Opcode::OpReturn);



        chunk.disassemble_chunk(&mut (Box::new(io::stdout()) as Box<dyn Write>));

        let mut vm = VM::new();
        vm.run(&chunk);

    }


    #[test]
    fn write_bytes() {
        let mut chunk: Chunk = Chunk::new();
        // chunk.write_chunk(Opcode::OpDivide);
        // chunk.write_chunk(Opcode::OpNegate);
        chunk.write_chunk(Opcode::OpJump(99));
        let mut idx;

        idx =  chunk.add_constant(Value::Boolean(true));
        chunk.write_chunk(Opcode::OpConstant(idx));
        //
        idx =  chunk.add_constant(Value::Number(1.2));
        chunk.write_chunk(Opcode::OpConstant(idx));

        idx = chunk.add_constant(Value::Nil);
        chunk.write_chunk(Opcode::OpConstant(idx));

        idx = chunk.add_constant(Value::Object(ObjectValue::String("hello".to_string())));
        chunk.write_chunk(Opcode::OpConstant(idx));

        let mut file = File::create("foo.txt").unwrap();
        chunk.to_bytes(&mut file);
        file.flush();


        let mut file1 = File::open("foo.txt").unwrap();
       // let mut v = Vec::new();
        // let mut buff = [0u8;1];
        // file1.read(&mut buff);
        // file1.read_to_end(&mut v);
        let chunk1 = Chunk::from_bytes(&mut file1);


        let a = 2;
    }

}
