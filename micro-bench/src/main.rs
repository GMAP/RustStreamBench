mod sequential;
mod rust_ssp;
mod std_threads;
mod tokio;
mod rayon;
mod pipeliner;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        println!();
        panic!("Correct usage: $ ./{:?} <runtime> <img size> <nthreads> <iter size 1> <iter size 2>", args[0]);
    }
    let runtime = &args[1];
    let size = args[2].parse::<usize>().unwrap();
    let threads = args[3].parse::<usize>().unwrap();
    let iter_size1 = args[4].parse::<i32>().unwrap();
    let iter_size2 = args[5].parse::<i32>().unwrap();

    match runtime.as_str() {
        "sequential" => sequential::sequential(size, iter_size1, iter_size2),
        "rust-ssp" => rust_ssp::rust_ssp_pipeline(size, threads, iter_size1, iter_size2),
        "std-threads" => std_threads::std_threads_pipeline(size, threads, iter_size1, iter_size2),
        "tokio" => tokio::tokio_pipeline(size, threads, iter_size1, iter_size2),
        "rayon" => rayon::rayon_pipeline(size, threads, iter_size1, iter_size2),
        "pipeliner" => pipeliner::pipeliner_pipeline(size, threads, iter_size1, iter_size2),
        _ => println!("Invalid run_mode, use: sequential | rust-ssp | std-threads | tokio | rayon | pipeliner"),
    }
}
