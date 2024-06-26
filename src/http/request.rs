use std::fmt::{Display, Formatter};
use std::str::FromStr;
use regex::Regex;

#[derive(Debug,Clone)]
pub struct HttpRequestError{
    msg:String
}

impl HttpRequestError{
    fn new(msg:&str) -> HttpRequestError{
        HttpRequestError{
            msg:String::from(msg)
        }
    }
}

impl Display for HttpRequestError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.msg)
    }
}

pub enum HttpMethod{
    Get
}

impl FromStr for HttpMethod {
    type Err = HttpRequestError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            _ => Err(HttpRequestError::new("Unknown http method"))
        }
    }
}

pub enum HttpVersion{
    V11
}

impl FromStr for HttpVersion {
    type Err = HttpRequestError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(HttpVersion::V11),
            _ => Err(HttpRequestError::new("Unknown http version"))
        }
    }
}

pub struct HttpRequestLine{
    pub method:HttpMethod,
    pub uri:String,
    pub version:HttpVersion
}

impl FromStr for HttpRequestLine{
    type Err = HttpRequestError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r#"([A-Z]*) (.*) (HTTP/[1-9.]*)"#)
            .map_err(|_| HttpRequestError::new("Not able to parse input Http request"))?;
        let caps = match re.captures(s) {
            Some(caps) => caps,
            None => {
                return Err(HttpRequestError::new(
                    "No captures found when parsing Http request",
                ))
            }
        };
        let method = match caps.get(1) {
            Some(match_method) => HttpMethod::from_str(match_method.as_str())?,
            None => {
                return Err(HttpRequestError::new(
                    "Http method not found in request line",
                ))
            }
        };
        let uri = match caps.get(2) {
            Some(uri) => uri.as_str(),
            None => return Err(HttpRequestError::new("Http uri not found in request line")),
        };
        let version = match caps.get(3) {
            Some(match_method) => HttpVersion::from_str(match_method.as_str())?,
            None => {
                return Err(HttpRequestError::new(
                    "Http version not found in request line",
                ))
            }
        };
        Ok(HttpRequestLine {
            method,
            uri: String::from(uri),
            version,
        })
    }
}

pub struct HttpRequest{
    pub line:HttpRequestLine
}

impl FromStr for HttpRequest {
    type Err = HttpRequestError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line = HttpRequestLine::from_str(s)?;
        Ok(HttpRequest{
            line
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_get_method(){
        let method = "GET";
        let result = HttpMethod::from_str(method);
        assert!(matches!(result,Ok(HttpMethod::Get)))
    }
}