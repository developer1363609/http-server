use std::env::var;
use std::fs;
use std::path::Path;
use mime::Mime;

pub type Message = Vec<u8>;

pub fn load_content_from_uri(uri:&str) -> Result<Message,std::io::Error>{
    let path = Path::new(uri);
    fs::read(path)
}

pub fn find_mime_type(filename:&str) -> Mime {
    let parts:Vec<&str> = filename.split('.').collect();
    let result = match parts.last() {
        None => mime::TEXT_PLAIN,
        Some(v) => match *v {
            "html" => mime::TEXT_HTML,
            "png" => mime::IMAGE_PNG,
            "jpg" => mime::IMAGE_JPEG,
            "json" => mime::APPLICATION_JSON,
            &_ => mime::TEXT_PLAIN
        }
    };
    result
}

pub fn build_content_type(mime:&Mime) -> String{
    format!("Content-Type: {}/{}\r\n",mime.type_(),mime.subtype())
}

#[cfg(test)]
mod tests {
    use std::arch::aarch64::vrev16q_u8;
    use super::*;

    #[test]
    fn test_load_existing_file() {
        let url = "example/hello.html";
        let result = load_content_from_uri(&url);
        assert!(result.is_ok())
    }
}