// Should be run only one if user will use cache from system, or
// everytime when change pieces. But should be never call within game.

use super::utils::screen_region::ScreenRegion;
use std::collections::HashMap;
extern crate serde;
use std::io::Write;

use crate::utils::file_system::{FileSystem, RealFileSystem};
use crate::webwrapper::WrapperMode;

static CORRECT_THRESHOLD_KEYS: &[char] =
    &['p', 'r', 'q', 'k', 'b', 'n', 'P', 'R', 'Q', 'K', 'B', 'N'];

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct Config {
    platform: WrapperMode,
    region: ScreenRegion,
    thresholds: HashMap<char, f64>,
    custom_pieces: bool,
}

impl Config {
    // Create config from passed data.
    pub fn new(
        platform: WrapperMode,
        region: ScreenRegion,
        thresholds: HashMap<char, f64>,
        custom_pieces: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        validate_region(&region)?;
        validate_thresholds(&thresholds)?;
        validate_custom_pieces(custom_pieces, &platform, &mut RealFileSystem)?;

        Ok(Config {
            platform,
            region,
            thresholds,
            custom_pieces,
        })
    }

    // Load already created config from file.
    pub fn from_cache() -> Result<Self, Box<dyn std::error::Error>> {
        let path = ".config.json";
        let config: Config =
            serde_json::from_str(path).expect("Can not serialize deserialize config object!");
        validate_region(&config.region)?;
        validate_thresholds(&config.thresholds)?;
        validate_custom_pieces(config.custom_pieces, &config.platform, &mut RealFileSystem)?;
        Ok(config)
    }
}

pub fn save_config(
    config: &Config,
    fs: &mut dyn FileSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = ".config.json";
    let data = serde_json::to_string(config).expect("Config object could not be serialized!");
    let exists = fs.exists(file_path);
    let mut file = fs.open(file_path, exists)?;
    Ok(file.write_all(data.as_bytes())?)
}

fn validate_region(region: &ScreenRegion) -> Result<(), Box<dyn std::error::Error>> {
    let (x, y, width, height) = region.values();
    if width == 0 || height == 0 {
        return Err("Width and height must be non-zero".into());
    }
    if x < 0 || y < 0 {
        return Err("Coordinates must be non-negative".into());
    }

    Ok(())
}

fn validate_thresholds(thresholds: &HashMap<char, f64>) -> Result<(), Box<dyn std::error::Error>> {
    if thresholds.is_empty() {
        return Err("Thresholds map cannot be empty".into());
    }

    if thresholds.keys().len() != 12 {
        return Err("Thresholds should have exact 12 keys".into());
    }

    for (key, &value) in thresholds {
        if !CORRECT_THRESHOLD_KEYS.contains(key) {
            return Err(format!("Key '{}' is not a valid character", key).into());
        }
        if value <= 0.0 || value >= 2.0 {
            return Err(format!(
                "Threshold {} for key '{}' is out of range (0, 2)",
                value, key
            )
            .into());
        }
    }
    Ok(())
}

fn validate_custom_pieces(
    custom: bool,
    wrapper: &WrapperMode,
    fs: &mut dyn FileSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let pieces_dir = if custom {
        "pieces/custom".to_string()
    } else {
        format!("pieces/{}", wrapper)
    };

    if !fs.exists(&pieces_dir) {
        return Err("Custom pieces directory does not exist".into());
    }

    let entries = fs.read_dir(&pieces_dir)?;

    if entries.len() != 12 {
        return Err("Custom pieces directory must contain exactly 12 PNG files".into());
    }

    for entry in entries {
        let path = std::path::Path::new(&entry);
        if let Some(file_stem_os) = path.file_stem() {
            let file_stem = file_stem_os.to_str().unwrap();
            if file_stem.len() != 1 {
                return Err(format!("Filename '{}' is not a single character", file_stem).into());
            }
            let ch = file_stem.chars().next().unwrap();
            if !CORRECT_THRESHOLD_KEYS.contains(&ch) {
                return Err(
                    format!("Filename '{}' is not a valid piece identifier", file_stem).into(),
                );
            }
        } else {
            return Err("Could not determine filename".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::file_system::{FakeFile, TestFileSystem};
    use rstest::{fixture, rstest};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[fixture]
    fn valid_thresholds() -> HashMap<char, f64> {
        let mut thresholds = HashMap::new();
        for key in CORRECT_THRESHOLD_KEYS {
            thresholds.insert(*key, 1.0);
        }
        thresholds
    }

    #[fixture]
    fn valid_config(valid_thresholds: HashMap<char, f64>) -> Config {
        Config {
            platform: WrapperMode::Chesscom,
            thresholds: valid_thresholds,
            region: ScreenRegion::new(1, 1, 100, 200),
            custom_pieces: false,
        }
    }

    // add mock file to test fs
    fn add_fake_file(fs: &mut TestFileSystem, path: &str) {
        fs.files
            .insert(path.to_string(), Rc::new(RefCell::new(FakeFile::default())));
    }

    // mock dir with pngs
    fn add_fake_files(fs: &mut TestFileSystem, base_dir: &str, file_names: &[char]) {
        fs.files.insert(
            base_dir.to_string(),
            Rc::new(RefCell::new(FakeFile::default())),
        );
        for name in file_names {
            let file_path = format!("{}/{}.png", base_dir, name);
            add_fake_file(fs, &file_path);
        }
    }

    #[test]
    fn test_validate_custom_pieces_custom_true_valid() {
        let mut fs = TestFileSystem::new();
        let base_dir = "pieces/custom";
        add_fake_files(&mut fs, base_dir, CORRECT_THRESHOLD_KEYS);

        let result = validate_custom_pieces(true, &WrapperMode::Chesscom, &mut fs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_custom_pieces_custom_true_directory_missing() {
        let mut fs = TestFileSystem::new();
        let result = validate_custom_pieces(false, &WrapperMode::Chesscom, &mut fs);
        assert!(result.is_err(),);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Custom pieces directory does not exist"
        );
    }

    #[test]
    fn test_validate_custom_pieces_custom_true_incorrect_number_of_files() {
        let mut fs = TestFileSystem::new();
        let base_dir = "pieces/custom";
        let keys = vec!['p', 'r', 'q'];
        add_fake_files(&mut fs, base_dir, &keys);

        let result = validate_custom_pieces(true, &WrapperMode::Chesscom, &mut fs);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Custom pieces directory must contain exactly 12 PNG files"
        );
    }

    #[test]
    fn test_validate_custom_pieces_custom_true_invalid_filename() {
        let mut fs = TestFileSystem::new();
        let base_dir = "pieces/custom";
        let mut keys = Vec::from(CORRECT_THRESHOLD_KEYS);
        keys[0] = 'X';
        add_fake_files(&mut fs, base_dir, &keys);

        let result = validate_custom_pieces(true, &WrapperMode::Chesscom, &mut fs);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Filename 'X' is not a valid piece identifier"
        );
    }

    #[test]
    fn test_validate_custom_pieces_custom_false_valid() {
        let mut fs = TestFileSystem::new();
        let wrapper_str = "chesscom";
        let base_dir = format!("pieces/{}", wrapper_str);
        add_fake_files(&mut fs, &base_dir, CORRECT_THRESHOLD_KEYS);

        let result = validate_custom_pieces(false, &WrapperMode::Chesscom, &mut fs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_custom_pieces_custom_false_directory_missing() {
        let mut fs = TestFileSystem::new();
        let result = validate_custom_pieces(false, &WrapperMode::Chesscom, &mut fs);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Custom pieces directory does not exist"
        );
    }

    #[rstest]
    fn test_valid_thresholds(valid_thresholds: HashMap<char, f64>) {
        let result = validate_thresholds(&valid_thresholds);
        assert!(
            result.is_ok(),
            "Expected valid thresholds to pass validation"
        );
    }

    #[rstest]
    fn test_empty_thresholds() {
        let thresholds: HashMap<char, f64> = HashMap::new();
        let result = validate_thresholds(&thresholds);
        assert!(result.is_err(), "Empty map should return an error");
        assert_eq!(
            result.unwrap_err().to_string(),
            "Thresholds map cannot be empty"
        );
    }

    #[rstest]
    fn test_wrong_number_of_keys(mut valid_thresholds: HashMap<char, f64>) {
        valid_thresholds.remove(&'K');
        let result = validate_thresholds(&valid_thresholds);
        assert!(
            result.is_err(),
            "Map with wrong number of keys should return an error"
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "Thresholds should have exact 12 keys"
        );
    }

    #[rstest]
    fn test_invalid_key(mut valid_thresholds: HashMap<char, f64>) {
        valid_thresholds.remove(&'k');
        valid_thresholds.insert('x', 1.0);
        let result = validate_thresholds(&valid_thresholds);
        assert!(
            result.is_err(),
            "Map with invalid key should return an error"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Key 'x' is not a valid character"));
    }

    #[rstest]
    fn test_invalid_threshold_value_zero(mut valid_thresholds: HashMap<char, f64>) {
        valid_thresholds.insert('p', 0.0);
        let result = validate_thresholds(&valid_thresholds);
        assert!(result.is_err(), "Threshold value 0.0 should be invalid");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Threshold 0 for key 'p' is out of range"));
    }

    #[rstest]
    fn test_invalid_threshold_value_two(mut valid_thresholds: HashMap<char, f64>) {
        valid_thresholds.insert('k', 2.0);
        let result = validate_thresholds(&valid_thresholds);
        assert!(result.is_err(), "Threshold value 2.0 should be invalid");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Threshold 2 for key 'k' is out of range"));
    }

    #[test]
    fn test_region_validation_passed() {
        let region = ScreenRegion::new(100, 100, 800, 600);

        let result = validate_region(&region);
        assert!(result.is_ok(), "Region should be valid");
    }

    #[rstest]
    #[case(100, 100, 0, 600, "Width and height must be non-zero")]
    #[case(100, 100, 800, 0, "Width and height must be non-zero")]
    #[case(-25, 100, 800, 100, "Coordinates must be non-negative")]
    #[case(42, -1, 800, 100, "Coordinates must be non-negative")]
    fn test_region_validation_failed(
        #[case] x: i32,
        #[case] y: i32,
        #[case] width: u32,
        #[case] height: u32,
        #[case] err_msg: String,
    ) {
        let region = ScreenRegion::new(x, y, width, height);

        let result = validate_region(&region);
        assert_eq!(result.err().unwrap().to_string(), err_msg);
    }

    #[rstest]
    fn test_save_config_file_does_not_exist(valid_config: Config) {
        let mut fake_fs = TestFileSystem::new();
        let file_path = ".config.json";

        assert!(!fake_fs.exists(file_path));

        save_config(&valid_config, &mut fake_fs).unwrap();

        assert!(fake_fs.exists(file_path));

        let file = fake_fs.files.get(file_path).unwrap();
        let data = String::from_utf8(file.borrow().data.clone()).unwrap();
        let config_from_file: Config = serde_json::from_str(&data).unwrap();
        assert_eq!(valid_config, config_from_file);
    }

    #[rstest]
    fn test_save_config_file_exists(valid_config: Config) {
        let mut fake_fs = TestFileSystem::new();
        let file_path = ".config.json";

        let preexisting_file = Rc::new(RefCell::new(FakeFile::default()));
        fake_fs
            .files
            .insert(file_path.to_string(), preexisting_file.clone());

        assert!(fake_fs.exists(file_path));

        save_config(&valid_config, &mut fake_fs).unwrap();

        let file = fake_fs.files.get(file_path).unwrap();
        let data = String::from_utf8(file.borrow().data.clone()).unwrap();
        let config_from_file: Config = serde_json::from_str(&data).unwrap();
        assert_eq!(valid_config, config_from_file);
    }
}
