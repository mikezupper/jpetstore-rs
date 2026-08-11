//! Registration data. The password is deliberately not a field here — it
//! travels separately, from form to hash to database, and never sits on a
//! struct that might get debug-logged or serialized.

/// The account page's read model — flat, because it's one table's row.
pub struct AccountInfo {
    pub email: String,
    pub phone: String,
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

/// The personalization switches from the profile table — the "dated but
/// harmless" quirks the port plan promised to keep. favorite_category is a
/// raw string here; the home page parses it at the boundary like any other
/// category id.
pub struct Prefs {
    pub favorite_category: Option<String>,
    pub my_list: bool,
    pub banner: bool,
}

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
