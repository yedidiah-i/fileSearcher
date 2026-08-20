use std::{env, error::Error, process, time::Instant};
use ignore::WalkBuilder;

fn main() {
    let config = Config::cloner(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1)
    });

    // Run the application engine
    if let Err(e) = file_search(&config) {
        eprintln!("Application Error: {e}");
        process::exit(1);
    }
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
    let start_time = Instant::now();
    let walker = WalkBuilder::new(&config.starting_dir)
                                .build_parallel();
    
    let target = config.target_file.clone();
    let ignore_case = config.ignore_case;

    walker.run(||{
        let target_file = target.clone();
        Box::new(move |result|{
            if let Ok(entry) = result{
                if let Some(file_name) = entry.file_name().to_str() {
                    let is_match = if ignore_case {
                        file_name.to_lowercase() == target_file.to_lowercase()
                    } else {
                        file_name == target_file
                    };
                    if is_match {
                        println!("[FOUND]: {}", entry.path().display())
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
    let duration = start_time.elapsed();
    let new_time_stamp = start_time + duration;
    println!("{:.2?}", new_time_stamp);
    Ok(())
}