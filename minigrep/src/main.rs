use std::env;
use std::process;

use minigrep::Config;



fn main() {
    // 获取命令行参数

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("problem parsing arguments: {err}");
        process::exit(1);  
    });

    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);
    
    if let Err(e) = minigrep::run(config){
        eprintln!("Application error: {e}");
        process::exit(1);
    }
    //dbg!(args);

    
}
