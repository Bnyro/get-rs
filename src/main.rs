use std::{
    cmp::min,
    fs::{remove_file, File},
    io::Write,
    path::Path,
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

const CMD_NAME: &str = "get";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    println!("\nUsage: {} <URL> <PATH>", CMD_NAME);
}

fn print_version() {
    println!("\n{} v{}", CMD_NAME, VERSION);
}

fn create_file(file_path: &str) -> File {
    let path = Path::new(file_path);
    if path.exists() {
        println!("\nFile already exists. Nothing left to do here, see you!");
        std::process::exit(1);
    }
    File::create(path).unwrap_or_else(|_| panic!("Failed to create file '{}'!", file_path))
}

#[allow(clippy::unused_io_amount)]
async fn download_file(url: &str, target: &str) -> Result<(), String> {
    let client = Client::new();
    let request = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to connect to {}", &url)))?;

    let mut downloaded = 0;
    let download_size = request.content_length().unwrap_or(0);

    let mut file = create_file(target);
    let mut stream = request.bytes_stream();

    let progress = ProgressBar::new(download_size);
    let progress_style = ProgressStyle::default_bar()
        .template("\n{msg}\n\n{spinner:.cyan/blue} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})");
    progress.set_style(progress_style.unwrap());
    progress.set_message(format!("Downloading {}", url));

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file".to_string()))?;
        file.write(&chunk)
            .or(Err("Error while writing to file".to_string()))?;
        downloaded = min(downloaded + (chunk.len() as u64), download_size);
        progress.set_position(downloaded);
    }

    progress.finish_with_message(format!("  Downloaded {} to {}", url, target));
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() <= 1 {
        print_usage();
        return;
    }

    let first_arg = args.get(1).unwrap();
    if vec!["-h", "--help", "h", "help"].contains(&first_arg.as_str()) {
        print_usage();
        return;
    }

    if vec!["-v", "--version", "v", "version"].contains(&first_arg.as_str()) {
        print_version();
        return;
    }

    let url = first_arg;
    let download_file_name = url
        .split('/')
        .last()
        .unwrap_or(url.replace('/', "").as_str())
        .to_owned();
    let target = args.get(2).unwrap_or(&download_file_name).to_owned();
    let target_copy = target.clone();

    ctrlc::set_handler(move || {
        let file = std::path::Path::new(target_copy.as_str());
        _ = remove_file(file);
        println!("\n\nFinished cleanup. See you next time!");
        std::process::exit(1);
    })
    .expect("Unable to set ctrl c handler!");

    if let Err(err) = download_file(url, &target).await {
        println!("Error: {}", err)
    };
}
