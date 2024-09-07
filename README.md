# LNC


## About

LNC is an interpreter for the [Little Man Computer](https://en.wikipedia.org/wiki/Little_man_computer).
It can be used to run, test, and debug LMC programs. Usage and a description of
the syntax that LNC understands can be found below. A couple of example LNC
programs can be found in [examples](./examples).


## Building

LNC is written in Rust and can be built with cargo. To build it, just clone the
repo and then use
```
cargo build --release
```
The binary will be located at `lnc/target/release/lnc`.


## Usage

To use LNC, you just need to specify the path to a source file like so
```
$ path/to/lnc path/to/source.lnc
```
This will try to turn the source code into a Little Man Computer program,
reporting errors if any are found, and then runs it in the interpreter. While
running the program, the interpreter logs each instruction that is fetched and
sometimes a little more info (e.g. output values, whether overflow occurred in
addition, ...). Once finished, a summary of the inputs and outputs is printed.

There are two optional flags for `lnc`:

- `-t`, `--test`: this runs the tests specified in the source file and prints
  their results.
- `-d`, `--debug`: this prints more detail about the state of the Little Man
  Computer and allows you to step through instructions manually. Before an
  instruction is executed, a prompt `>>>` is displayed. This is used to enter
  how many instructions you want to execute before being prompted again. For
  example, if you wanted to execute the next 10 instructions you would type 10
  into the prompt: `>>> 10` and then press enter. If no number is entered, then
  only a single instruction is executed.

For example, if you wanted to run the tests for `examples/countdown.lnc`, you
would use
```
lnc examples/countdown.lnc -t
```
If you wanted to run it in debugging mode, would use
```
lnc examples/countdown.lnc -d
```

## Syntax

### Instructions
Source files are just plain text files. Each instruction is written on its own
line, and the address of that instruction will be 0 + the number of instructions
before it (i.e. the first instruction has address 0, the second address 1, and
so on). The supported instructions are:

- `lda xx` (load, `5xx`): copies the value located at address `xx` into the
  accumulator.
- `sto xx` (store, `3xx`): writes the value in the accumulator to memory at the
  address `xx`.
- `add xx` (add, `1xx`): adds the value located at address `xx` to the
  accumulator. Note that since the accumulator can only hold numbers 0-999, it
  is possible for overflow to occur. In the LNC interpreter, addition wraps
  around back to 0 upon overflow.
- `sub xx` (subtract, `2xx`): subtracts the value located at address `xx` from
  the accumulator. While the accumulator cannot hold negative numbers, if the
  result of the subtraction is below zero then the flag `neg_flag` is set for
  use with the `brp` instruction. In the LNC interpreter, subtraction will wrap
  around to 999 upon underflow.
- `inp` (input, `901`): takes the next input from the input basket and stores it
  in the accumulator. The input must be an integer in the range 0-999. When
  running the program normally or with the debug flag, this will prompt input
  from the user.
- `out` (output, `902`): copies the value from the accumulator and places it
  into the output basket.
- `hlt` (halt, `000`): halts the interpreter.
- `brz xx` (branch if zero, `7xx`): jumps to the address `xx` if the value in
  the accumulator is zero.
- `brp xx` (branch if positive, `8xx`): jumps to the address `xx` if `neg_flag`
  is not set. `neg_flag` is reset before executing an arithmetic instruction
  (`add` or `sub`) and is only set when the result of a `sub` instruction is
  negative.
- `bra xx` (branch always, `6xx`): jumps to the address `xx`.
- `dat xxx` (data, `xxx`): puts the value `xxx` in memory at the address of the
  instruction.

### Labels

Since it is hard specify the addresses numerically for each instruction that
needs one as an operand, labels are supported. To define a label, you can write
its name followed by a colon (`:`) at the beginning of a line. The address of
the label will be the same as the address of the next instruction that comes
after the label's definition. E.g.
```
inp
sto some_data

some_data: dat 0
```
Here, the label `some_data` is a synonym for the address `2` since there are two
instructions before its definition.

Labels can also be on their own line like so:
```
inp
sto some_data

some_data:

dat 0
```
This is equivalent to the previous snippet.

Labels must start with a letter or underscore (`_`), but the rest of the label
can contain letters, numbers, and underscores.

### White-space

The only white-space that is significant in source files is new lines: each line
can contain at most one instruction. Blank lines are okay.

### Comments

The semicolon character (`;`) is used to denote comments. Comments extend from
the semicolon until the end of the line, and their content is ignored.

### Tests

Tests are specified in the source files by using the following syntax:
```
.test_name [input1, input2, ...] [output1, output2, ...]
```
The line starts with a dot (`.`) and is immediately followed by the name of the
test, what the inputs will be, and what the outputs should be.

The test name follows the same rules as label names.

The inputs and outputs are surrounded by square brackets (`[]`) and are
separated by commas (`,`). If there are no inputs/outputs, you can put square
brackets with no numbers in between them: `[]`.
