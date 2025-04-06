pub async fn retrive_last_patch() -> reqwest::Response {
    let last_patch = reqwest::get("https://ddragon.leagueoflegends.com/api/versions.json");
    last_patch.await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_retrive() {
        let response = retrive_last_patch().await;
        println!("{:#?}", response.json::<Vec<String>>().await.unwrap()[0]);
        assert!( 1 == 1);
    }
}
