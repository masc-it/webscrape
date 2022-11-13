use std::{path::Path};

pub fn save_screenshot(img_data: &Vec<u8>, file_path: &Path) -> Result<bool, String> {

    if file_path.exists() {
        return Err(String::from("File already exists."));
    }

    std::fs::write(file_path, img_data).unwrap();

    return Ok(true);
    
}

pub fn img_to_base64(img_data: &Vec<u8>) -> String {

    base64::encode(img_data)

}