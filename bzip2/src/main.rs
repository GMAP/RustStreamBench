use std::env;

mod sequential;
mod rust_ssp;
mod std_threads;
mod tokio;
mod rayon;
mod pipeliner;

fn main() -> std::io::Result<()>{
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!();
        panic!("Correct usage: $ ./{:?} <runtime> <nthreads> <compress/decompress> <file name>", args[0]);
    }
    let run_mode = &args[1];
    let threads = args[2].parse::<usize>().unwrap();
    let file_action = &args[3];
    let file_name = &args[4];

	match run_mode.as_str() {
        "sequential" => sequential::sequential(file_action, file_name),
        "sequential-io" => sequential::sequential_io(file_action, file_name),
        "rust-ssp" => rust_ssp::rust_ssp(threads, file_action, file_name),
        "rust-ssp-io" => rust_ssp::rust_ssp_io(threads, file_action, file_name),
        "std-threads" => std_threads::std_threads(threads, file_action, file_name),
        "std-threads-io" => std_threads::std_threads_io(threads, file_action, file_name),
        "tokio" => tokio::tokio(threads, file_action, file_name),
        "tokio-io" => tokio::tokio_io(threads, file_action, file_name),
        "rayon" => rayon::rayon(threads, file_action, file_name),
        "pipeliner" => pipeliner::pipeliner(threads, file_action, file_name),
        _ => println!("Invalid run_mode, use: sequential | rust-ssp | std-threads | tokio | rayon | pipeliner"),
    
    }
	
	Ok(())
}
