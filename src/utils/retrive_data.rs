use tar::Archive;
use std::io::Cursor;
use flate2::read::GzDecoder;

pub async fn retrive_last_patch() -> String{
    let response = reqwest::get("https://ddragon.leagueoflegends.com/api/versions.json");
    response.await.unwrap().json::<Vec<String>>().await.unwrap()[0].clone()
}

pub async fn get_data_tarball(){
    let url = format!("https://ddragon.leagueoflegends.com/cdn/dragontail-{}.tgz", retrive_last_patch().await);
    
    println!("Downloading: {:#?}... Might take a while", url);
    let response = reqwest::get(url).await.unwrap();
    let content = Cursor::new(response.bytes().await.unwrap());
    let decompressed = GzDecoder::new(content);

    let mut archive = Archive::new(decompressed);
    archive.unpack("assets/data_tarball").unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_retrive() {
        get_data_tarball().await;
    }
}
