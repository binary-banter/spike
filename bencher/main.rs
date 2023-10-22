use git2::{Commit, Oid, Repository};
use pathdiff::diff_paths;
use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::interpreter::{TestIO, IO};
use rust_compiler_construction::language::x86var::IStats;
use rust_compiler_construction::parser::parse_program;
use rust_compiler_construction::utils::split_test::split_test_raw;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tempdir::TempDir;
use walkdir::WalkDir;
use std::fs::read_to_string;
use mongodb::bson::{doc, Document, to_bson};
use mongodb::options::{ClientOptions, Credential, ServerAddress};
use mongodb::sync::{Client, Collection};

/// Stats gathered by the bencher.
#[derive(Debug, Serialize)]
struct BStats {
    binary_size: usize,
    // todo: speed?
}

/// Accumulated stats.
#[derive(Debug, Serialize)]
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

fn main() {
    let mongo_pw =
        env::var("MONGO_PW").expect("No environment variable was set to connect to the database.");
    let address = env::var("MONGO_ADDRESS")
        .expect("No environment variable was set to connect to the database.");

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

    let mut test_data = doc!();

    for entry in WalkDir::new("./programs/good")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.file_type().is_file())
    {
        let content = read_to_string(entry.path()).unwrap();
        let (input, _, _, program) = split_test_raw(&content);
        let mut io = TestIO::new(input);

        let mut path = diff_paths(entry.path(), "./programs/good").unwrap();
        path.set_extension("");
        let path = path
            .to_str()
            .unwrap()
            .to_string()
            .replace(['/', '\\'], "::");

        test_data.insert(path, to_bson(&Stats::new(program, &mut io)).unwrap());
    }

    let db = client.database("rust-compiler-construction");
    let benches = db.collection("benches");

    let repo = Repository::open(".").unwrap();
    let oid = repo.head().unwrap().target().unwrap();
    let commit = repo.find_commit(oid).unwrap();

    assert!(check_parents(&benches, &commit, &test_data));

    // write_commit(&benches, &commit, &test_data);
}

fn check_parents(benches: &Collection<Document>, commit: &Commit, test_data: &Document) -> bool {
    let mut failure = false;
    for parent in commit.parents(){
        let filter = doc!("commits.hash": parent.id().to_string());
        let options = None;
        if let Some(parent_data) = benches.find_one(filter, options).unwrap()  {
            let parent_data = parent_data.get_array("commits").unwrap().first().unwrap().as_document().unwrap().get_document("tests").unwrap();
            test_data.check(parent_data);
        } else{
            failure |= check_parents(benches, commit, test_data);
        };
    }
    !failure
}

impl Check for Document{
    fn check(&self, other: &Self) -> bool {
        todo!()
    }
}

fn write_commit(benches: &Collection<Document>, commit: &Commit<'_>, test_data: &Document) {
    let hash = commit.id().to_string();
    let time = commit.time().seconds();
    let summary = commit.summary().unwrap();

    let commit = doc!(
        "hash": hash,
        "summary": summary,
        "time": time,
        "tests": test_data
    );

    let filter = doc!();
    let update = doc! ("$push": {"commits": commit});
    let options = None;
    benches
        .find_one_and_replace(filter, update, options)
        .unwrap()
        .unwrap();
}

impl Stats {
    fn new(program: &str, io: &mut impl IO) -> Self {
        let tempdir = TempDir::new("cc-bench").unwrap();
        let output = tempdir.path().join("output");

        let prg_concluded = parse_program(program)
            .unwrap()
            .type_check()
            .unwrap()
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

impl BStats {
    fn new(output: &PathBuf) -> Self {
        BStats {
            binary_size: binary_size(output),
        }
    }
}

fn binary_size(output: &Path) -> usize {
    output.metadata().unwrap().len() as usize
}
