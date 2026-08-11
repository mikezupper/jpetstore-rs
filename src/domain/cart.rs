//! The cart, as a plain value. It lives in the session (web/cart.rs decides
//! where), serializes with serde, and knows nothing about HTTP or SQL.
//!
//! Each line denormalizes the display name and unit price at add time —
//! same as the original, which parks whole Item objects in the session.
//! The price a buyer saw in the cart is the price checkout will honor;
//! lesson 9 returns to that decision when orders become real.

use serde::{Deserialize, Serialize};

use super::catalog::{Cents, ItemId};

/// One form input caps at four digits; the domain enforces the same limit
/// so a hand-crafted POST can't order four billion goldfish.
const MAX_QUANTITY: u32 = 9999;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartLine {
    pub item_id: ItemId,
    pub name: String,
    pub unit_price: Cents,
    pub quantity: u32,
}

impl CartLine {
    pub fn subtotal(&self) -> Cents {
        self.unit_price.times(self.quantity)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cart {
    lines: Vec<CartLine>,
}

impl Cart {
    /// Adding an item already in the cart bumps its quantity — the original
    /// behaves the same way.
    pub fn add(&mut self, item_id: ItemId, name: String, unit_price: Cents) {
        if let Some(line) = self.lines.iter_mut().find(|l| l.item_id == item_id) {
            line.quantity = (line.quantity + 1).min(MAX_QUANTITY);
        } else {
            self.lines.push(CartLine { item_id, name, unit_price, quantity: 1 });
        }
    }

    /// Setting a line to zero removes it; setting an item that isn't in the
    /// cart does nothing. Both rules keep the handler code free of cases.
    pub fn set_quantity(&mut self, item_id: &ItemId, quantity: u32) {
        if quantity == 0 {
            self.remove(item_id);
            return;
        }
        if let Some(line) = self.lines.iter_mut().find(|l| &l.item_id == item_id) {
            line.quantity = quantity.min(MAX_QUANTITY);
        }
    }

    pub fn remove(&mut self, item_id: &ItemId) {
        self.lines.retain(|l| &l.item_id != item_id);
    }

    pub fn lines(&self) -> &[CartLine] {
        &self.lines
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn total(&self) -> Cents {
        self.lines.iter().map(CartLine::subtotal).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> ItemId {
        ItemId::try_from(id.to_string()).unwrap()
    }

    fn cart_with_est1() -> Cart {
        let mut cart = Cart::default();
        cart.add(item("EST-1"), "Large Angelfish".into(), Cents(1650));
        cart
    }

    #[test]
    fn adding_the_same_item_merges_lines() {
        let mut cart = cart_with_est1();
        cart.add(item("EST-1"), "Large Angelfish".into(), Cents(1650));
        assert_eq!(cart.lines().len(), 1);
        assert_eq!(cart.lines()[0].quantity, 2);
        assert_eq!(cart.total(), Cents(3300));
    }

    #[test]
    fn zero_quantity_removes_the_line() {
        let mut cart = cart_with_est1();
        cart.set_quantity(&item("EST-1"), 0);
        assert!(cart.is_empty());
    }

    #[test]
    fn totals_sum_across_lines() {
        let mut cart = cart_with_est1();
        cart.add(item("EST-20"), "Adult Male Goldfish".into(), Cents(550));
        cart.set_quantity(&item("EST-1"), 3);
        assert_eq!(cart.total(), Cents(3 * 1650 + 550));
    }

    #[test]
    fn quantities_cap_instead_of_overflowing() {
        let mut cart = cart_with_est1();
        cart.set_quantity(&item("EST-1"), u32::MAX);
        assert_eq!(cart.lines()[0].quantity, 9999);
    }

    #[test]
    fn updating_an_absent_item_is_a_no_op() {
        let mut cart = cart_with_est1();
        cart.set_quantity(&item("EST-99"), 5);
        assert_eq!(cart.lines().len(), 1);
        assert_eq!(cart.lines()[0].quantity, 1);
    }
}
