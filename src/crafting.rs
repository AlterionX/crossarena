use std::collections::HashMap;
use itertools::{Itertools, Either};

use crate::systems::items;

#[derive(Debug, Clone)]
pub struct Recipes {
    input: HashMap<items::Item, u64>,
    output: Vec<items::Stack>,
}

#[derive(Debug, Clone)]
pub enum Error {
    MissingItems {
        missing: Vec<(items::Item, u64)>,
        overflow: Option<Vec<items::Stack>>,
    },
    OverflowItems(Vec<items::Stack>),
}

impl Recipes {
    pub fn attempt_craft(&self, inv: &mut items::Inventory) -> Result<
        Option<Vec<items::Stack>>,
        Error
    > {
        let inputs: Vec<_> = self.input
            .iter()
            .map(|(item, count)| inv.attempt_take(item.clone(), *count))
            .collect();
        if inputs.iter().any(|res| res.is_err()) {
            let (add_back, errors): (Vec<_>, Vec<_>) = inputs
                .into_iter()
                .partition_map(|res| match res {
                    Ok(add_back) => Either::Left(add_back),
                    Err(e) => Either::Right(e),
                });
            let missing = errors
                .into_iter()
                .filter_map(|e| match e {
                    items::Error::TooManyItems(item, count) => {
                        log::error!("Crafting formula is incorrect! Request for {} of item {:?} exceeds maximum inventory load.", count, item);
                        None
                    },
                    items::Error::NotEnoughItems(item, count) => {
                        Some((item, count))
                    },
                    e => {
                        log::warn!("Unexpected error {:?}!", e);
                        None
                    }
                })
                .collect();
            let overflow = inv.attempt_add(add_back);
            return Err(Error::MissingItems {
                missing,
                overflow,
            });
        }

        Ok(inv.attempt_add(self.output.clone()))
    }
}
