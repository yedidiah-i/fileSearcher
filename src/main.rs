use std::{env, error::Error, process, sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use ignore::WalkBuilder;
use sysinfo::{Disks};

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
    println!("Search completed in: {:.2?}", duration)
}
struct Config {
    target_file: String,
    starting_dir: Option<String>,
    ignore_case: bool
}
impl Config {
    fn cloner(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str>{
        args.next();
        let target_file = args.next().ok_or("You have not entered the target file consider adding the\nfilename after the searchfile keyword")?;
        let starting_dir = args.next();
        let ignore_case = env::var("IGNORE_CASE").is_err();
        Ok(Config { 
            target_file, 
            starting_dir, 
            ignore_case })
    }
}
fn file_search(config: &Config) -> Result<(), Box<dyn Error>>{
    let search_paths: Vec<String> = match config.starting_dir.as_ref() {
        Some(chosen_folder) => {
            println!("Scanning directory starting from '{chosen_folder}'...");
            vec![chosen_folder.clone()]
        }
        None => {
            println!("No directory provided. Detecting all connected system drives...");
            let disks = Disks::new_with_refreshed_list();
            let paths: Vec<String> = disks
                .list()
                .iter()
                .map(|disk| disk.mount_point().to_string_lossy().into_owned())
                .collect();
            
            println!("Searching across whole PC: ");
            paths
        }
    };    
    let target = config.target_file.clone();
    let ignore_case = config.ignore_case;
    let match_count = AtomicUsize::new(0);
    
    for path in search_paths {
        let walker = WalkBuilder::new(&path)
            .ignore(false)
            .git_ignore(false)
            .parents(false)
            .hidden(false)
            .build_parallel();
        let count_ref = &match_count;
        
        walker.run(||{
            let target_file = target.clone();
            Box::new(move |entry_result|{
                if let Ok(entry) = entry_result{
                    if let Some(file_name) = entry.file_name().to_str() {
                        let is_match = if ignore_case {
                            file_name.to_lowercase().contains(&target_file.to_lowercase())
                        } else {
                            file_name.contains(&target_file)
                        };
                        if is_match {
                            println!("\n[FOUND]: {}", entry.path().display());
                            count_ref.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });
    }
    let total_found =match_count.load(Ordering::SeqCst);
    if total_found == 0 {
        println!("\nNo files with the same found");
    }
    else {
        println!("\nFound {} match files", total_found)
    }
    Ok(())
}