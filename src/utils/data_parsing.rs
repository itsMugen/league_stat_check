use crate::retrive_last_patch;
use crate::Stats;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub async fn aggregate_data() -> HashMap<String, Stats> {
    println!(
        "{:#?}",
        format!(
            "assets/data_tarball/{}/data/en_US/champion",
            retrive_last_patch().await
        )
    );
    let champs_dir = fs::read_dir(format!(
        "assets/data_tarball/{}/data/en_US/champion",
        retrive_last_patch().await
    ))
    .unwrap();
    let mut champs: HashMap<String, Stats> = HashMap::new();
    for champ_path in champs_dir {
        let path = champ_path.unwrap();
        let champ_file = &path.file_name().into_string().unwrap();
        let champ_name = champ_file.split(".").collect::<Vec<_>>()[0];

        let champ_json = fs::read(&path.path());

        let champ_data: Value = serde_json::from_slice(&champ_json.unwrap()).expect("some data");

        let stats: Stats =
            serde_json::from_value(champ_data["data"][champ_name]["stats"].clone()).unwrap();

        champs.insert(champ_name.to_string(), stats);
    }
    champs
}
