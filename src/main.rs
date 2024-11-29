use reqwest::Client;
use serde::Deserialize;
use tokio::sync::Semaphore;
use futures::future;
use std::{
    fs::File,
    io::{self, Write},
    path::Path, sync::Arc,
};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter the desired Minecraft version: ");
    let mut mc_version = String::new();
    io::stdin().read_line(&mut mc_version)?;
    let mc_version = mc_version.trim();

    println!("Enter the desired loader: ");
    let mut loader = String::new();
    io::stdin().read_line(&mut loader)?;
    let loader = loader.trim().to_lowercase();

    let api_url = "https://api.modrinth.com/v3/collection/3GZrvdRN";

    let client = Client::new();

    let response = client.get(api_url).send().await?;
    let collection: ModrinthCollection = response.json().await?;

    std::fs::create_dir_all("mods")?;

    // Semaphore to limit the number of concurrent downloads
    let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent downloads (adjust as needed)
    
    let mut download_tasks = vec![];

    for project_id in collection.projects {
        let project_url = format!("https://api.modrinth.com/v3/project/{}", project_id);

        // Fetch project details
        let project_response = client.get(&project_url).send().await?;
        let project: ModrinthProject = project_response.json().await?;

        for version_id in project.versions {
            let version_url = format!("https://api.modrinth.com/v3/version/{}", version_id);

            // Fetch version details
            let version_response = client.get(&version_url).send().await?;
            let version: ModrinthVersion = version_response.json().await?;

            // Check compatibility
            if version.game_versions.contains(&mc_version.to_string())
                && version.loaders.contains(&loader)
            {
                let file_url = version.files[0].url.clone();
                let file_name = format!("mods/{}", version.files[0].filename);

                if !Path::new(&file_name).exists() {
                    let client = client.clone();
                    let file_url = file_url.clone();
                    let semaphore = Arc::clone(&semaphore);
                    let semaphore = semaphore.clone();

                    // Spawn a task for downloading the mod concurrently
                    let task = tokio::spawn(async move {
                        let permit = semaphore.acquire().await.unwrap(); // Acquire a permit before starting the download
                        match download_file(&client, &file_url, &file_name).await {
                            Ok(()) => println!("Downloaded {} successfully.", file_name),
                            Err(e) => eprintln!("Failed to download {}: {}", file_name, e),
                        }
                        drop(permit); // Release the permit after the task completes
                    });

                    download_tasks.push(task);
                } else {
                    println!("File {} already exists, skipping download.", file_name);
                }
            }
        }
    }

    // Wait for all download tasks to finish
    future::join_all(download_tasks).await; 

    println!("All compatible mods downloaded!");
    Ok(())
}

// Function to handle the file download
async fn download_file(client: &Client, file_url: &str, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(file_name)?;
    let mut response = client.get(file_url).send().await?;

    // Stream the content to the file
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }

    Ok(())
}

// Data structures for deserializing API responses
#[derive(Deserialize)]
struct ModrinthCollection {
    projects: Vec<String>, // Projects are represented as project IDs (strings)
}

#[derive(Deserialize)]
struct ModrinthProject {
    slug: String,
    versions: Vec<String>, // Versions are represented as version IDs (strings)
}

#[derive(Deserialize)]
struct ModrinthVersion {
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<ModrinthFile>,
}

#[derive(Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
}
