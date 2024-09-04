pub trait Output {
    fn send(&mut self, val: usize);
}

pub trait Input {
    fn take(&mut self) -> Result<LNCInput, String>;
}

pub trait Log {
    fn log(&mut self, msg: String);
}

pub struct LNCInput(usize);

impl LNCInput {
    pub fn new(num: usize) -> Option<Self> {
        if num < 1000 {
            Some(LNCInput(num))
        } else {
            None
        }
    }
}

impl From<LNCInput> for usize {
    fn from(value: LNCInput) -> Self {
        value.0
    }
}

pub struct Interpreter<'a, I: Input, O: Output, L: Log> {
    mem: [usize; 100],
    pc: usize,
    acc: usize,
    neg_flag: bool,
    halted: bool,
    input: &'a mut I,
    output: &'a mut O,
    logger: &'a mut L,
}

impl<'a, I: Input, O: Output, L: Log> Interpreter<'a, I, O, L> {
    pub fn new(mem: [usize; 100], input: &'a mut I, output: &'a mut O, logger: &'a mut L) -> Self {
        Self {
            mem,
            pc: 0,
            acc: 0,
            neg_flag: false,
            halted: false,
            input,
            output,
            logger,
        }
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn step(&mut self) -> Result<(), String> {
        if self.halted {
            self.logger.log("Cannot step: interpreter is halted".into());
            return Ok(());
        }

        let code = self.mem[self.pc];

        self.logger.log(format!(
            "Fetched instruction: {} at address {}",
            code, self.pc
        ));

        self.pc += 1;

        let (first_digit, op) = (code / 100, code % 100);

        match first_digit {
            // load
            5 => self.lda(op),
            // store
            3 => self.sto(op),
            // add
            1 => self.add(op),
            // subtract
            2 => self.sub(op),
            9 => {
                match op {
                    // input
                    01 => self.inp()?,
                    // output
                    02 => self.out(),
                    _ => return Err(format!("{}{}: undefined instruction", first_digit, op)),
                }
            }
            // halt
            0 => match op {
                00 => self.hlt(),
                _ => return Err(format!("{}{}: undefined instruction", first_digit, op)),
            },
            // branch if zero
            7 => self.brz(op),
            // branch if zero or positive
            8 => self.brp(op),
            // branch always
            6 => self.bra(op),
            _ => return Err(format!("{}{}: undefined instruction", first_digit, op)),
        };

        Ok(())
    }

    fn lda(&mut self, addr: usize) {
        self.logger.log(format!("--> lda {}", addr));
        self.acc = self.mem[addr];
    }

    fn sto(&mut self, addr: usize) {
        self.logger.log(format!("--> sto {}", addr));
        self.mem[addr] = self.acc;
    }

    fn inp(&mut self) -> Result<(), String> {
        self.logger.log("--> inp".into());

        let inp_val = self.input.take()?.into();
        self.logger.log(format!("--> {} was input value", inp_val));

        self.acc = inp_val;

        Ok(())
    }

    fn out(&mut self) {
        self.logger.log("--> out".into());
        self.logger
            .log(format!("--> {} was output value", self.acc));

        self.output.send(self.acc);
    }

    fn hlt(&mut self) {
        self.logger.log("--> hlt".into());
        self.halted = true;
    }

    fn add(&mut self, addr: usize) {
        self.logger.log(format!("--> add {}", addr));

        let new_val = self.acc + self.mem[addr];
        if new_val >= 1000 {
            self.logger.log(format!(
                "--> {} + {} = {} >= 1000: overflow",
                self.acc, self.mem[addr], new_val
            ));
        }
        self.acc = new_val % 1000;

        self.neg_flag = false;
    }

    fn sub(&mut self, addr: usize) {
        self.logger.log(format!("--> sub {}", addr));

        let new_val = self.acc as isize - self.mem[addr] as isize;
        self.neg_flag = new_val < 0;

        if self.neg_flag {
            self.logger.log(format!(
                "--> {} - {} = {} < 1000: underflow",
                self.acc, self.mem[addr], new_val
            ));
            self.logger.log("neg_flag set".into());
        }

        self.acc = (new_val + 1000) as usize % 1000;
    }

    fn brz(&mut self, addr: usize) {
        self.logger.log(format!("--> brz {}", addr));
        if self.acc == 0 {
            self.pc = addr;
        }
    }

    fn brp(&mut self, addr: usize) {
        self.logger.log(format!("--> brp {}", addr));
        if !self.neg_flag {
            self.pc = addr;
        }
    }

    fn bra(&mut self, addr: usize) {
        self.logger.log(format!("--> bra {}", addr));
        self.pc = addr;
    }
}
