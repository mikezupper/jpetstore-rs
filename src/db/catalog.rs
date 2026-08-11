//! Catalog queries. Plain async functions, per the port plan — no repository
//! trait until something (lesson 12's tests) creates the pressure for one.
//!
//! Every query here is a query_as! macro: the SQL is checked against the real
//! schema at compile time, and each column is mapped to a domain type with
//! the `as "name: Type"` override syntax. The `!` overrides assert that a
//! column the 2002 schema left nullable is in fact never null in this data.

use sqlx::SqlitePool;

use crate::domain::catalog::{Category, CategoryId, Cents, Item, ItemId, Product, ProductId};

pub async fn categories(pool: &SqlitePool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as!(
        Category,
        r#"SELECT catid as "id: CategoryId", name as "name!", descn as "description!"
           FROM category ORDER BY name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn category(pool: &SqlitePool, id: &CategoryId) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as!(
        Category,
        r#"SELECT catid as "id: CategoryId", name as "name!", descn as "description!"
           FROM category WHERE catid = ?1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn products_in_category(
    pool: &SqlitePool,
    category_id: &CategoryId,
) -> Result<Vec<Product>, sqlx::Error> {
    sqlx::query_as!(
        Product,
        r#"SELECT productid as "id: ProductId", category as "category_id: CategoryId",
                  name as "name!", descn as "description!"
           FROM product WHERE category = ?1 ORDER BY productid"#,
        category_id
    )
    .fetch_all(pool)
    .await
}

pub async fn product(pool: &SqlitePool, id: &ProductId) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as!(
        Product,
        r#"SELECT productid as "id: ProductId", category as "category_id: CategoryId",
                  name as "name!", descn as "description!"
           FROM product WHERE productid = ?1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn items_for_product(
    pool: &SqlitePool,
    product_id: &ProductId,
) -> Result<Vec<Item>, sqlx::Error> {
    sqlx::query_as!(
        Item,
        r#"SELECT i.itemid as "id: ItemId", i.productid as "product_id: ProductId",
                  i.listprice as "list_price!: Cents", i.attr1 as "attribute",
                  inv.qty as "quantity!: i64"
           FROM item i JOIN inventory inv ON inv.itemid = i.itemid
           WHERE i.productid = ?1 ORDER BY i.itemid"#,
        product_id
    )
    .fetch_all(pool)
    .await
}

pub async fn item(pool: &SqlitePool, id: &ItemId) -> Result<Option<Item>, sqlx::Error> {
    sqlx::query_as!(
        Item,
        r#"SELECT i.itemid as "id: ItemId", i.productid as "product_id: ProductId",
                  i.listprice as "list_price!: Cents", i.attr1 as "attribute",
                  inv.qty as "quantity!: i64"
           FROM item i JOIN inventory inv ON inv.itemid = i.itemid
           WHERE i.itemid = ?1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // An in-memory database, migrated and seeded by the exact code and SQL
    // production runs. Each test gets a fresh one — the seed data is the
    // fixture, and it can't drift from reality because it is reality.
    async fn test_pool() -> SqlitePool {
        crate::db::pool("sqlite::memory:").await.expect("test db")
    }

    fn id<T: TryFrom<String>>(raw: &str) -> T
    where
        T::Error: std::fmt::Debug,
    {
        T::try_from(raw.to_string()).expect("valid test id")
    }

    #[tokio::test]
    async fn five_categories_sorted_by_name() {
        let pool = test_pool().await;
        let cats = categories(&pool).await.unwrap();
        let names: Vec<_> = cats.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Birds", "Cats", "Dogs", "Fish", "Reptiles"]);
    }

    #[tokio::test]
    async fn fish_category_has_four_products() {
        let pool = test_pool().await;
        let products = products_in_category(&pool, &id("FISH")).await.unwrap();
        assert_eq!(products.len(), 4);
        assert!(products.iter().all(|p| p.category_id == id("FISH")));
    }

    #[tokio::test]
    async fn large_angelfish_costs_16_50() {
        let pool = test_pool().await;
        let est1 = item(&pool, &id("EST-1")).await.unwrap().expect("EST-1 exists");
        assert_eq!(est1.list_price, Cents(1650));
        assert_eq!(est1.attribute.as_deref(), Some("Large"));
        assert_eq!(est1.quantity, 10000);
    }

    #[tokio::test]
    async fn unknown_ids_are_none_not_errors() {
        let pool = test_pool().await;
        assert!(category(&pool, &id("NOPE")).await.unwrap().is_none());
        assert!(item(&pool, &id("EST-999")).await.unwrap().is_none());
    }
}
