pub fn retrive_last_patch() {
    let last_patch = reqwest::get("https://ddragon.leagueoflegends.com/api/versions.json");
    println!("{:#?}", last_patch);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_retrive() {
        retrive_last_patch();
        assert!( 1 == 1);
    }
}
