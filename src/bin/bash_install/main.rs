abes_nice_things::windows!(compile_error!("Fuck Windows"));

use std::fs::File;
use std::io::{BufRead, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

const OVERRIDE_PATH: &str = "bash_install";
const OVERRIDE_METADATA_PATH: &str = "metadata";
const OVERRIDE_BASE_PATH: &str = "base";
const OVERRIDE_VERIFICATION_PATH: &str = "verification";
const OVERRIDE_EXTRACTOR_PATH: &str = "extractor";
const OVERRIDE_COLLECTOR_PATH: &str = "collector";
const IGNORE_PATH: &str = ".bash_install_ignore";

struct Settings {
    project_directory: PathBuf,
    language: Option<Language>,
    script_name: String,
    // Extract binary and delete source
    extract_binary: bool,
    binary_name: Option<String>,
}
impl Settings {
    const DEFAULT_SCRIPT_NAME: &str = "installer.bash";
    fn new() -> Settings {
        Settings {
            project_directory: PathBuf::from(".").canonicalize().unwrap(),
            language: None,
            script_name: Settings::DEFAULT_SCRIPT_NAME.to_string(),
            extract_binary: false,
            binary_name: None,
        }
    }
    fn do_all_the_things(&self) {
        let path = &self.project_directory;
        let script_name_binding = self.script_name.clone() + ".bash";
        let script_name = script_name_binding.as_str();
        let language = self.language.unwrap();
        println!("Creating script file ({script_name})");
        let mut script = File::create(script_name).unwrap();

        println!("Placing base file");
        language.create_base_file(path.clone(), &mut script);
        script.seek(SeekFrom::End(0)).unwrap();

        println!("Placing project metadata");
        language.get_metadata(self, &mut script);
        script.seek(SeekFrom::End(0)).unwrap();

        println!("Placing verifier");
        language.get_verification(path.clone(), &mut script);
        script.seek(SeekFrom::End(0)).unwrap();

        println!("Placing extractor");
        language.get_extractor(path.clone(), &mut script);
        script.seek(SeekFrom::End(0)).unwrap();

        println!("Placing exit\nDetermining data start point");
        let start_point = language.place_exit(&mut script);
        script.seek(SeekFrom::End(0)).unwrap();

        println!("Getting data");
        language.get_data(path.clone(), &mut script, script_name);
        script.seek(SeekFrom::End(0)).unwrap();

        println!("Placing start point");
        language.put_start_point(&mut script, start_point);
    }
}
#[derive(Clone, Copy, Debug)]
enum Language {
    Override,
    Rust,
}
impl Language {
    fn determine(path: PathBuf) -> Option<Language> {
        if override_available(path.clone(), false).is_ok_and(|x| x) {
            Some(Language::Override)
        } else if path.join("Cargo.toml").exists() {
            Some(Language::Rust)
        } else {
            None
        }
    }
    fn create_base_file(self, path: PathBuf, script: &mut File) {
        script
            .write_all(&match self {
                Language::Override => {
                    std::fs::read(path.join(OVERRIDE_PATH).join(OVERRIDE_BASE_PATH)).unwrap()
                }
                _ => "#!/bin/bash\n".as_bytes().to_vec(),
            })
            .unwrap()
    }
    fn get_metadata(self, settings: &Settings, script: &mut File) {
        if let Language::Override = self {
            assert!(
                std::process::Command::new(
                    settings
                        .project_directory
                        .join(OVERRIDE_PATH)
                        .join(OVERRIDE_METADATA_PATH)
                )
                .arg(settings.project_directory.as_path())
                .arg(settings.script_name.as_str())
                .spawn()
                .unwrap()
                .wait()
                .unwrap()
                .success(),
                "Failed to generate metadata"
            );
            return;
        }
        match self {
            Language::Override => {
                unreachable!("oopsie");
            }
            Language::Rust => {
                writeln!(script, "SCRIPT_NAME=\"{}\"", settings.script_name).unwrap();
                if settings.extract_binary {
                    writeln!(
                        script,
                        "EXTRACT=\"{}\"",
                        self.get_binary_name(settings.project_directory.clone())
                            .unwrap()
                    )
                    .unwrap();
                } else {
                    writeln!(script, "EXTRACT=\"\"").unwrap();
                }
            }
        }
    }
    fn get_verification(self, path: PathBuf, script: &mut File) {
        script
            .write_all(
                match self {
                    Language::Override => {
                        std::fs::read(path.join(OVERRIDE_PATH).join(OVERRIDE_VERIFICATION_PATH))
                            .unwrap()
                    }
                    Language::Rust => include_bytes!("rust/verification.bash").to_vec(),
                }
                .as_slice(),
            )
            .unwrap()
    }
    fn get_extractor(self, path: PathBuf, script: &mut File) {
        script
            .write_all(
                match self {
                    Language::Override => {
                        std::fs::read(path.join(OVERRIDE_PATH).join(OVERRIDE_EXTRACTOR_PATH))
                            .unwrap()
                    }
                    Language::Rust => include_bytes!("rust/extractor.bash").to_vec(),
                }
                .as_slice(),
            )
            .unwrap()
    }
    fn place_exit(self, script: &mut File) -> u64 {
        writeln!(script, "exit 0").unwrap();
        script.stream_position().unwrap()
    }
    fn get_data(self, path: PathBuf, script: &mut File, script_name: &str) {
        let mut ignore = vec![path.join(script_name), path.join("target")];
        if let Ok(file) = File::open(path.join(IGNORE_PATH)) {
            for line in std::io::BufReader::new(file).lines() {
                if let Ok(line) = line {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    ignore.push(path.join(line.trim()));
                }
            }
        }
        println!("ignore list: {ignore:?}");
        match self {
            Self::Override => {
                assert!(
                    std::process::Command::new(
                        path.join(OVERRIDE_PATH).join(OVERRIDE_COLLECTOR_PATH)
                    )
                    .arg(path)
                    .arg(script_name)
                    .status()
                    .unwrap()
                    .success(),
                    "Failed to get data"
                );
            }
            Self::Rust => self.get_data_helper(path.clone(), PathBuf::new(), script, &ignore),
        }
    }
    fn get_data_helper(
        self,
        absolute_path: PathBuf,
        relative_path: PathBuf,
        script: &mut File,
        ignore: &Vec<PathBuf>,
    ) {
        // Format:
        //  len of file: u64
        //  len of path: u32
        //  executable?: bool
        //  path
        //  data
        //
        // Total header length: 8 + 4 + 1 = 13 bytes
        println!(
            "Getting file data in {} ({})",
            relative_path.display(),
            absolute_path.display()
        );
        for entry in std::fs::read_dir(absolute_path.as_path()).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            let new_relative_path = relative_path.join(entry.file_name());
            if ignore.contains(&entry.path()) {
                continue;
            }
            if metadata.is_dir() {
                self.get_data_helper(entry.path(), new_relative_path, script, ignore);
            } else if metadata.is_file() {
                println!(
                    "Adding file: {} ({})",
                    relative_path.join(entry.file_name()).display(),
                    entry.path().display()
                );
                // data len
                script.write_all(&metadata.len().to_le_bytes()).unwrap();

                let new_relative_path = new_relative_path.into_os_string().into_string().unwrap();
                // path len
                script
                    .write_all(&(new_relative_path.len() as u32).to_le_bytes())
                    .unwrap();

                // Executable?
                script
                    .write_all(&[
                        if std::fs::metadata(entry.path())
                            .unwrap()
                            .permissions()
                            .mode()
                            & 0o100
                            != 0
                        {
                            1
                        } else {
                            0
                        },
                    ])
                    .unwrap();

                // Path
                script.write_all(new_relative_path.as_bytes()).unwrap();

                // Data
                // This does load the entire file into memory, but I don't care enough to make it
                // nicer
                script
                    .write_all(std::fs::read(entry.path()).unwrap().as_slice())
                    .unwrap();
            }
        }
    }
    fn put_start_point(self, script: &mut File, start_point: u64) {
        script.write_all(&start_point.to_le_bytes()).unwrap()
    }
    fn get_binary_name(self, path: PathBuf) -> Result<String, ()> {
        let mut binary_name = None;
        match self {
            Self::Override => {
                eprintln!("You must specify the binary when using an override");
                return Err(());
            }
            Self::Rust => {
                println!("Checking Cargo.toml for default-run field");
                for line in std::fs::read_to_string(path.join("Cargo.toml"))
                    .unwrap()
                    .lines()
                {
                    let line = line.trim();
                    if line.starts_with("default-run") {
                        println!("Found line with default-run: \"{line}\"");
                        let line = line.strip_suffix('"').unwrap();
                        binary_name = Some(line.split('"').last().unwrap().to_string());
                        break;
                    }
                }
                if binary_name.is_none() {
                    println!("Could not find default-run in Cargo.toml, defaulting to path name");
                    binary_name = Some(path.file_name().unwrap().to_str().unwrap().to_string());
                }
            }
        };
        if binary_name.is_none() {
            return Err(());
        }
        println!("Found binary name of {}", binary_name.as_ref().unwrap());
        Ok(binary_name.unwrap())
    }
}
impl std::str::FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_lowercase().as_str() {
            "override" => Language::Override,
            "rust" | "rs" | "crab" | "ferris" | "the_best_one" | "cheese_crab" | "rustacean" => {
                Language::Rust
            }
            other => return Err(format!("{other} is not a supported language")),
        })
    }
}

fn main() -> Result<(), ()> {
    let mut settings = Settings::new();
    let arg_parser: ArgParser<'_, Settings> = ArgParser::new()
        .arg(Arg::new(
            &["--language", "-l"],
            1,
            Some("Expected one argument after --language".to_string()),
            &|settings: &mut Settings, args| match args[0].parse() {
                Ok(language) => {
                    settings.language = Some(language);
                    Ok(())
                }
                Err(error) => {
                    eprintln!("{error}");
                    Err(())
                }
            },
        ))
        .arg(Arg::new(
            &["--path", "-p"],
            1,
            Some("Expected one argument after --path".to_string()),
            &|settings, mut args| {
                if std::fs::exists(&args[0]).is_ok_and(|exists| exists) {
                    settings.project_directory = args.pop().unwrap().into();
                    Ok(())
                } else {
                    eprintln!("Expected a valid path after --path");
                    Err(())
                }
            },
        ))
        .arg(Arg::new(
            &["--script-name", "--script", "--name", "-s"],
            1,
            Some("Expected one argument after --script_name".to_string()),
            &|settings, mut args| {
                settings.script_name = args.pop().unwrap();
                Ok(())
            },
        ))
        .arg(Arg::new(
            &["--binary", "--bin", "-b"],
            1,
            Some("Expected oen argument after --binary".to_string()),
            &|settings, mut args| {
                settings.binary_name = Some(args.pop().unwrap());
                Ok(())
            },
        ))
        .arg(Arg::new(&["--extract", "-e"], 0, None, &|settings, _| {
            settings.extract_binary = true;
            Ok(())
        }));
    arg_parser.parse(&mut settings)?;
    if let Some(name) = settings.project_directory.file_name() {
        settings.script_name = name.to_str().unwrap().to_string();
    }
    if let Some(Language::Override) = settings.language {
        if let Err(error) = override_available(settings.project_directory.clone(), true) {
            eprintln!("{error}");
            return Err(());
        }
    }
    if settings.language.is_none() {
        settings.language = Some(
            Language::determine(settings.project_directory.clone())
                .expect("Failed to determine parser"),
        );
    }
    if settings.binary_name.is_none() && settings.extract_binary {
        settings.binary_name = Some(
            settings
                .language
                .unwrap()
                .get_binary_name(settings.project_directory.clone())
                .unwrap(),
        );
    }
    println!("Parser is {:?}", settings.language.unwrap());
    settings.do_all_the_things();
    Ok(())
}
fn override_available(path: PathBuf, crash: bool) -> Result<bool, &'static str> {
    use std::fs::exists;
    let path = path.join(OVERRIDE_PATH);
    if !path.exists() {
        return if crash {
            Err("You are missing your override directory")
        } else {
            Ok(false)
        };
    }
    if !exists_and_is_executable(path.join(OVERRIDE_METADATA_PATH)) {
        return if crash {
            Err("The metadata file is either missing or non-executable")
        } else {
            Ok(false)
        };
    }
    if !exists(path.join(OVERRIDE_BASE_PATH)).is_ok_and(|x| x) {
        return if crash {
            Err("You are missing the base file")
        } else {
            Ok(false)
        };
    }
    if !exists(path.join(OVERRIDE_VERIFICATION_PATH)).is_ok_and(|x| x) {
        return if crash {
            Err("You are missing the verification file")
        } else {
            Ok(false)
        };
    }
    if !exists(path.join(OVERRIDE_EXTRACTOR_PATH)).is_ok_and(|x| x) {
        return if crash {
            Err("You are missing the extractor file")
        } else {
            Ok(false)
        };
    }
    if !exists_and_is_executable(path.join(OVERRIDE_COLLECTOR_PATH)) {
        return if crash {
            Err("The metadata file is either missing or non-executable")
        } else {
            Ok(false)
        };
    }
    Ok(true)
}
fn exists_and_is_executable(path: PathBuf) -> bool {
    if !path.exists() {
        return false;
    }
    std::fs::metadata(path.clone()).is_ok_and(|metadata| {
        println!("{}'s mode is {:o}", path.clone().display(), metadata.mode());
        (metadata.mode() & 0o001) != 0
    })
}

struct ArgParser<'a, Settings = ()> {
    args: Vec<Arg<'a, Settings>>,
}
impl<'a, Settings> ArgParser<'a, Settings> {
    const fn new() -> Self {
        ArgParser { args: Vec::new() }
    }
    fn arg(mut self, arg: Arg<'a, Settings>) -> Self {
        self.args.push(arg);
        self
    }
}
impl<'a, Settings> ArgParser<'a, Settings> {
    fn parse(&'a self, settings: &'a mut Settings) -> Result<(), ()> {
        let mut args = std::env::args();
        while let Some(arg) = args.next() {
            for valid in self.args.iter() {
                if !valid.keywords.contains(&arg) {
                    continue;
                }
                let mut collected = Vec::new();
                for _ in 0..valid.args {
                    let current = args.next();
                    if current.is_some() {
                        collected.push(current.unwrap());
                    } else if let Some(error) = &valid.error {
                        eprintln!("{error}");
                        return Err(());
                    } else {
                        // we don't care if there are missing args and will pass it as is
                        break;
                    }
                }
                if valid.error.is_some() {
                    assert_eq!(collected.len(), valid.args);
                }
                (valid.closure)(settings, collected)?;
            }
        }
        Ok(())
    }
}
type ArgParseClosure<'a, Settings> = &'a dyn Fn(&mut Settings, Vec<String>) -> Result<(), ()>;
struct Arg<'a, Settings> {
    keywords: Vec<String>,
    args: usize,
    // Errors get propagated and parsing stops
    closure: ArgParseClosure<'a, Settings>,
    // Error used when there are too few arguments, if put as none then it will just send over
    // however many it could get
    error: Option<String>,
}
impl<'a, Settings> Arg<'a, Settings> {
    fn new(
        keywords: &[impl ToString],
        args: usize,
        error: Option<String>,
        closure: ArgParseClosure<'a, Settings>,
    ) -> Self {
        Arg {
            keywords: keywords.iter().map(|keyword| keyword.to_string()).collect(),
            args,
            closure,
            error: error.map(|msg| msg.to_string()),
        }
    }
}
