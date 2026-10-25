fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let output = clens_cli::dispatch_live(&args);
    println!("{}", output.text);
    std::process::exit(output.code as i32);
}
