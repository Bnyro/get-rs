use std::{
    cmp::min,
    fs::{remove_file, File},
    io::Write,
    path::Path,
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

fn create_file(file_path: String) -> File {
    let path = Path::new(&file_path);
    if path.exists() {
        println!("\nFile already exists! Quitting...");
        std::process::exit(1);
    }
    File::create(path).expect(format!("Failed to create file '{}'!", file_path).as_str())
}

async fn download_file(url: String, target: String) -> Result<(), String> {
    let client = Client::new();
    let request = client
        .get(&url)
        .send()
        .await
        .or(Err(format!("Failed to connect to {}", &url)))?;

    let download_size = request.content_length().unwrap_or(0);

    let mut file = create_file(target.clone());
    let mut downloaded = 0;
    let mut stream = request.bytes_stream();

    let progress = ProgressBar::new(download_size);
    let progress_style = ProgressStyle::default_bar()
        .template("\n{msg}\n\n{spinner:.cyan/blue} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})");
    progress.set_style(progress_style.unwrap());
    progress.set_message(format!("Downloading {}", url));

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        downloaded = min(downloaded + (chunk.len() as u64), download_size);
        progress.set_position(downloaded);
    }

    progress.finish_with_message(format!("  Downloaded {} to {}", url, target));
    return Ok(());
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 0 {
        println!("Please provide an URL as parameter!");
        return;
    }

    let url = args.get(1).unwrap();
    let download_file_name = url
        .split("/")
        .last()
        .unwrap_or(url.replace("/", "").as_str())
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

    match download_file(url.to_owned(), target.to_owned()).await {
        Err(err) => {
            println!("Error: {}", err.to_string())
        }
        _ => {}
    };
}
