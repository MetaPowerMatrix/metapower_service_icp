use anyhow::anyhow;
use futures::StreamExt;
use reqwest::Client;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

pub async fn download_image(url: &str, file_path: &str) -> Result<(), anyhow::Error> {
    // Create a reqwest Client
    let client = Client::new();

    // Send a GET request to the image URL
    match client.get(url).send().await{
        Ok(response) => {
            // Ensure the response is successful (status code in the 200 range)
            if !response.status().is_success() {
                return Err(anyhow!(
                    format!("Failed to download image: {}", response.status()),
                ));
            }

            // Create a File to save the image data
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)
                .await?;

            // Create a ReaderStream from the response body
            let bytes = response.bytes().await?;
            let mut stream = ReaderStream::new(bytes.as_ref());
            // Copy the stream data to the file
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
            }
        },
        Err(e) => {
            return Err(anyhow!(format!("Failed to send request: {}", e)));
        }
    }

    Ok(())
}
