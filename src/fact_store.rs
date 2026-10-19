//! Abstraction over sources of measures ("fact stores").
//!
//! Each store knows how many measures it holds and can return a random one.
//! [`random_measure`] picks a store weighted by its size (so the overall draw is
//! uniform across every measure) and delegates. New sources (a remote API, a
//! themed pack, …) just implement [`FactStore`] and get added to [`stores`].

use interface::Measure;
use loco_rs::prelude::*;
use rand::Rng;
use sea_orm::{DatabaseConnection, PaginatorTrait, QuerySelect};

use crate::models::_entities::measures::Entity;

#[async_trait]
pub trait FactStore: Send + Sync {
    /// How many measures this store can offer.
    async fn count(&self, db: &DatabaseConnection) -> Result<usize>;
    /// A random measure from this store, or `None` if it is empty.
    async fn random_measure(&self, db: &DatabaseConnection) -> Result<Option<Measure>>;
}

/// Built-in measures from `config/measures.toml`.
struct ConfigStore;

#[async_trait]
impl FactStore for ConfigStore {
    async fn count(&self, _db: &DatabaseConnection) -> Result<usize> {
        Ok(crate::measures_config::defaults().len())
    }

    async fn random_measure(&self, _db: &DatabaseConnection) -> Result<Option<Measure>> {
        let defaults = crate::measures_config::defaults();
        if defaults.is_empty() {
            return Ok(None);
        }
        let idx = rand::thread_rng().gen_range(0..defaults.len());
        Ok(defaults.get(idx).cloned())
    }
}

/// User-added measures stored in the database.
struct DbStore;

#[async_trait]
impl FactStore for DbStore {
    async fn count(&self, db: &DatabaseConnection) -> Result<usize> {
        Ok(usize::try_from(Entity::find().count(db).await?).unwrap_or(0))
    }

    async fn random_measure(&self, db: &DatabaseConnection) -> Result<Option<Measure>> {
        let n = self.count(db).await?;
        if n == 0 {
            return Ok(None);
        }
        let offset = u64::try_from(rand::thread_rng().gen_range(0..n)).unwrap_or(0);
        Ok(Entity::find().offset(offset).one(db).await?.map(Into::into))
    }
}

/// All registered fact stores.
fn stores() -> Vec<Box<dyn FactStore>> {
    vec![Box::new(ConfigStore), Box::new(DbStore)]
}

/// Pick a random measure across all stores, weighting each store by its size so
/// every measure is equally likely.
pub async fn random_measure(db: &DatabaseConnection) -> Result<Measure> {
    let stores = stores();
    let mut counts = Vec::with_capacity(stores.len());
    let mut total = 0usize;
    for store in &stores {
        let c = store.count(db).await?;
        counts.push(c);
        total += c;
    }
    if total == 0 {
        return Err(Error::NotFound);
    }

    let mut pick = rand::thread_rng().gen_range(0..total);
    for (store, count) in stores.iter().zip(counts) {
        if pick < count {
            return store
                .random_measure(db)
                .await?
                .ok_or_else(|| Error::NotFound);
        }
        pick -= count;
    }
    Err(Error::NotFound)
}
