use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use godot::{ToVariant, FromVariant};

#[derive(Serialize, Deserialize)]
#[derive(ToVariant, FromVariant)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    Raw,
    Unique,
    Ammo,
}

impl Category {
    fn max_in_stack(&self) -> u64 {
        match self {
            Category::Raw => 999,
            Category::Unique => 1,
            Category::Ammo => 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(ToVariant, FromVariant)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    category: Category,

    name: String,
    desc: String,
}

impl Item {
}

#[derive(Serialize, Deserialize)]
#[derive(ToVariant, FromVariant)]
#[derive(Debug, Clone)]
pub struct Drop {
    item: Item,
    range: (u64, u64),
}

impl Drop {
    fn generate_drop(&self, wave: u64) -> Stack {
        use rand::distributions::{Distribution, Uniform};
        Stack {
            item: self.item.clone(),
            count: Uniform::from((wave + self.range.0 * (wave - 1))..(self.range.1*wave)).sample(&mut rand::thread_rng()),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(ToVariant, FromVariant)]
#[derive(Debug, Clone)]
pub struct DropGroup(Vec<(Drop, f64)>);

impl DropGroup {
    fn generate_drop(&self, wave: u64) -> Option<Stack> {
        use rand::distributions::{Distribution, Uniform};
        let sum = self.0.iter().fold(0., |accum, (_, chance)| accum + *chance);
        let mut bucket = Uniform::new(0., sum).sample(&mut rand::thread_rng());
        for (drop, chance) in &self.0 {
            if bucket <= *chance {
                return Some(drop.generate_drop(wave));
            }
            bucket -= chance;
        }
        None
    }
}

#[derive(Serialize, Deserialize)]
#[derive(ToVariant, FromVariant)]
#[derive(Debug, Clone)]
pub struct DropTable(Vec<(DropGroup, u64)>);

impl DropTable {
    pub fn generate_drops(&self, wave: u64) -> Vec<Stack> {
        self.0
            .iter()
            .flat_map(|(group, count)| (0..*count).filter_map(move |_| group.generate_drop(wave)))
            .collect()
    }
}

#[derive(Debug)]
pub enum Error {
    Full,
    MismatchedItem(Stack),
    MismatchedItemGroup(Vec<Stack>),
    TooManyItems(Item, u64),
    NotEnoughItems(Item, u64),
}

#[derive(ToVariant, FromVariant)]
#[derive(Debug, Clone)]
pub struct Stack {
    item: Item,
    count: u64,
}

impl Stack {
    fn is_composed_of(&self, item: &Item) -> bool {
        &self.item == item
    }

    fn group_stacks(stacks: Vec<Self>) -> HashMap<Item, Vec<Self>> {
        let mut map = HashMap::new();
        for stack in stacks {
            if map.contains_key(&stack.item) {
                map.insert(stack.item.clone(), vec![stack]);
            } else {
                if let Some(bucketed_stacks) = map.get_mut(&stack.item) {
                    bucketed_stacks.push(stack);
                }
            }
        }
        map
    }

    fn collapse_stacks(mut stacks: Vec<Self>) -> Result<Vec<Self>, Error> {
        if stacks.len() == 0 {
            return Ok(stacks);
        }

        { // ensure all stacks have the same item
            let mut failed_check = true;
            let item = &stacks[1].item;
            for stack in &stacks {
                if !stack.is_composed_of(item) {
                    failed_check = false;
                    break;
                }
            }
            if failed_check {
                return Err(Error::MismatchedItemGroup(stacks));
            }
        }

        let stack_count = stacks.len();
        'base: for trailing_idx in (0..stack_count).rev() {
            let mut trailing = stacks.pop().expect("Trailing element exists.");
            for agglomerate in 0..trailing_idx {
                match Stack::merge(&mut stacks[agglomerate], trailing)? {
                    Some(remaining) => {
                        trailing = remaining;
                    },
                    None => {
                        continue 'base;
                    },
                }
            }
            stacks.push(trailing);
            break;
        }

        Ok(stacks)
    }

    fn merge(into: &mut Stack, mut from: Stack) -> Result<Option<Stack>, Error> {
        if into.item != from.item {
            return Err(Error::MismatchedItem(from));
        }

        if into.is_full() {
            return Ok(Some(from));
        }

        let num_can_merge = into.item.category.max_in_stack() - into.count;
        if from.count <= num_can_merge {
            into.count += from.count;
            Ok(None)
        } else {
            into.count += num_can_merge;
            from.count -= num_can_merge;
            Ok(Some(from))
        }
    }

    fn is_full(&self) -> bool {
        self.count >= self.item.category.max_in_stack()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Inventory {
    max_stacks: usize,
    stacks: HashMap<Item, Stack>,
}

impl Inventory {
    pub fn attempt_add(&mut self, new_stacks: Vec<Stack>) -> Option<Vec<Stack>> {
        let overflow: Vec<_> = new_stacks
            .into_iter()
            .filter_map(|new_stack| {
                if let Some(stack) = self.stacks.get_mut(&new_stack.item) {
                    Stack::merge(stack, new_stack).expect("Item to be equivalent.")
                } else {
                    self.stacks.insert(new_stack.item.clone(), new_stack);
                    None
                }
            })
            .collect();
        if overflow.len() != 0 {
            Some(overflow)
        } else {
            None
        }
    }

    fn count_items(&self, item: &Item) -> u64 {
        self.stacks.get(item).map_or(0, |stack| stack.count)
    }

    pub fn attempt_take(&mut self, item: Item, count: u64) -> Result<Stack, Error> {
        if count > item.category.max_in_stack() {
            let overflow = count - item.category.max_in_stack();
            return Err(Error::TooManyItems(item, overflow));
        }

        if count == 0 {
            return Ok(Stack {
                item,
                count: 0,
            });
        }

        let num_held = self.count_items(&item);
        if num_held < count {
            return Err(Error::NotEnoughItems(item, count - num_held))
        }

        self.stacks.get_mut(&item).expect("Stack to be present.").count -= count;

        Ok(Stack {
            item,
            count: 0,
        })
    }
}
