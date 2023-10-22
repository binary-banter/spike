use git2::{Commit, Repository};
use mongodb::bson;
use mongodb::bson::{doc, to_bson, Bson, Document};
use mongodb::options::{ClientOptions, Credential, ServerAddress};
use mongodb::sync::{Client, Collection};
use pathdiff::diff_paths;
use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::interpreter::{TestIO, IO};
use rust_compiler_construction::language::x86var::IStats;
use rust_compiler_construction::parser::parse_program;
use rust_compiler_construction::utils::split_test::split_test_raw;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::fs::File;
use std::path::Path;
use tempdir::TempDir;
use walkdir::WalkDir;

/// Stats gathered by the bencher.
// #[derive(Check)]
#[derive(Debug, Deserialize, Serialize)]
struct BStats {
    // #[lower_is_better]
    binary_size: usize,
    // todo: speed?
}

/// Accumulated stats.
// #[derive(Check)]
#[derive(Debug, Deserialize, Serialize)]
struct Stats {
    // #[recursive]
    bencher_stats: BStats,
    // #[recursive]
    interpreter_stats: IStats,
}

#[derive(Debug, Deserialize, Serialize)]
struct StatsPartial {
    bencher_stats: Option<BStatsPartial>,
    interpreter_stats: Option<IStatsPartial>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BStatsPartial {
    binary_size: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IStatsPartial {}

trait Check {
    type Partial;

    /// Checks whether `self` does not regress compared to `other`.
    fn check(&self, prev: &Self::Partial) -> bool;
}

impl Check for Stats {
    type Partial = StatsPartial;

    fn check(&self, prev: &Self::Partial) -> bool {
        if let Some(prev) = &prev.bencher_stats {
            if !self.bencher_stats.check(prev) {
                return false;
            }
        }
        if let Some(prev) = &prev.interpreter_stats {
            if !self.interpreter_stats.check(prev) {
                return false;
            }
        }

        true
    }
}

impl Check for BStats {
    type Partial = BStatsPartial;

    fn check(&self, prev: &Self::Partial) -> bool {
        if let Some(prev) = prev.binary_size {
            if !(self.binary_size <= prev) {
                eprint!(
                    "Statistic `binary_size` regressed from {prev:?} to {:?} in test ",
                    self.binary_size
                );
                return false;
            }
        }

        true
    }
}

impl Check for IStats {
    type Partial = IStatsPartial;

    fn check(&self, _prev: &Self::Partial) -> bool {
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

    let new_stats =
        bson::from_bson::<HashMap<String, Stats>>(Bson::Document(test_data.clone())).unwrap();

    assert!(check_parents(&benches, &commit, &new_stats));
    write_commit(&benches, &commit, &test_data);
}

fn check_parents(
    benches: &Collection<Document>,
    commit: &Commit,
    new_stats: &HashMap<String, Stats>,
) -> bool {
    let mut ok = true;
    for parent in commit.parents() {
        let filter = doc!("commits.hash": parent.id().to_string());
        let options = None;
        if let Some(parent_data) = benches.find_one(filter, options).unwrap() {
            let parent_data = parent_data
                .get_array("commits")
                .unwrap()
                .first()
                .unwrap()
                .as_document()
                .unwrap()
                .get_document("tests")
                .unwrap();

            let old_stats = bson::from_bson::<HashMap<String, StatsPartial>>(Bson::Document(
                parent_data.clone(),
            ))
            .unwrap();

            for (test_name, new_stats) in new_stats {
                if let Some(old_stats) = old_stats.get(test_name) {
                    if !new_stats.check(old_stats) {
                        ok = false;
                        eprintln!(
                            "`{test_name}` when comparing with parent `{}`.",
                            parent.id()
                        );
                    }
                }
            }
        } else {
            ok &= check_parents(benches, &parent, new_stats);
        };
    }
    ok
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
        .find_one_and_update(filter, update, options)
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
    fn new(output: &Path) -> Self {
        BStats {
            binary_size: binary_size(output),
        }
    }
}

fn binary_size(output: &Path) -> usize {
    output.metadata().unwrap().len() as usize
}
