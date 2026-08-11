use sqlx::SqlitePool;

use crate::domain::account::{AccountInfo, NewAccount, Prefs};
use crate::domain::order::Address;

pub async fn info(pool: &SqlitePool, username: &str) -> Result<Option<AccountInfo>, sqlx::Error> {
    sqlx::query_as!(
        AccountInfo,
        r#"SELECT email as "email!", phone as "phone!",
                  firstname as "first_name!", lastname as "last_name!",
                  addr1 as "address!", city as "city!", state as "state!",
                  zip as "zip!", country as "country!"
           FROM account WHERE userid = ?1"#,
        username
    )
    .fetch_optional(pool)
    .await
}

pub async fn prefs(pool: &SqlitePool, username: &str) -> Result<Option<Prefs>, sqlx::Error> {
    sqlx::query_as!(
        Prefs,
        r#"SELECT favcategory as "favorite_category", mylistopt as "my_list!: bool",
                  banneropt as "banner!: bool"
           FROM profile WHERE userid = ?1"#,
        username
    )
    .fetch_optional(pool)
    .await
}

/// The favorite category's banner image, already parsed out of the legacy
/// markup the bannerdata table stores.
pub async fn banner_image(
    pool: &SqlitePool,
    favcategory: &str,
) -> Result<Option<String>, sqlx::Error> {
    let legacy: Option<Option<String>> = sqlx::query_scalar!(
        "SELECT bannername FROM bannerdata WHERE favcategory = ?1",
        favcategory
    )
    .fetch_optional(pool)
    .await?;
    Ok(legacy
        .flatten()
        .and_then(|html| crate::domain::catalog::parse_legacy_description(&html).image))
}

/// The account's address block, shaped for prefilling checkout forms.
pub async fn address(pool: &SqlitePool, username: &str) -> Result<Option<Address>, sqlx::Error> {
    sqlx::query_as!(
        Address,
        r#"SELECT firstname as "first_name!", lastname as "last_name!",
                  addr1 as "address!", city as "city!", state as "state!",
                  zip as "zip!", country as "country!"
           FROM account WHERE userid = ?1"#,
        username
    )
    .fetch_optional(pool)
    .await
}

/// The stored PHC hash for a username, if the user exists. Sign-in decides
/// what to do with the two Nones (unknown user, wrong password) — this
/// function just fetches.
pub async fn password_hash(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT password FROM signon WHERE username = ?1", username)
        .fetch_optional(pool)
        .await
}

/// Registration writes three tables — signon, account, profile — atomically.
/// A duplicate username surfaces as a unique-constraint violation from the
/// first insert and rolls the whole thing back: the database is the arbiter
/// of uniqueness, so there is no check-then-insert race to lose.
pub async fn create(
    pool: &SqlitePool,
    account: &NewAccount,
    stored_hash: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "INSERT INTO signon (username, password) VALUES (?1, ?2)",
        account.username,
        stored_hash
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO account (userid, email, firstname, lastname, status,
                              addr1, addr2, city, state, zip, country, phone)
         VALUES (?1, ?2, ?3, ?4, 'OK', ?5, NULL, ?6, ?7, ?8, ?9, ?10)",
        account.username,
        account.email,
        account.first_name,
        account.last_name,
        account.address,
        account.city,
        account.state,
        account.zip,
        account.country,
        account.phone
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO profile (userid, langpref, favcategory, mylistopt, banneropt)
         VALUES (?1, 'english', NULL, 0, 0)",
        account.username
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error().is_some_and(|db| db.is_unique_violation())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        crate::db::pool("sqlite::memory:").await.expect("test db")
    }

    fn mike() -> NewAccount {
        NewAccount {
            username: "mike".into(),
            email: "mike@example.com".into(),
            first_name: "Mike".into(),
            last_name: "Zupper".into(),
            address: "1 Main St".into(),
            city: "Tampa".into(),
            state: "FL".into(),
            zip: "33601".into(),
            country: "USA".into(),
            phone: "555-555-0100".into(),
        }
    }

    #[tokio::test]
    async fn the_seeded_demo_user_verifies() {
        let pool = test_pool().await;
        let hash = password_hash(&pool, "j2ee").await.unwrap().expect("j2ee seeded");
        assert!(hash.starts_with("$argon2id$"));
        assert!(crate::auth::verify(&hash, "j2ee"));
        assert!(!crate::auth::verify(&hash, "J2EE"));
    }

    #[tokio::test]
    async fn registration_writes_and_reads_back() {
        let pool = test_pool().await;
        create(&pool, &mike(), "$argon2id$fake").await.unwrap();
        assert_eq!(
            password_hash(&pool, "mike").await.unwrap().as_deref(),
            Some("$argon2id$fake")
        );
    }

    #[tokio::test]
    async fn duplicate_usernames_are_a_unique_violation() {
        let pool = test_pool().await;
        create(&pool, &mike(), "h").await.unwrap();
        let err = create(&pool, &mike(), "h").await.unwrap_err();
        assert!(is_unique_violation(&err));
    }
}
