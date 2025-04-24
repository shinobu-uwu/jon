use core::fmt::Display;

use alloc::{format, string::String};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<'a> {
    pub scheme: &'a str,
    pub path: &'a str,
}

impl<'a> Path<'a> {
    pub fn new(scheme: &'a str, path: &'a str) -> Self {
        Self { scheme, path }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.scheme, self.path)
    }
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(value: &'a str) -> Self {
        let mut parts = value.split(":");

        Self {
            scheme: parts.next().unwrap_or(""),
            path: parts.next().unwrap_or(""),
        }
    }
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.scheme, self.path)
    }
}
