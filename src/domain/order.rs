//! Checkout data on its way to becoming an order. The draft lives in the
//! session between the form and the confirmation page; lesson 10 turns a
//! confirmed draft plus the cart into rows.
//!
//! Look at what OrderDraft doesn't have: a card number field. The checkout
//! form collects one (the original's flow does, and dropping the field from
//! the page would change what we're porting), but it dies at the end of the
//! handler that received it — there is no field to put it in, here or in
//! the schema. Absence is the strongest guarantee a type can make.

use serde::{Deserialize, Serialize};

use super::catalog::Cents;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

impl Address {
    pub fn is_complete(&self) -> bool {
        [
            &self.first_name,
            &self.last_name,
            &self.address,
            &self.city,
            &self.state,
            &self.zip,
            &self.country,
        ]
        .iter()
        .all(|field| !field.trim().is_empty())
    }
}

/// The card *brand* is not a secret — the original stores it and so do we.
/// An enum instead of a string because there are three of them, not
/// arbitrarily many, and a typo'd brand should fail at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Visa,
    MasterCard,
    AmericanExpress,
}

impl CardType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardType::Visa => "Visa",
            CardType::MasterCard => "MasterCard",
            CardType::AmericanExpress => "American Express",
        }
    }
}

impl TryFrom<String> for CardType {
    type Error = ();

    fn try_from(raw: String) -> Result<Self, ()> {
        match raw.as_str() {
            "Visa" => Ok(CardType::Visa),
            "MasterCard" => Ok(CardType::MasterCard),
            "American Express" => Ok(CardType::AmericanExpress),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDraft {
    pub ship: Address,
    pub bill: Address,
    pub card_type: CardType,
}

/// One row of order history. The date is a plain "YYYY-MM-DD" string —
/// SQLite stores it that way, the page shows it that way, and nothing in
/// between does date arithmetic, so a chrono dependency would be ceremony.
pub struct OrderSummary {
    pub id: i64,
    pub date: String,
    pub total: Cents,
}

/// A line read back from a placed order, joined with the catalog for its
/// display name. The price is the one lesson 10 wrote — history shows what
/// you paid, not what the item costs today.
pub struct OrderLine {
    pub item_id: crate::domain::catalog::ItemId,
    pub name: String,
    pub attribute: Option<String>,
    pub quantity: i64,
    pub unit_price: Cents,
}

impl OrderLine {
    pub fn subtotal(&self) -> Cents {
        self.unit_price.times(self.quantity as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full() -> Address {
        Address {
            first_name: "ABC".into(),
            last_name: "XYX".into(),
            address: "901 San Antonio Road".into(),
            city: "Palo Alto".into(),
            state: "CA".into(),
            zip: "94303".into(),
            country: "USA".into(),
        }
    }

    #[test]
    fn an_address_needs_every_field() {
        assert!(full().is_complete());
        let mut missing = full();
        missing.zip = "   ".into();
        assert!(!missing.is_complete());
    }

    #[test]
    fn card_types_parse_from_the_form_and_nothing_else() {
        assert_eq!(CardType::try_from("Visa".to_string()), Ok(CardType::Visa));
        assert_eq!(
            CardType::try_from("American Express".to_string()),
            Ok(CardType::AmericanExpress)
        );
        assert!(CardType::try_from("Diners Club".to_string()).is_err());
    }
}
