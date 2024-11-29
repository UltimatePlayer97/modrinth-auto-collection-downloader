use reqwest::Client;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
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

    for project_id in collection.projects {
        let project_url = format!("https://api.modrinth.com/v3/project/{}", project_id);

        // Fetch project details
        let project_response = client.get(&project_url).send().await?;
        let project: ModrinthProject = project_response.json().await?;

        let mut downloaded = false;

        for version_id in project.versions {
            let version_url = format!("https://api.modrinth.com/v3/version/{}", version_id);

            // Fetch version details
            let version_response = client.get(&version_url).send().await?;
            let version: ModrinthVersion = version_response.json().await?;

            // Check compatibility
            if version.game_versions.contains(&mc_version.to_string())
                && version.loaders.contains(&loader)
            {
                // Download the file only once for the first matching version
                if !downloaded {
                    let file_url = &version.files[0].url;
                    let file_name = format!("mods/{}", version.files[0].filename);

                    if !Path::new(&file_name).exists() {
                        println!("Downloading {}...", version.files[0].filename);
                        let mut file = File::create(&file_name)?;
                        let mut response = client.get(file_url).send().await?;

                        // Stream the content to the file
                        while let Some(chunk) = response.chunk().await? {
                            file.write_all(&chunk)?;
                        }
                        println!("Downloaded {} successfully.", version.files[0].filename);
                    } else {
                        println!("File {} already exists, skipping download.", version.files[0].filename);
                    }

                    downloaded = true;  // Mark the project as downloaded to prevent further downloads
                    break;  // Break the inner loop once a valid version is found and downloaded
                }
            }
        }
    }

    println!("All compatible mods downloaded!");
    Ok(())
}


#[derive(Deserialize)]
struct ModrinthCollection {
    projects: Vec<String>,
}

#[derive(Deserialize)]
struct ModrinthProject {
    slug: String,
    versions: Vec<String>,
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
