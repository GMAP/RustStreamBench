use std::env;

mod sequential;
mod rust_ssp;
mod pipeliner;
mod tokio;
mod rayon;
mod std_threads;

fn main() -> std::io::Result<()>{
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!();
        panic!("Correct usage: $ ./{:?} <runtime> <nthreads> <images dir>", args[0]);
    }
    let run_mode = &args[1];
    let threads = args[2].parse::<usize>().unwrap();
    let dir_name = &args[3];

	match run_mode.as_str() {
        "sequential" => sequential::sequential(dir_name),
        "rust-ssp" => rust_ssp::rust_ssp(dir_name, threads),
        "pipeliner" => pipeliner::pipeliner(dir_name, threads),
        "tokio" => tokio::tokio(dir_name, threads),
        "rayon" => rayon::rayon(dir_name, threads),
        "std-threads" => std_threads::std_threads(dir_name, threads),
        _ => println!("Invalid run_mode, use: sequential | rust-ssp | std-threads | tokio | rayon | pipeliner"),
    
    }
	Ok(())
}
