mod bf_tape {

    pub struct Tape<T> {
        data: Vec<T>,
        ptr: usize,
    }

    impl<T: Copy + Clone + Default> Tape<T> {
        fn allocate_for_ptr(&mut self, ptr_pos: usize) {
            // Ensure Tape.data has at least the size of ptr, resizing if necessary.
            // This function allocates for pos + 1000 to avoid excessive calls to Vec.resize().
            if self.data.len() < ptr_pos + 1 {
                self.data.resize(ptr_pos + 1000, T::default());
            }
        }

        pub fn get(&self) -> T {
            // Get the byte where the tape head points
            self.data[self.ptr]
        }

        pub fn new() -> Tape<T> {
            let mut tape = Tape {
                data: Vec::new(),
                ptr: 0,
            };
            tape.allocate_for_ptr(0);
            tape
        }

        pub fn next(&mut self) -> T {
            let elem = self.get();
            self.seek(1).expect("Couldn't seek tape by +1");
            elem
        }

        pub fn pos(&self) -> usize {
            self.ptr
        }

        pub fn seek(&mut self, offset: i32) -> Result<(), &str> {
            let new_ptr_pos = self.ptr as i32 + offset;
            if new_ptr_pos < 0 {
                return Err("Attempted to move tape head below 0.");
            }
            let new_ptr_pos = new_ptr_pos as usize;
            self.allocate_for_ptr(new_ptr_pos);
            self.ptr = new_ptr_pos;
            Ok(())
        }

        pub fn set(&mut self, elem: T) {
            // Set the element where the tape head points to <byte>.
            self.data[self.ptr] = elem;
        }

        pub fn zero(&mut self) {
            // Rewind the tape to 0.
            self.ptr = 0;
        }
    }

    impl<T: Copy + Clone + Default> FromIterator<T> for Tape<T> {
        fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
            let mut tape = Tape::<T>::new();
            for i in iter {
                tape.set(i);
                tape.seek(1).unwrap();
            }
            tape.zero();
            tape
        }
    }
}

pub mod brainfuck {
    use crate::bf_tape::Tape;
    use std::fs;
    use std::io;
    use std::io::Read;
    use std::io::Write;

    pub fn load_program(file: &String) -> Result<Tape<char>, String> {
        // Load program from file or stdin, return program as a Tape of
        // valid brainfuck symbols.

        // Get program
        let mut input = match fs::read_to_string(file) {
            Ok(input) => input,
            Err(err) => {
                return Err(format!("Couldn't read file: {}", err.to_string()));
            }
        };

        // Check program is valid, and load.
        input.retain(|c| !c.is_whitespace()); // Strip whitespace
        input.retain(|c| "<>+-.,[]".contains(c)); // Strip non-BF chars

        Ok(input.chars().collect())
    }

    pub fn run_program(mut program: Tape<char>) -> Result<(), &'static str> {
        let mut memory = Tape::<u8>::new();
        let mut bracket_stack = Vec::<usize>::new();
        loop {
            match program.next() {
                '>' => {
                    memory.seek(1).unwrap();
                }
                '<' => {
                    memory.seek(-1).unwrap();
                }
                '+' => {
                    memory.set(memory.get().wrapping_add(1));
                }
                '-' => {
                    memory.set(memory.get().wrapping_add_signed(-1));
                }
                '.' => {
                    print_byte_to_stdout(memory.get()).unwrap();
                }
                ',' => {
                    memory.set(get_byte_from_stdin().expect("Couldn't read from stdin"));
                }
                '[' => {
                    left_bracket(&mut program, &memory, &mut bracket_stack).unwrap();
                }
                ']' => {
                    right_bracket(&mut program, &memory, &mut bracket_stack).unwrap();
                }
                '\0' => return Ok(()), // End of tape
                _ => {
                    return Err("Invalid program symbol.");
                }
            }
        }
    }

    fn get_byte_from_stdin() -> Result<u8, io::Error> {
        if let Some(byte) = io::stdin().bytes().next() {
            byte
        } else {
            Ok(0u8)
        }
    }

    fn left_bracket(
        program: &mut Tape<char>,
        memory: &Tape<u8>,
        bracket_stack: &mut Vec<usize>,
    ) -> Result<(), &'static str> {
        // Implement the BF [ command. If returns Err if no ] is found.
        let orig_stack_len = bracket_stack.len();
        bracket_stack.push(program.pos());
        if memory.get() != 0 {
            return Ok(());
        }
        loop {
            match program.next() {
                '[' => {
                    bracket_stack.push(program.pos());
                }
                ']' => {
                    bracket_stack
                        .pop()
                        .expect("Unexpected empty bracket stack.");
                    if bracket_stack.len() == orig_stack_len {
                        // We've reached the matching ]
                        return Ok(());
                    }
                }
                '\0' => return Err("Reached end of tape before finding matching ]"),
                _ => (),
            }
        }
    }

    fn print_byte_to_stdout(byte: u8) -> std::io::Result<()> {
        io::stdout().write(&[byte])?;
        io::stdout().flush()?;
        Ok(())
    }

    fn right_bracket(
        program: &mut Tape<char>,
        memory: &Tape<u8>,
        bracket_stack: &mut Vec<usize>,
    ) -> Result<(), &'static str> {
        // Implement the BF ] command. If returns Err if no [ is found.
        if bracket_stack.len() == 0 {
            return Err("Encountered ] without matching [");
        }
        if memory.get() == 0 {
            bracket_stack.pop();
        } else {
            program
                .seek(*bracket_stack.last().unwrap() as i32 - program.pos() as i32)
                .expect("Coudn't seek program tape.");
        }
        Ok(())
    }
}
