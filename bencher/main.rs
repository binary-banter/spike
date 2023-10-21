use mongodb::options::{ClientOptions, Credential, ServerAddress};
use mongodb::Client;
use rust_compiler_construction::compile;
use std::fs::{File, FileType};
use std::path::Path;
use std::{env, fs};
use tempdir::TempDir;
use walkdir::WalkDir;

#[tokio::main]
async fn main() {
    let mongo_pw =
        env::var("MONGO_PW").expect("No environment variable was set to connect to the database.");

    let client_options = ClientOptions::builder()
        .credential(Some(
            Credential::builder()
                .username(Some("cc".to_string()))
                .password(Some(mongo_pw))
                .build(),
        ))
        .hosts(vec![ServerAddress::parse("83.86.212.125:27017").unwrap()])
        .build();
    let client = Client::with_options(client_options).unwrap();

    for db_name in client.list_database_names(None, None).await.unwrap() {
        println!("{}", db_name);
    }

    // let tempdir = TempDir::new("cc-bench").unwrap();
    //
    // let path = Path::new("../programs/good");
    // for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()).filter(|f| f.file_type().is_file()) {
    //     let content = fs::read_to_string(entry.path()).unwrap();
    //     compile(&content, &tempdir.path().join("output")).unwrap();
    //
    // }
}
