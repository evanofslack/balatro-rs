use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let paths: Vec<String> = env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: jkr <file.jkr>...");
        return ExitCode::from(2);
    }

    let mut had_error = false;
    for (i, path) in paths.iter().enumerate() {
        if paths.len() > 1 {
            if i > 0 {
                println!();
            }
            println!("== {path} ==");
        }

        let result = fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| balatro_jkr::decode(&bytes).map_err(|e| e.to_string()));

        match result {
            Ok(value) => println!("{}", balatro_jkr::print_pretty(&value)),
            Err(e) => {
                eprintln!("{path}: {e}");
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
