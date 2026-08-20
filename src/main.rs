use std::{env, error::Error, process, sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use ignore::WalkBuilder;

fn main() {
    let config = Config::cloner(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1)
    });
    let start_time = Instant::now();
    // Run the application engine
    if let Err(e) = file_search(&config) {
        eprintln!("Application Error: {e}");
        process::exit(1);
    }
    let duration = start_time.elapsed();
    println!("\nSearch completed in: {:.2?}", duration)
}



struct Config {
    target_file: String,
    starting_dir: String,
    ignore_case: bool
}
impl Config {
    fn cloner(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str>{
        args.next();
        let target_file = match args.next() {
            Some(arg) => arg,
            None => return Err("You did not input the target file"),
        };
        let starting_dir = match args.next() {
            Some(arg) => arg,
            None => return Err("You did not input the directory")
        }; 
        let ignore_case = env::var("IGNORE_CASE").is_err();
        Ok(Config { 
            target_file, 
            starting_dir, 
            ignore_case })
    }
    
    
}
fn file_search(config: &Config) -> Result<(), Box<dyn Error>>{
    let walker = WalkBuilder::new(&config.starting_dir)
                                .build_parallel();
    
    let target = config.target_file.clone();
    let ignore_case = config.ignore_case;
    let match_count = AtomicUsize::new(0);
    walker.run(||{
        let count_ref = &match_count;
        let target_file = target.clone();
        Box::new(move |result|{
            if let Ok(entry) = result{
                if let Some(file_name) = entry.file_name().to_str() {
                    let is_match = if ignore_case {
                        file_name.to_lowercase().contains(&target_file.to_lowercase())
                    } else {
                        file_name.contains(&target_file)
                    };
                    if is_match {
                        println!("[FOUND]: {}", entry.path().display());
                        count_ref.fetch_add(1,Ordering::SeqCst);
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
    let total_found =match_count.load(Ordering::SeqCst);
    if total_found == 0 {
        println!("\nNo files with the same found");
    }
    else {
        println!("Found {} match files", total_found)
    }
    Ok(())
}