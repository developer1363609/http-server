use std::fmt::{Display, Formatter};
use std::str::{from_utf8, FromStr};
use http::StatusCode;
use crate::http::content::{build_content_type, find_mime_type, load_content_from_uri, Message};
use crate::http::request::{HttpMethod, HttpRequest};

#[derive(Debug,Clone)]
pub struct ServerError{
    msg:String
}

impl ServerError{
    pub(crate) fn new(msg:&str) -> ServerError{
        ServerError{
            msg:String::from(msg)
        }
    }
}

impl Display for ServerError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.msg)
    }
}

pub trait Connection{
    fn listen<T:'static + Copy + Fn(&[u8]) -> Result<Vec<u8>,ServerError> + Send + Sync> (
        &self,
        callback:T
    );
}

pub struct Server<T> where T: Connection {
    connection: T,
}

impl <T:Connection> Server<T> {
    pub fn new(connection:T) -> Server<T>{
        Server{
            connection
        }
    }

    pub fn run(&self) {
        self.connection.listen(|request|Self::request_handler(request));
    }

    fn request_handler(request:&[u8]) -> Result<Message,ServerError>{
        from_utf8(request)
            .map_or_else(
                |_| {
                    Err(ServerError::new(
                        "Unable to convert request to utf8 format. Request rejected",
                    ))
                },
                |request| Ok(HttpRequest::from_str(request)),
            )?
            .map_or_else(
                |_| Ok(Self::build_not_implemented_response()),
                |http_request| match http_request.line.method {
                    HttpMethod::Get => Self::handle_get_request(&http_request),
                },
            )
    }

    fn handle_get_request(request:&HttpRequest) -> Result<Message,ServerError>{
        let mime = find_mime_type(&request.line.uri[1..]);
        load_content_from_uri(&request.line.uri[1..]).map_or_else(
            |_|Ok(Self::build_not_found_response()),
            |content|{
                let response = Self::build_http_response(200).unwrap();
                let content_type = build_content_type(&mime);
                let blank_line = "\r\n";
                let mut message = Vec::new();
                message.extend_from_slice(response.as_bytes());
                message.extend_from_slice(content_type.as_bytes());
                message.extend_from_slice(blank_line.as_bytes());
                message.extend_from_slice(&content);
                Ok(message)
            },
        )
    }

    fn build_not_found_response() -> Message {
        load_content_from_uri("404.html").map_or_else(
            |_| {
                format!(
                    "{}\r\n404 - Page not found",
                    Self::build_http_response(404).unwrap()
                ).into_bytes()
            },
            |content|{
                let response = Self::build_http_response(404).unwrap();
                let blank_line = "\r\n";
                let mut message = Vec::new();
                message.extend_from_slice(response.as_bytes());
                message.extend_from_slice(blank_line.as_bytes());
                message.extend_from_slice(&content);
                message
            }
        )
    }

    fn build_http_response(status_code:u16) -> Result<String,ServerError> {
        StatusCode::from_u16(status_code).map_or_else(
            |_| Err(ServerError::new("Unknown status code")),
            |code|Ok(format!("HTTP/1.1 {} {} \r\n",status_code,code.as_str()))
        )
    }

    fn build_not_implemented_response() -> Message {
        format!("{}\r\n",Self::build_http_response(501).unwrap()).into_bytes()
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::cell::RefCell;
    struct TestConnection{
        pull_message:Vec<Vec<u8>>,
        push_message:RefCell<Vec<Vec<u8>>>
    }

    impl TestConnection{
        fn new() -> TestConnection{
            TestConnection{
                pull_message:vec![
                    String::from("1").as_bytes().to_vec(),
                    String::from("2").as_bytes().to_vec(),
                    String::from("3").as_bytes().to_vec(),
                ],
                push_message: RefCell::new(vec![])
            }
        }
    }

    impl Connection for TestConnection {
        fn listen<T: 'static + Copy + Fn(&[u8]) -> Result<Vec<u8>, ServerError> + Send + Sync>(&self, callback: T) {
            self.push_message
                .borrow_mut()
                .push(callback(&self.pull_message[0]).unwrap())
        }
    }

    #[test]
    fn pull_message(){
        let test_connection = TestConnection::new();
        test_connection.listen(|_|Ok(String::from("Test").as_bytes().to_vec()));
        assert_eq!(
            String::from("Test").as_bytes().to_vec(),
            test_connection.push_message.borrow()[0]
        );
    }

    #[test]
    fn test_load_non_existing_png_file() {
        let uri = "non_existing.png";
        let result = load_content_from_uri(&uri);
        assert!(result.is_err());
    }
}