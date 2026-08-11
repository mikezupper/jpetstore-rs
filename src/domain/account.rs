//! Registration data. The password is deliberately not a field here — it
//! travels separately, from form to hash to database, and never sits on a
//! struct that might get debug-logged or serialized.

pub struct NewAccount {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub phone: String,
}
