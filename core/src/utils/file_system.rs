use std::collections::HashMap;
use std::io::Write;

use std::cell::RefCell;
use std::rc::Rc;

pub trait FileSystem {
    fn exists(&self, path: &str) -> bool;
    fn open(&mut self, path: &str, file_exists: bool) -> std::io::Result<Box<dyn Write>>;
    fn read_dir(&self, path: &str) -> std::io::Result<Vec<String>>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    fn open(&mut self, path: &str, file_exists: bool) -> std::io::Result<Box<dyn Write>> {
        if file_exists {
            Ok(Box::new(
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(path)?,
            ))
        } else {
            Ok(Box::new(
                std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)?,
            ))
        }
    }
    fn read_dir(&self, path: &str) -> std::io::Result<Vec<String>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| ext.eq_ignore_ascii_case("png"))
            {
                entries.push(path.to_string_lossy().into_owned());
            }
        }
        Ok(entries)
    }
}

#[derive(Default)]
pub struct FakeFile {
    pub data: Vec<u8>,
}

impl Write for FakeFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub type SharedFakeFile = Rc<RefCell<FakeFile>>;

pub struct FakeFileWriter {
    pub file: SharedFakeFile,
}

impl Write for FakeFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.borrow_mut().flush()
    }
}

pub struct TestFileSystem {
    pub files: HashMap<String, SharedFakeFile>,
}

impl TestFileSystem {
    pub fn new() -> Self {
        TestFileSystem {
            files: HashMap::new(),
        }
    }
}

impl FileSystem for TestFileSystem {
    fn exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
    fn open(&mut self, path: &str, file_exists: bool) -> std::io::Result<Box<dyn Write>> {
        if file_exists {
            if let Some(file) = self.files.get(path) {
                file.borrow_mut().data.clear();
                Ok(Box::new(FakeFileWriter { file: file.clone() }))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Not found",
                ))
            }
        } else {
            let file = Rc::new(RefCell::new(FakeFile::default()));
            self.files.insert(path.to_string(), file.clone());
            Ok(Box::new(FakeFileWriter { file }))
        }
    }
    fn read_dir(&self, path: &str) -> std::io::Result<Vec<String>> {
        let prefix = format!("{}/", path);

        let entries: Vec<String> = self
            .files
            .keys()
            .filter(|k| k.starts_with(&prefix) && k.to_lowercase().ends_with(".png"))
            .cloned()
            .collect();
        Ok(entries)
    }
}
