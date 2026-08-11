use sqlx::SqlitePool;

use crate::domain::cart::Cart;
use crate::domain::catalog::ItemId;
use crate::domain::order::OrderDraft;

#[derive(Debug, thiserror::Error)]
pub enum PlaceOrderError {
    #[error("Not enough stock for {0}.")]
    OutOfStock(ItemId),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// The whole purchase, atomically: the order row, one line item per cart
/// line, the status row, and the inventory decrements. Any failure —
/// including insufficient stock discovered on the last line — rolls back
/// everything, because the transaction commits at the end or not at all.
///
/// Returns the new order id, which the database assigned. The original
/// fetched-and-bumped a counter row in its `sequence` table to get this
/// number; that entire class of code has no equivalent here.
pub async fn place(
    pool: &SqlitePool,
    username: &str,
    draft: &OrderDraft,
    cart: &Cart,
) -> Result<i64, PlaceOrderError> {
    let mut tx = pool.begin().await?;

    let total = cart.total();
    let card_type = draft.card_type.as_str();
    let order_id: i64 = sqlx::query_scalar!(
        r#"INSERT INTO orders (userid, orderdate,
               shipaddr1, shipaddr2, shipcity, shipstate, shipzip, shipcountry,
               billaddr1, billaddr2, billcity, billstate, billzip, billcountry,
               courier, totalprice,
               billtofirstname, billtolastname, shiptofirstname, shiptolastname,
               cardtype, locale)
           VALUES (?1, date('now'),
               ?2, NULL, ?3, ?4, ?5, ?6,
               ?7, NULL, ?8, ?9, ?10, ?11,
               'UPS', ?12,
               ?13, ?14, ?15, ?16,
               ?17, 'en_US')
           RETURNING orderid as "orderid!""#,
        username,
        draft.ship.address,
        draft.ship.city,
        draft.ship.state,
        draft.ship.zip,
        draft.ship.country,
        draft.bill.address,
        draft.bill.city,
        draft.bill.state,
        draft.bill.zip,
        draft.bill.country,
        total,
        draft.bill.first_name,
        draft.bill.last_name,
        draft.ship.first_name,
        draft.ship.last_name,
        card_type
    )
    .fetch_one(&mut *tx)
    .await?;

    for (index, line) in cart.lines().iter().enumerate() {
        let linenum = index as i64 + 1;
        let quantity = i64::from(line.quantity);

        sqlx::query!(
            "INSERT INTO lineitem (orderid, linenum, itemid, quantity, unitprice)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            order_id,
            linenum,
            line.item_id,
            quantity,
            line.unit_price
        )
        .execute(&mut *tx)
        .await?;

        // Decrement and stock-check in one statement: the WHERE clause
        // refuses to oversell, and zero rows affected means this purchase
        // cannot proceed. Returning the error drops `tx`, and a dropped
        // transaction rolls back — the order row and earlier decrements
        // vanish with it.
        let updated = sqlx::query!(
            "UPDATE inventory SET qty = qty - ?2 WHERE itemid = ?1 AND qty >= ?2",
            line.item_id,
            quantity
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(PlaceOrderError::OutOfStock(line.item_id.clone()));
        }
    }

    // 'P' for pending, like the original. (Its linenum here was the order id
    // — a 2002 oddity we decline to reproduce.)
    sqlx::query!(
        "INSERT INTO orderstatus (orderid, linenum, timestamp, status)
         VALUES (?1, 1, date('now'), 'P')",
        order_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(order_id)
}

/// Who placed an order — the ownership check for order pages. None means
/// no such order; the caller treats "not yours" and "not real" identically.
pub async fn owner(pool: &SqlitePool, order_id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT userid FROM orders WHERE orderid = ?1", order_id)
        .fetch_optional(pool)
        .await
}

use crate::domain::catalog::Cents;
use crate::domain::order::{OrderLine, OrderSummary};

pub async fn history(pool: &SqlitePool, username: &str) -> Result<Vec<OrderSummary>, sqlx::Error> {
    sqlx::query_as!(
        OrderSummary,
        r#"SELECT orderid as "id!: i64", orderdate as "date!: String",
                  totalprice as "total!: Cents"
           FROM orders WHERE userid = ?1 ORDER BY orderid DESC"#,
        username
    )
    .fetch_all(pool)
    .await
}

/// Existence and ownership answered by one query: the WHERE clause only
/// matches this user's order, so "not yours" and "not real" are both None —
/// the same 404 upstream, decided in SQL instead of two round trips.
pub async fn summary_for(
    pool: &SqlitePool,
    order_id: i64,
    username: &str,
) -> Result<Option<OrderSummary>, sqlx::Error> {
    sqlx::query_as!(
        OrderSummary,
        r#"SELECT orderid as "id!: i64", orderdate as "date!: String",
                  totalprice as "total!: Cents"
           FROM orders WHERE orderid = ?1 AND userid = ?2"#,
        order_id,
        username
    )
    .fetch_optional(pool)
    .await
}

/// Lines joined with the catalog for display names — but the price comes
/// from lineitem, where lesson 10 froze it, not from the item table.
pub async fn lines(pool: &SqlitePool, order_id: i64) -> Result<Vec<OrderLine>, sqlx::Error> {
    sqlx::query_as!(
        OrderLine,
        r#"SELECT li.itemid as "item_id: ItemId", p.name as "name!",
                  i.attr1 as "attribute", li.quantity as "quantity!: i64",
                  li.unitprice as "unit_price!: Cents"
           FROM lineitem li
           JOIN item i ON i.itemid = li.itemid
           JOIN product p ON p.productid = i.productid
           WHERE li.orderid = ?1 ORDER BY li.linenum"#,
        order_id
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::catalog::Cents;
    use crate::domain::order::{Address, CardType};

    async fn test_pool() -> SqlitePool {
        crate::db::pool("sqlite::memory:").await.expect("test db")
    }

    fn draft() -> OrderDraft {
        let addr = Address {
            first_name: "ABC".into(),
            last_name: "XYX".into(),
            address: "901 San Antonio Road".into(),
            city: "Palo Alto".into(),
            state: "CA".into(),
            zip: "94303".into(),
            country: "USA".into(),
        };
        OrderDraft { ship: addr.clone(), bill: addr, card_type: CardType::Visa }
    }

    fn item(id: &str) -> ItemId {
        ItemId::try_from(id.to_string()).unwrap()
    }

    fn cart() -> Cart {
        let mut c = Cart::default();
        c.add(item("EST-1"), "Large Angelfish".into(), Cents(1650));
        c.add(item("EST-1"), "Large Angelfish".into(), Cents(1650));
        c.add(item("EST-20"), "Adult Male Goldfish".into(), Cents(550));
        c
    }

    #[tokio::test]
    async fn placing_writes_order_lines_status_and_inventory() {
        let pool = test_pool().await;
        let id = place(&pool, "j2ee", &draft(), &cart()).await.unwrap();

        let total: i64 = sqlx::query_scalar("SELECT totalprice FROM orders WHERE orderid = ?1")
            .bind(id).fetch_one(&pool).await.unwrap();
        assert_eq!(total, 2 * 1650 + 550);

        let lines: i64 = sqlx::query_scalar("SELECT count(*) FROM lineitem WHERE orderid = ?1")
            .bind(id).fetch_one(&pool).await.unwrap();
        assert_eq!(lines, 2);

        let qty: i64 = sqlx::query_scalar("SELECT qty FROM inventory WHERE itemid = 'EST-1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(qty, 9998);

        assert_eq!(owner(&pool, id).await.unwrap().as_deref(), Some("j2ee"));
    }

    #[tokio::test]
    async fn out_of_stock_rolls_the_whole_order_back() {
        let pool = test_pool().await;
        // One goldfish left; the cart wants EST-1 twice (fine) and EST-20
        // once — but we drain EST-20 to zero first.
        sqlx::query("UPDATE inventory SET qty = 0 WHERE itemid = 'EST-20'")
            .execute(&pool).await.unwrap();

        let err = place(&pool, "j2ee", &draft(), &cart()).await.unwrap_err();
        assert!(matches!(err, PlaceOrderError::OutOfStock(_)));

        // Nothing survived: no order, no line items, and EST-1's decrement
        // was rolled back along with everything else.
        let orders: i64 = sqlx::query_scalar("SELECT count(*) FROM orders")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(orders, 0);
        let est1: i64 = sqlx::query_scalar("SELECT qty FROM inventory WHERE itemid = 'EST-1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(est1, 10000);
    }

    #[tokio::test]
    async fn history_and_detail_read_back_what_place_wrote() {
        let pool = test_pool().await;
        let id = place(&pool, "j2ee", &draft(), &cart()).await.unwrap();

        let mine = history(&pool, "j2ee").await.unwrap();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].total, Cents(2 * 1650 + 550));

        let order_lines = lines(&pool, id).await.unwrap();
        assert_eq!(order_lines.len(), 2);
        assert_eq!(order_lines[0].name, "Angelfish");
        assert_eq!(order_lines[0].subtotal(), Cents(3300));

        // Ownership folded into the query: ACID sees nothing.
        assert!(summary_for(&pool, id, "j2ee").await.unwrap().is_some());
        assert!(summary_for(&pool, id, "ACID").await.unwrap().is_none());
        assert!(history(&pool, "ACID").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn order_ids_come_from_the_database() {
        let pool = test_pool().await;
        let first = place(&pool, "j2ee", &draft(), &cart()).await.unwrap();
        let second = place(&pool, "ACID", &draft(), &cart()).await.unwrap();
        assert!(second > first);
    }
}
