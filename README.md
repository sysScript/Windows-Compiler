# SystemScript Compiler (ssc) -Beta

SystemScript is a statically-typed systems programming language. The compiler performs lexical analysis, parsing, semantic analysis, and code generation to produce native executables. You can find the [documentation here](https://github.com/sysScript/System-Script)

# Installation (with images)

> **BEFORE HAND YOU MUST HAVE RUST'S COMPILER INSTALLED**

## First; Install [NASM](https://www.nasm.us/pub/nasm/releasebuilds/3.01/win64/)
* Add it to root. (in search look up `system properties`) <img width="1208" height="626" alt="image" src="https://github.com/user-attachments/assets/fc99bc97-563d-4082-a96d-3afa14a38a69" />
* Then compile System Script's windows compiler. 
* Next open powershell and `cd` into the directory that you have downloaded the source code too. 
* Now you will need to execute the build script using `.\build.bat`

The compiler should be in `target/release` named `ssc.exe`


## Second; Install VS code
* Once done look up `X64 Native Tools Command Prompt` in your search bar. and `cd` into the same directory as the source code.
* You will now be able to use System Script's windows compiler! Have fun! <3



# Usage

```bash
ssc <source_file> [options]
```

### Options

- `-o <file>` - Set output file name (default: a.out)
- `-O<level>` - Set optimization level (0-3)
- `--emit-ir` - Generate intermediate representation file

### Examples

```bash
# Compile a program
ssc program.ss -o program.exe

# Compile with optimization
ssc program.ss -o program.exe -O2

# Generate IR for debugging
ssc program.ss -o program.exe --emit-ir
```

# Language Features

### Variables

```rust
let x: i32 = 42;           // Immutable variable
let mut y: i32 = 10;       // Mutable variable
const MAX: i32 = 100;      // Constant
```

### Types

- Integers: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- Floating point: `f32`, `f64`
- Boolean: `bool`
- Character: `char`
- String: `str`

### Operators

**Arithmetic:** `+`, `-`, `*`, `/`, `%`

**Comparison:** `==`, `!=`, `<`, `<=`, `>`, `>=`

**Logical:** `&&`, `||`, `!`

**Unary:** `-`, `!`

### Control Flow

```rust
// If-else
if (x > 0) {
    print("positive");
} else {
    print("non-positive");
}

// While loop
while (x < 10) {
    x = x + 1;
}

// For loop (range-based)
for (i in 0..5) {
    print("iteration");
}

// Infinite loop
loop {
    if (condition) {
        break;
    }
}
```

### Functions

```rust
fn main() -> i32 {
    return 0;
}
```

### Built-in Functions

- `print(str)` - Print string to stdout

## Example Program

```rust
module main;

fn main() -> i32 {
    const MAX: i32 = 10;
    let mut sum: i32 = 0;
    
    for (i in 0..MAX) {
        sum = sum + i;
    }
    
    print("Done");
    return 0;
}
```

## Compilation Pipeline

1. **Lexical Analysis** - Source code → Tokens
2. **Parsing** - Tokens → Abstract Syntax Tree (AST)
3. **Semantic Analysis** - Type checking and scope validation
4. **Code Generation** - AST → x64 assembly
5. **Assembly & Linking** - Assembly → Executable

## Output Files

- `<output>.exe` - Executable file
- `<output>.asm` - Assembly source (preserved for debugging)
- `<output>.ir` - Intermediate representation (with `--emit-ir`)

## Requirements

- NASM assembler
- Microsoft Visual Studio (for linker)
- Windows x64 platform

## Error Messages

The compiler reports errors with context:

```
Compilation failed: Semantic error: Cannot assign to immutable variable 'x'
```


> [!WARNING] 
> Disclaimer; This compiler is in beta and is still in development.
