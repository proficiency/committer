use rand::Rng;
use serde::Deserialize;

use std::{
    fmt::Debug,
    fs::{File, read_to_string, write},
    io::{self, BufReader, Write},
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
struct Config {
    remote_origin_url: String,
    branch_name: String,
    file_path: String,
    commit_schedule: i32,
    random_schedule: bool,
}

fn main() {
    // load committer.json as a config.
    let config = load_config("comitter.json").expect("Failed to load config");

    // the main program loop
    loop {
        let start_time = Instant::now();
        let unique_id = generate_unique_id();
        let commit_message = format!("[committer] commit #{}", unique_id);

        if let Err(e) = modify_file_randomly(&config.file_path, &unique_id) {
            spdlog::info!("failed to modify file: {}", e);
            wait_for_keypress();
            return;
        }

        if !run_git_command(&["add", "."]) {
            spdlog::info!("failed to stage changes with `git add .`");
            wait_for_keypress();
            return;
        }

        if !run_git_command(&["commit", "-m", &commit_message]) {
            spdlog::info!("Failed to commit changes");
            wait_for_keypress();
            return;
        }

        if !run_git_command(&["push", &config.remote_origin_url, &config.branch_name]) {
            spdlog::info!("Failed to push changes to remote");
            wait_for_keypress();
            return;
        }

        spdlog::info!("pushed a commit with ID: {}", unique_id);

        let wait_time = if config.random_schedule {
            let mut rng = rand::thread_rng();
            Duration::from_secs(rng.gen_range(5..61) * 60)
        } else {
            Duration::from_secs((config.commit_schedule * 60).try_into().unwrap())
        };

        let elapsed = start_time.elapsed();
        let sleep_duration = wait_time.saturating_sub(elapsed);

        spdlog::info!("sleeping {:?} before the next commit...", sleep_duration);
        thread::sleep(sleep_duration);
    }
}

fn load_config(filename: &str) -> io::Result<Config> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader).map_err(|e| {
        spdlog::error!("failed to parse config: {}", e);
        io::Error::new(io::ErrorKind::InvalidData, "invalid json")
    })?;

    spdlog::info!("parsed config");
    Ok(config)
}

fn modify_file_randomly(file_path: &str, unique_id: &str) -> io::Result<()> {
    let mut content = read_to_string(file_path).unwrap_or_else(|_| String::new());
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..2) == 0 && !content.is_empty() {
        let delete_idx = rng.gen_range(0..content.len());
        content = content[..delete_idx].to_string();
    } else {
        content.push_str(&format!("{} ", unique_id));
    }

    write(file_path, content)
}

fn generate_unique_id() -> String {
    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| charset[rng.gen_range(0..charset.len())])
        .collect()
}

fn run_git_command(args: &[&str]) -> bool {
    let status = Command::new("git").args(args).status().expect("failed to run git command");
    spdlog::info!("Running: {:?}", args);
    status.success()
}

fn wait_for_keypress() {
    println!("Press any key to exit...");
    let _ = io::stdin().read_line(&mut String::new());
}
