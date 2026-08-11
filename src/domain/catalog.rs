//! The catalog as types.
//!
//! The rule here is parse-don't-validate: raw strings from URLs or forms get
//! checked once, at construction, and everything past that point handles a
//! type that is valid by existence. There is no function in this app that
//! takes a "product id string" and hopes.

use std::fmt;

#[derive(Debug, thiserror::Error)]
#[error("not a valid id: {reason}")]
pub struct InvalidId {
    reason: &'static str,
}

// Matches the schema's varchar(10) id columns. The check is deliberately
// dumb — length and emptiness — because the goal is a boundary, not a
// format spec the seed data doesn't actually follow.
fn validate_id(raw: &str) -> Result<(), InvalidId> {
    if raw.is_empty() {
        return Err(InvalidId { reason: "empty" });
    }
    if raw.len() > 10 {
        return Err(InvalidId { reason: "longer than 10 characters" });
    }
    Ok(())
}

// One newtype per id the catalog knows. They wrap the same kind of string,
// and that's the point: the compiler now refuses to pass a product id where
// a category id belongs — the mix-up MyBatis mappers can't catch.

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct CategoryId(String);

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct ProductId(String);

// ItemId also derives serde because it rides inside the session cart;
// a serde newtype struct serializes as its inner value, so the cookie
// payload stays plain.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct ItemId(String);

macro_rules! id_impls {
    ($t:ident) => {
        impl $t {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $t {
            type Error = InvalidId;
            fn try_from(raw: String) -> Result<Self, InvalidId> {
                validate_id(&raw)?;
                Ok(Self(raw))
            }
        }
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id_impls!(CategoryId);
id_impls!(ProductId);
id_impls!(ItemId);

/// Money as integer cents (see migrations/0001_schema.sql). Arithmetic on
/// i64 is exact; Display is how a price becomes "$16.50" in a template,
/// and it's the only place in the app that knows about dollar signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct Cents(pub i64);

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}.{:02}", self.0 / 100, self.0 % 100)
    }
}

// The only arithmetic money supports is what the cart actually does:
// scale a unit price by a quantity, and sum subtotals. There is still no
// `Add` — nothing in the app adds two prices directly, so that operation
// doesn't exist yet.
//
// Both operations saturate instead of overflowing. Lesson 12's property
// tests found the panic here that no realistic catalog price triggers but
// the domain API permits; a cart total pinned at i64::MAX is absurd, but
// absurd beats aborted.
impl Cents {
    pub fn times(self, quantity: u32) -> Cents {
        Cents(self.0.saturating_mul(i64::from(quantity)))
    }
}

impl std::iter::Sum for Cents {
    fn sum<I: Iterator<Item = Cents>>(iter: I) -> Self {
        iter.fold(Cents(0), |acc, c| Cents(acc.0.saturating_add(c.0)))
    }
}

/// What a 2002 description string parses into. The seed data embeds
/// presentation in the data — `<image src="../images/fish1.gif">Salt Water
/// fish from Australia` — and the original's JSPs render it raw. We parse
/// it once, here, into structure; templates render the parts and escape
/// everything by default.
pub struct Description {
    /// Image filename ("fish1.gif"), if the legacy markup carried one.
    pub image: Option<String>,
    pub text: String,
}

pub fn parse_legacy_description(descn: &str) -> Description {
    let mut rest = descn;
    let mut image = None;

    if let Some(start) = rest.find(r#"<image src="../images/"#) {
        let after = &rest[start + r#"<image src="../images/"#.len()..];
        if let Some(end) = after.find(r#"">"#) {
            image = Some(after[..end].to_string());
            rest = &after[end + 2..];
        }
    }

    // The category rows also wrap their name in <font> tags. Drop any
    // remaining tags wholesale — this is legacy cleanup, not an HTML parser.
    let mut text = String::with_capacity(rest.len());
    let mut in_tag = false;
    for ch in rest.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => text.push(c),
            _ => {}
        }
    }

    Description { image, text: text.trim().to_string() }
}

pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub description: String,
}

pub struct Product {
    pub id: ProductId,
    pub category_id: CategoryId,
    pub name: String,
    pub description: String,
}

impl Product {
    pub fn description_parts(&self) -> Description {
        parse_legacy_description(&self.description)
    }
}

/// A purchasable item, joined with its live inventory count — the shape the
/// product page needs, which is the shape the query returns. attr1 is the
/// only attribute column the seed data uses; the other four stay in the
/// schema, unloved, exactly like the original.
pub struct Item {
    pub id: ItemId,
    pub product_id: ProductId,
    pub list_price: Cents,
    pub attribute: Option<String>,
    pub quantity: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_reject_garbage() {
        assert!(CategoryId::try_from(String::new()).is_err());
        assert!(ItemId::try_from("X".repeat(11)).is_err());
        assert!(ProductId::try_from("FI-SW-01".to_string()).is_ok());
    }

    #[test]
    fn cents_compare_exactly() {
        // No arithmetic impls yet — the cart adds them in lesson 7, when
        // something actually needs to add money.
        assert!(Cents(1650) > Cents(1649));
        assert_eq!(Cents(550), Cents(550));
    }

    #[test]
    fn cents_display_as_dollars() {
        assert_eq!(Cents(1650).to_string(), "$16.50");
        assert_eq!(Cents(529).to_string(), "$5.29");
        assert_eq!(Cents(200).to_string(), "$2.00");
    }

    #[test]
    fn legacy_descriptions_parse_into_parts() {
        let d = parse_legacy_description(
            r#"<image src="../images/fish1.gif">Salt Water fish from Australia"#,
        );
        assert_eq!(d.image.as_deref(), Some("fish1.gif"));
        assert_eq!(d.text, "Salt Water fish from Australia");

        // Category rows: icon image plus <font>-wrapped name.
        let d = parse_legacy_description(
            r#"<image src="../images/fish_icon.gif"><font size="5" color="blue"> Fish</font>"#,
        );
        assert_eq!(d.image.as_deref(), Some("fish_icon.gif"));
        assert_eq!(d.text, "Fish");

        // Plain text passes through untouched.
        let d = parse_legacy_description("Just words");
        assert_eq!(d.image, None);
        assert_eq!(d.text, "Just words");
    }
}
