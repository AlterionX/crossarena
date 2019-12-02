use nalgebra as na;
use std::time::Instant;
use gdnative::{Node, Node2D, NodePath, PackedScene};

use crate::{util::conv, entity::{arena::spawn::Cache, enemy::Cfg as EnemyCfg}};

pub(super) struct Spawn {
    pos: na::Vector2<f64>,
    spawn_id: u64,
}

impl Spawn {
    pub fn spawn(self, root: &mut Node, cache: &Cache, target: NodePath) -> Result<Option<Node>, ()> {
        log::info!("Spawning enemy {} at {:?}.", self.spawn_id, self.pos);
        let enemy = if let Some(enemy) = cache.get_spawn(self.spawn_id) {
            enemy
        } else {
            return Err(());
        };
        let instance = if let Ok(enemy) = enemy.scene.lock(){
            enemy.instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
        } else {
            return Err(());
        };
        unsafe {
            root.add_child(instance, false);
            if let Some(mut instance) = instance.and_then(|instance| instance.cast::<Node2D>()) {
                instance.set_global_position(conv::na64_to_g(self.pos));
                EnemyCfg::call_set_target(instance.to_node(), target);
            }
        }
        Ok(instance)
    }
}

#[derive(Debug)]
pub(super) struct Wave {
    start: Instant,
    wave_num: u64,
}

impl Wave {
    pub(super) fn initial() -> Self {
        Self {
            start: Instant::now(),
            wave_num: 0,
        }
    }

    pub(super) fn successor(mut self) -> Self {
        self.start = Instant::now();
        self.wave_num += 1;
        self
    }

    pub(super) fn generate_spawns(&self, cache: &Cache, pos: na::Vector2<f64>, dim: na::Vector2<f64>) -> Vec<Spawn> {
        use rand::{Rng, distributions::Uniform};
        let mut wave_value = self.value() as f64;
        let enemy_models = cache.available_units(self.wave_num);
        let mut spawns = vec![];
        let uniform = Uniform::new_inclusive(0., 1.);
        let mut rng = rand::thread_rng();
        while wave_value > 0. {
            use rand::seq::SliceRandom;
            if let Some(&(id, picked)) = enemy_models.choose(&mut rand::thread_rng()) {
                wave_value -= picked.cfg.value;
                // TODO sample
                spawns.push(Spawn {
                    pos: na::Vector2::new(
                         rng.sample(uniform),
                         rng.sample(uniform),
                    ).component_mul(&dim) + pos,
                    spawn_id: id,
                });
            } else {
                break;
            }
        }
        spawns
    }

    pub(super) fn value(&self) -> u64 {
        let a = self.wave_num * 2;
        a + 1
    }

    pub(super) fn num(&self) -> u64 {
        self.wave_num + 1
    }
}
