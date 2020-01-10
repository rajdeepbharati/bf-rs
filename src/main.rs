extern crate termios;
use std::io;
use std::io::Read;
use std::io::Write;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use std::char;
use std::collections::HashMap;
use std::env;
use std::fs;

fn evaluate(code: String) {
    let code_vec: Vec<char> = code.chars().collect();
    let code_without_comments = cleanup(code_vec);
    let code_vec_fmt: Vec<char> = code_without_comments.chars().collect();
    let (bracemap, code_vec_fmt) = buildbracemap(code_vec_fmt);

    let mut cells = vec![0];
    let mut codeptr = 0;
    let mut cellptr = 0;
    let mut command;
    while codeptr < code_vec_fmt.len() {
        command = code_vec_fmt[codeptr];

        if command == '>' {
            cellptr += 1;
            if cellptr == cells.len() {
                cells.push(0);
            }
        }
        if command == '<' {
            if cellptr <= 0 {
                cellptr = 0;
            } else {
                cellptr = cellptr - 1;
            }
        }
        if command == '+' {
            if cells[cellptr] < 255 {
                cells[cellptr] = cells[cellptr] + 1;
            } else {
                cells[cellptr] = 0;
            }
        }
        if command == '-' {
            if cells[cellptr] > 0 {
                cells[cellptr] = cells[cellptr] - 1;
            } else {
                cells[cellptr] = 255;
            }
        }

        if command == '[' && cells[cellptr] == 0 {
            codeptr = bracemap[&codeptr];
        }
        if command == ']' && cells[cellptr] != 0 {
            codeptr = bracemap[&codeptr];
        }
        if command == '.' {
            let u8_c = cells[cellptr] as u8;
            print!("{}", u8_c as char);
        }
        if command == ',' {
            let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work
                           // on /dev/stdin or /dev/tty
            let termios = Termios::from_fd(stdin).unwrap();
            let mut new_termios = termios.clone(); // make a mutable copy of termios
                                                   // that we will modify
            new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
            tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();
            let stdout = io::stdout();
            let mut reader = io::stdin();
            let mut buffer = [0; 1]; // read exactly one byte

            stdout.lock().flush().unwrap();
            reader.read_exact(&mut buffer).unwrap();

            cells[cellptr] = buffer[0];

            tcsetattr(stdin, TCSANOW, &termios).unwrap(); // reset the stdin to original termios data
        }

        codeptr += 1;
    }
}

fn cleanup(code: Vec<char>) -> String {
    let mut fmstr = String::from("");
    let allowed_tokens = ['>', '<', '+', '-', '.', ',', '[', ']'];
    for c in code {
        if allowed_tokens.contains(&c) {
            fmstr.push(c);
        }
    }

    fmstr
}

fn buildbracemap(code: Vec<char>) -> (HashMap<usize, usize>, Vec<char>) {
    let mut temp_bracestack = vec![];
    let mut start;
    let mut bracemap: HashMap<usize, usize> = HashMap::new();
    for (position, command) in code.iter().enumerate() {
        if command.to_string() == "[".to_string() {
            temp_bracestack.push(position);
        }
        if command.to_string() == "]".to_string() {
            start = temp_bracestack.remove(temp_bracestack.len() - 1);
            bracemap.insert(start, position);
            bracemap.insert(position, start);
        }
    }

    (bracemap, code)
}

fn execute(filename: String) {
    let contents =
        fs::read_to_string(filename).expect("Oops, something went wrong reading the file");
    evaluate(contents);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        loop {
            print!("bf$ ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            evaluate(input)
        }
    } else {
        let filename = String::from(&args[1]);
        println!("In file {}", filename);
        execute(filename);
        println!();
    }
}
