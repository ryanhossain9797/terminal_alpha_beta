use super::*;
pub mod discord;
pub mod telegram;

extern crate lazy_static;

use telegram::API;

///Any initialization required for setting up the Clients should go here
pub fn initialize() {
    //---Start the telegram API
    lazy_static::initialize(&API);
}
