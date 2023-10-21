use mongodb::options::{ClientOptions, Credential, ServerAddress};
use mongodb::Client;
use rust_compiler_construction::utils::split_test::split_test_raw;
use std::fs::{File};
use std::path::{Path, PathBuf};
use std::{env, fs};
use std::collections::HashMap;
use mongodb::bson::{doc, Document};
use pathdiff::diff_paths;
use tempdir::TempDir;
use walkdir::WalkDir;
use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::interpreter::{IO, TestIO};
use rust_compiler_construction::language::x86var::IStats;
use rust_compiler_construction::parser::parse_program;

/// Stats gathered by the bencher.
#[derive(Debug)]
struct BStats {
    binary_size: usize,
    // todo: speed?
}

/// Accumulated stats.
#[derive(Debug)]
struct Stats {
    bencher_stats: BStats,
    interpreter_stats: IStats,
}

trait Check {
    /// Checks whether `self` does not regress compared to `other`.
    fn check(&self, other: &Self) -> bool;
}

impl Check for Stats {
    fn check(&self, other: &Self) -> bool {
        self.bencher_stats.check(&other.bencher_stats)
            && self.interpreter_stats.check(&other.interpreter_stats)
    }
}

impl Check for BStats {
    fn check(&self, other: &Self) -> bool {
        self.binary_size <= other.binary_size
    }
}

impl Check for IStats {
    fn check(&self, _other: &Self) -> bool {
        true
    }
}

#[tokio::main]
async fn main() {
    let mongo_pw =
        env::var("MONGO_PW").expect("No environment variable was set to connect to the database.");
    let address = env::var("MONGO_ADDRESS").expect("No environment variable was set to connect to the database.");

    let client_options = ClientOptions::builder()
        .credential(Some(
            Credential::builder()
                .username(Some("cc".to_string()))
                .password(Some(mongo_pw))
                .build(),
        ))
        .hosts(vec![ServerAddress::parse(address).unwrap()])
        .build();
    let client = Client::with_options(client_options).unwrap();

    let mut test_data = HashMap::new();

    for entry in WalkDir::new("./programs/good")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.file_type().is_file())
    {
        let content = fs::read_to_string(entry.path()).unwrap();
        let (input, _, _, program) = split_test_raw(&content);
        let mut io = TestIO::new(input);

        let mut path = diff_paths(entry.path(), "./programs/good").unwrap();
        path.set_extension("");
        let path =  path.to_str().unwrap().to_string().replace(['/', '\\'], "::");

        test_data.insert(path, Stats::new(program, &mut io));
    }

    // what is our current commit?
    // what are our previous good commits?

    let db = client.database("rust-compiler-construction");
    let benches = db.collection::<Document>("benches");

    let test = "e";
    let docs = vec![
        doc! { test: "asd" }
    ];

    benches.insert_many(docs, None).await?;

    for (test, stats) in test_data {
        println!("{test:?}: {stats:?}");
    }
}

impl Stats{
    fn new(program: &str, io: &mut impl IO) -> Self {
        let tempdir = TempDir::new("cc-bench").unwrap();
        let output = tempdir.path().join("output");

        let prg_concluded = parse_program(program).unwrap()
            .type_check().unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .explicate()
            .select()
            .add_liveness()
            .compute_interference()
            .color_interference()
            .assign_homes()
            .patch()
            .conclude();

        let (_, interpreter_stats) = prg_concluded.interpret_with_stats(io);

        let (entry, program) = prg_concluded.emit();

        let elf = ElfFile::new(entry, &program);
        let mut file = File::create(&output).unwrap();
        elf.write(&mut file);

        let bencher_stats = BStats::new(&output);

        Stats {
            bencher_stats,
            interpreter_stats,
        }
    }
}

impl BStats{
    fn new(output: &PathBuf) -> Self {
        BStats{
            binary_size: binary_size(output),
        }
    }
}

fn binary_size(output: &Path) -> usize {
    output.metadata().unwrap().len() as usize
}
