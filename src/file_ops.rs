use std::{
    collections::HashMap,
    fs::{self, create_dir_all, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::OnceLockMethod;
use bincode;
use dirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

static VALID_FILE_TYPES: OnceLockMethod<HashMap<&str, FileType>> =
    OnceLockMethod::new(&|| -> HashMap<&str, FileType> {
        let mut map: HashMap<&str, FileType> = HashMap::new();
        for (key, value) in VALID_FILE_TYPE_VALUES.iter() {
            map.insert(key, *value);
        }
        return map;
    });
pub const VALID_FILE_TYPE_VALUES: [(&str, FileType); 2] =
    [("toml", FileType::Toml), ("bin", FileType::Bin)];

#[derive(Copy, Clone, Default, Deserialize, Serialize, PartialEq, Debug, Eq)]
pub enum FileType {
    #[default]
    Toml,
    Bin,
}
impl FileType {
    pub fn deserialize<T>(&self, bytes: Vec<u8>) -> Result<T, DeserializeFileTypeError>
    where
        T: Serialize + DeserializeOwned,
    {
        match self {
            // Deserializing does require the wrapped value to be stored before hand
            // It can't determine the implied type, so don't imply it
            FileType::Toml => {
                let wrapped: Result<T, toml::de::Error> = toml::from_slice(bytes.as_slice());
                match wrapped {
                    Err(error) => return Err(DeserializeFileTypeError::Toml(error)),
                    Ok(value) => return Ok(value),
                }
            }
            FileType::Bin => {
                let wrapped: Result<T, Box<bincode::ErrorKind>> =
                    bincode::deserialize(bytes.as_slice());
                match wrapped {
                    Err(error) => return Err(DeserializeFileTypeError::Bin(error)),
                    Ok(value) => return Ok(value),
                }
            }
        }
    }
    pub fn serialize<T>(&self, item: T) -> Result<Vec<u8>, SerializeFileTypeError>
    where
        T: Serialize + DeserializeOwned,
    {
        match self {
            // Serializing does not require the wrapped value to be explicitly defined type because it is by the arg
            FileType::Toml => match toml::to_vec(&item) {
                Ok(value) => return Ok(value),
                Err(error) => return Err(SerializeFileTypeError::Toml(error)),
            },
            FileType::Bin => match bincode::serialize(&item) {
                Ok(value) => return Ok(value),
                Err(error) => return Err(SerializeFileTypeError::Bin(error)),
            },
        }
    }
    pub fn from_path<P: AsRef<Path>>(path: &P) -> FileType {
        let extension: &str = path.as_ref().extension().unwrap().to_str().unwrap();
        let valid: HashMap<&str, FileType> = VALID_FILE_TYPES.get_or_init().to_owned().unwrap();
        *valid.get_key_value(extension).unwrap().1
    }
}
#[derive(Debug)]
pub enum DeserializeFileTypeError {
    Toml(toml::de::Error),
    Bin(Box<bincode::ErrorKind>),
}
#[derive(Debug)]
pub enum SerializeFileTypeError {
    Toml(toml::ser::Error),
    Bin(Box<bincode::ErrorKind>),
}

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq, Debug, Eq)]
pub enum Directory {
    None,
    Audio,
    Cache,
    Config,
    Data,
    LocalData,
    Desktop,
    Document,
    Download,
    Executable,
    Font,
    Home,
    Picture,
    Preference,
    Public,
    Runtime,
    Template,
    Video,
}
impl Default for Directory {
    fn default() -> Directory {
        Directory::new()
    }
}
impl Directory {
    pub const fn new() -> Directory {
        return Directory::None;
    }
    pub fn to_path_buf(&self) -> PathBuf {
        match self {
            Directory::None => return PathBuf::new(),
            Directory::Audio => return dirs::audio_dir().unwrap(),
            Directory::Cache => return dirs::cache_dir().unwrap(),
            Directory::Config => return dirs::config_dir().unwrap(),
            Directory::Data => return dirs::data_dir().unwrap(),
            Directory::LocalData => return dirs::data_local_dir().unwrap(),
            Directory::Desktop => return dirs::desktop_dir().unwrap(),
            Directory::Document => return dirs::document_dir().unwrap(),
            Directory::Download => return dirs::download_dir().unwrap(),
            Directory::Executable => return dirs::executable_dir().unwrap(),
            Directory::Font => return dirs::font_dir().unwrap(),
            Directory::Home => return dirs::home_dir().unwrap(),
            Directory::Picture => return dirs::picture_dir().unwrap(),
            Directory::Preference => return dirs::preference_dir().unwrap(),
            Directory::Public => return dirs::public_dir().unwrap(),
            Directory::Runtime => return dirs::runtime_dir().unwrap(),
            Directory::Template => return dirs::template_dir().unwrap(),
            Directory::Video => return dirs::video_dir().unwrap(),
        }
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq, Debug, Eq)]
pub struct FileOptions {
    pub create_missing_directories: bool,
    pub create_missing_files: bool,
    /// Load only
    pub reset_invalid_deserialization: bool,
    /// Save only
    pub truncate_existing_files: bool,
    pub start_location: Directory,
}
impl Default for FileOptions {
    fn default() -> FileOptions {
        FileOptions::new()
    }
}
impl FileOptions {
    pub const fn new() -> FileOptions {
        return FileOptions {
            create_missing_directories: true,
            create_missing_files: true,
            reset_invalid_deserialization: true,
            truncate_existing_files: true,
            start_location: Directory::new(),
        };
    }
    // These methods can't be const because const functions can't take mutable references
    pub fn create_missing_directories(&mut self, value: bool) -> &mut FileOptions {
        self.create_missing_directories = value;
        return self;
    }
    pub fn create_missing_files(&mut self, value: bool) -> &mut FileOptions {
        self.create_missing_files = value;
        return self;
    }
    pub fn reset_on_invalid_deserialization(&mut self, value: bool) -> &mut FileOptions {
        self.reset_invalid_deserialization = value;
        return self;
    }
    pub fn truncate_existing_files(&mut self, value: bool) -> &mut FileOptions {
        self.truncate_existing_files = value;
        return self;
    }

    pub fn generate_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path: PathBuf = self.start_location.to_path_buf().join(path);
        if self.create_missing_directories {
            if let Some(dir_path) = path.parent() {
                create_dir_all(dir_path).unwrap()
            }
        }
        return path;
    }
    pub fn load<T, P>(&self, path: P) -> T
    where
        P: AsRef<Path>,
        T: Serialize + DeserializeOwned + Default,
    {
        let path: PathBuf = self.generate_path(path);
        let file_type: FileType = FileType::from_path(&path);
        let mut file: File = OpenOptions::new()
            .create(self.create_missing_files)
            .truncate(false)
            .read(true)
            .write(self.reset_invalid_deserialization)
            .open(path)
            .unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let wrapped: Result<T, DeserializeFileTypeError> = file_type.deserialize(content);
        match wrapped {
            Err(error) => {
                if !self.reset_invalid_deserialization {
                    panic!("{:?}", error);
                }
                file.set_len(0).unwrap();
                file.write_all(toml::to_string(&T::default()).unwrap().as_bytes())
                    .unwrap();
                let mut content: Vec<u8> = Vec::new();
                file.read_to_end(&mut content).unwrap();
                return file_type.deserialize(content).unwrap();
            }
            Ok(value) => return value,
        }
    }
    pub fn save<T, P>(&self, item: T, path: P) -> Result<(), SerializeFileTypeError>
    where
        T: Serialize + DeserializeOwned,
        P: AsRef<Path>,
    {
        let path: PathBuf = self.generate_path(path);
        let file_type: FileType = FileType::from_path(&path);
        let mut file: File = OpenOptions::new()
            .create(self.create_missing_files)
            .truncate(self.truncate_existing_files)
            .read(false)
            .write(true)
            .open(path)
            .unwrap();
        let content: Vec<u8>;
        match file_type.serialize(item) {
            Ok(value) => content = value,
            Err(error) => return Err(error),
        }
        let content: &[u8] = content.as_slice();
        file.write_all(content).unwrap();
        return Ok(());
    }
}

/// This function is used to save an implied type to a .bin file.
/// If the file already exists, it will be overriden.
/// If the file does not exist, it will be created.
/// The data can be of any type, but it needs to implement [Serialize] and [Deserialize].
/// If the file cannot be opened or made, it will panic.
/// If the data cannot be [serialized](bincode::serialize), it will panic.
/// If the type doesn't implement [Deserialize], it it will panic.
/// That may seem confusing, but if it doesn't implement [Deserialize], how are you going to read the data?
/// For example, to store an isize, you would:
/// ```
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// let x: isize = 5;
/// save_bin("example.bin", &x);
/// # let load: isize = load_bin("example.bin");
/// # delete("example.bin");
/// assert_eq!(load, x);
/// ```
/// Note that both [Serialize] and [Deserialize] are implemented for types included with rust.
///
/// Also note that all files must be of the .bin file type.
///
/// While storing an [isize] is nice, most of the time, you will be storing much larger data types.
/// Here is an example of storing a struct containing some basic fields.
/// ```
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// # use serde::{Serialize, Deserialize};
/// #[derive(Default, Serialize, Deserialize)]
/// # #[derive(Debug, PartialEq)]
/// struct Example {
///     x: isize,
///     y: String,
///     z: f32,
/// }
/// let example: Example = Default::default();
/// save_bin("example.bin", &example);
/// # let load: Example = load_bin("example.bin");
/// # delete("example.bin");
/// # assert_eq!(load, example);
/// ```
/// This would be more useful, however, it will not work if it contained a struct or enum of its own.
/// All, data must implement [Serialize] and [Deserialize].
/// Including the data inside of structs and enums.
/// For example:
/// ```
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// # use serde::{Serialize, Deserialize};
/// #[derive(Default, Serialize, Deserialize)]
/// # #[derive(Debug, PartialEq)]
/// struct Value {
///     x: isize
/// }
/// #[derive(Default, Serialize, Deserialize)]
/// # #[derive(Debug, PartialEq)]
/// struct Example {
///     x: isize,
///     y: Value,
/// }
/// let example: Example = Default::default();
/// save_bin("example.bin", &example);
/// # let load: Example = load_bin("example.bin");
/// # delete("example.bin");
/// # assert_eq!(load, example);
/// ```
/// Please make sure to have any sub data implement [Serialize] and [Deserialize].
/// Otherwise, it will not run, and will be caught by the compiler.
/// Even if the main data implements both.
/// For example:
/// ```compile_fail
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// # use serde::{Serialize, Deserialize};
/// #[derive(Default)]
/// # #[derive(Debug, PartialEq)]
/// struct Value {
///     x: isize
/// }
/// #[derive(Default, Serialize, Deserialize)]
/// # #[derive(Debug, PartialEq)]
/// struct Example {
///     x: isize,
///     y: Value,
/// }
/// let example: Example = Default::default();
/// save_bin("example.bin", &example);
/// ```
pub fn save_bin<T, P: AsRef<Path>>(path: P, data: &T)
where
    T: Serialize + DeserializeOwned,
{
    let file = File::create(path);
    match file {
        Ok(mut file) => {
            let serialized = bincode::serialize(data).expect("Serialization failed");
            file.write_all(&serialized)
                .expect("Failed to write data to file");
        }
        Err(e) => {
            panic!("Error opening file: {}", e)
        }
    }
}

/// This saves a struct to a .toml file.
/// The benefit of using toml is the readability and compatability.
/// toml files are very easy to read and modify for people.
/// They also are directly converted to struct instances.
///
/// For example,
/// ```no_run
/// struct Example {
///     x: isize,
///     y: String,
/// }
/// let example = Example {
///     x: -27,
///     y: "hello".to_owned(),
/// };
/// ```
/// would be stored as:
/// ```toml
/// x = -27
/// y = "hello"
/// ```
/// That may be confusing, after all, the first line in cargo.toml is [package]
/// but there are no brackets at all.
/// If there are structs in the struct you are storing,
/// the substruct's area is started with the name of the field in brackets.
/// For example:
/// ```no_run
/// # struct Example {
/// #     x: isize,
/// #     y: String,
/// # }
/// struct AnotherExample {
///     a: Example,
///     b: Option<usize>,
/// }
/// let example = AnotherExample {
///     a: Example {
///         x: -256,
///         y: "bonjour".to_owned(),
///     },
///     b: Some(34),
/// };
/// ```
/// Would be stored as:
/// ```toml
/// [a]
/// x = -256
/// y = "bonjour"
///
/// b: 34
/// ```
/// There are two things I would like to note:
/// first, the option is not shown visibly, instead,
/// it will look for the field name, if it finds it,
/// it will have whatever the value be effectively Some(whatever is in here).
/// Second, b is not in a block, but then how does it know it isn't a part of a?
/// It uses the empty line to show that it is not a part of a.
/// Similarly, that also allows for infinitely many sub structs.
/// They would just be shown as a new area without an empty line.
pub fn save_toml<T, P: AsRef<Path>>(path: P, data: &T)
where
    T: Serialize + DeserializeOwned,
{
    let file = File::create(path);
    match file {
        Ok(mut file) => {
            file.write_all(toml::to_string(data).unwrap().as_bytes())
                .unwrap();
        }
        Err(e) => {
            panic!("Error opening file: {}", e)
        }
    }
}

/// This function loads an implied type from a file.
/// If the file doesn't exist or can't be read, it will panic.
/// The type must implement both [Serialize] and [Deserialize].
///
/// Because it returns an implied type, it cannot be used directly as an implied value;
/// ```compile_fail
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// let data: String = "Example".to_owned();
/// save_bin("wont_work.bin", &data);
/// assert_eq!(data, load_bin("wont_work.bin"));
/// ```
/// This produces an error because the compiler does not know what type the loaded data will be when it loads.
/// Which is normally defined like this:
/// ```
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// let data: usize = 57;
/// save_bin("example.bin", &data);
/// let load: usize = load_bin("example.bin");
/// # delete("example.bin");
/// assert_eq!(data, load);
/// ```
/// Structs can be loaded the same way:
/// ```
/// # use abes_nice_things::file_ops::{save_bin, load_bin, delete};
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize, Default)]
/// # #[derive(Debug, PartialEq)]
/// struct Example {
///     x: usize,
///     y: Vec<String>,
/// }
/// let data: Example = Default::default();
/// save_bin("example.bin", &data);
/// let load: Example = load_bin("example.bin");
/// # delete("example.bin");
/// assert_eq!(data, load);
/// ```
/// It is worth noting that the struct(or anything you're loading) must implement both [Serialize] and [Deserialize].
pub fn load_bin<T, P: AsRef<Path>>(path: P) -> T
where
    T: Serialize + DeserializeOwned,
{
    let serialized = fs::read(path).expect("Failed to read data from file");
    return bincode::deserialize(&serialized).expect("Deserialization failed");
}

pub fn load_toml<T, P: AsRef<Path>>(path: P) -> T
where
    T: Serialize + DeserializeOwned,
{
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let out: T = toml::from_str(&content).unwrap();
    return out;
}
/// Loads the file, then reads the file.
/// If the [deserialization](Deserialize::deserialize) fails,
/// it will save the [default](Default) of the type,
/// then it will [load](load_toml) that.
/// The reason it loads the [File] instead of just returning the [default](Default) directly
/// is because it will catch more problems that way.
/// It is a more complex version of [load_toml].
/// Like [load_toml], it requires [Serialize] and [Deserialize] to be implemented.
/// Unlike [load_toml], it also requires [Default] to be implemented.
/// It is useful in situations like where you want to load a file,
/// but it might have been modified and you don't want that to cause a panic.
/// However, if there are any problems in [deserialization](Deserialize::deserialize),
/// it will overwrite the file to the [default](Default) of the type,
/// meaning that if you want only problematic fields to be replaced,
/// this won't be what you need.
/// Here is an example use case:
/// ```
/// # use toml;
/// # use serde::{Serialize, Deserialize};
/// # use abes_nice_things::file_ops::{load_toml_or_generate_default, delete};
/// # use std::fs::File;
/// # use std::io::Write;
/// # #[derive(Debug, PartialEq)]
/// #[derive(Serialize, Deserialize, Default)]
/// struct Example {
///     x: usize,
///     y: String,
/// }
/// let actual = toml::toml! {
///     x = -26
///     // Note that this isn't a valid usize
///     y = "Hello"
/// }
/// # .to_string();
/// # let actual = actual.as_bytes();
/// # File::create("example.toml").unwrap().write_all(actual).unwrap();
/// let load: Example = load_toml_or_generate_default("example.toml");
/// # delete("example.toml");
/// assert_eq!(load, Example::default());
/// ```
/// Because one of the fields was invalid, the file got overwritten to be the default.
pub fn load_toml_or_generate_default<T>(path: &str) -> T
where
    T: Serialize + DeserializeOwned + Default,
{
    if !valid(path) {
        save_toml(path, &T::default());
    }
    let mut file: File = File::open(path).unwrap();
    let mut content: String = String::new();
    file.read_to_string(&mut content).unwrap();
    match toml::from_str(&content) {
        Ok(data) => return data,
        Err(_) => {
            save_toml(path, &T::default());
            let out: T = load_toml(path);
            return out;
        }
    }
}
/// This removes a file of any file type.
/// It doesn't have much use except that I like having all the operations.
/// Also, I don't have to unwrap it
/// and [delete()] is shorter than [fs::remove_file()].unwrap()
///
/// TLDR:
/// [fs::remove_file] but no result and shorter to write
pub fn delete<P: AsRef<Path>>(path: P) {
    if let Err(err) = fs::remove_file(path) {
        panic!("Failed to delete file: {}", err)
    }
}

/// This checks if the path leads to a file.
/// If the path is invalid, it will return false.
/// If the path does not lead to a file, it will return false.
/// # Example:
/// ```
/// # use abes_nice_things::file_ops::{save_bin, delete, valid};
/// let data: bool = false;
/// save_bin("example.bin", &data);
/// assert!(valid("example.bin"));
/// # delete("example.bin");
/// ```
pub fn valid(path: &str) -> bool {
    let metadata = std::fs::metadata(path);
    if let Ok(metadata) = metadata {
        return metadata.is_file();
    }
    return false;
}
