use core::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<'a> {
    pub scheme: &'a str,
    pub path: &'a str,
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(value: &'a str) -> Self {
        let mut parts = value.split(":");

        Self {
            scheme: parts.next().unwrap(),
            path: parts.next().unwrap(),
        }
    }
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.scheme, self.path)
    }
}
