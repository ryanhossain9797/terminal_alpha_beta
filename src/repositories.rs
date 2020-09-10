pub mod covid_repository;
pub mod gfycat_repository;
pub mod info_repository;
pub mod notes_repository;
pub mod people_repository;
pub mod search_repository;
use super::*;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}
