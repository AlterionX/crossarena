use nalgebra as na;
use gdnative::{
    Node,
    NodePath,
    PackedScene,
    ResourceLoader,
    GodotString,
};
use tap::TapResultOps;
use std::{path::Path, fs::File, io, sync::{Arc, Mutex}};

use crate::{
    entity::{
        arena::wave::Wave,
        enemy::Cfg as EnemyCfg
    },
    systems::items,
    util::{error, path_ops},
};

#[derive(Debug)]
pub(super) struct Data {
    pub scene: Arc<Mutex<PackedScene>>,
    pub drops: items::DropTable,
    pub cfg: EnemyCfg,
}

impl Data {
    fn load_scene(dir: &Path) -> io::Result<Option<PackedScene>> {
        let mut enemy_name = if let Some(file_name) = dir.file_name() {
            file_name.to_os_string()
        } else {
            dir.canonicalize()?.file_name().expect("No `..` paths after canonicalizing.").to_os_string()
        };
        enemy_name.push(".tscn");
        let enemy_path = dir.join(enemy_name);
        log::info!("Loading enemy PackedScene from: {:?}.", enemy_path);
        Ok((|| {
            let enemy_scene_path: GodotString = enemy_path.to_str()?.to_string().into();
            let mut loader = ResourceLoader::godot_singleton();
            let loaded = loader.load(enemy_scene_path, "PackedScene".into(), true)?;
            loaded.cast()
        })())
    }

    fn read_enemy_cfg(sc: PackedScene) -> Option<EnemyCfg> {
        EnemyCfg::call_get_cfg(sc.instance(PackedScene::GEN_EDIT_STATE_INSTANCE)?)
    }

    fn read_drop_cfg(dir: &Path) -> Result<items::DropTable, error::JsonIOError> {
        let mut enemy_name = if let Some(file_name) = dir.file_name() {
            file_name.to_os_string()
        } else {
            dir.canonicalize()?.file_name().expect("No `..` paths after canonicalizing.").to_os_string()
        };
        enemy_name.push("_drops.json");
        let enemy_path = dir.join(enemy_name);
        log::info!("Loading enemy drop table from: {:?}.", enemy_path);
        Ok(json::from_reader(File::open(enemy_path)?)?)
    }

    pub(super) fn load_from(_: &Cfg, dir: &Path) -> Option<Self> {
        let scene = Self::load_scene(dir).ok()??;
        Some(Self {
            cfg: Self::read_enemy_cfg(scene.new_ref())?,
            drops: Self::read_drop_cfg(dir)
                .tap_ok(|drops| log::info!("Loaded drop table {:?} for enemy {:?}.", drops, dir))
                .ok()?,
            scene: Arc::new(Mutex::new(scene)),
        })
    }
}

unsafe impl Send for Data {}
unsafe impl Sync for Data {}

#[derive(Debug)]
pub(super) struct Cfg {
    pub dir: String,
    pub cfg_file: String,
}

impl Cfg {
    pub(super) const DIR: &'static str = "res://enemies";
    pub(super) const CFG_FILE: &'static str = "spawn_values.json";
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            dir: Self::DIR.into(),
            cfg_file: Self::CFG_FILE.to_string(),
        }
    }
}

impl Cfg {
    fn globalize_path(&mut self) {
        self.dir = path_ops::abs_asset(self.dir.clone());
    }
}

pub(super) struct Cache {
    templates: Vec<Data>,
}

impl Cache {
    // TODO Don't load everything at once.
    pub fn load_with(cfg: &Cfg) -> io::Result<Self> {
        use std::fs::read_dir;
        log::info!("Loading cache from: {:?}", cfg.dir);
        let templates = read_dir(&cfg.dir)?
            .map(|entry| entry.map(|e| e.path()).map_err(|e| e.into()))
            .map(|path| path.and_then(|p| {
                log::info!("Attempting to scan for path {:?}", p);
                Data::load_from(
                    cfg,
                    &p.canonicalize()?
                ).ok_or(io::Error::new(io::ErrorKind::NotFound, "Cannot load data."))
            }))
            .collect::<Result<_, _>>()?;
        let ret = Self {
            templates,
        };
        log::info!("Finished loading data. Loaded {} elements: {:?}", ret.templates.len(), ret.templates);
        Ok(ret)
    }

    pub fn get_spawn(&self, id: u64) -> Option<&Data> {
        self.templates.get(id as usize)
    }

    pub fn available_units(&self, wave: u64) -> Vec<(u64, &Data)> {
        self.templates
            .iter()
            .enumerate()
            .filter(|(_, t)| t.cfg.available_from <= wave)
            .map(|(id, t)| (id as u64, t))
            .collect()
    }
}

#[derive(Default)]
pub(super) struct System {
    pub cfg: Cfg,
    pub cache: Option<Cache>,
}

impl System {
    pub fn load_cache(&mut self) {
        self.cfg.globalize_path();
        match Cache::load_with(&self.cfg) {
            Ok(cache) => self.cache = Some(cache),
            Err(err) => log::error!("Could not load cache! Err: {:?}", err),
        }
    }

    pub fn spawn_wave(
        &self,
        wave: &Wave,
        mut world: Option<Node>,
        arena_dim: na::Vector2<f64>,
        arena_pos: na::Vector2<f64>,
        target: NodePath,
    ) -> Vec<Node> {
        if let (Some(cache), Some(world)) = (self.cache.as_ref(), world.as_mut()) {
            wave.generate_spawns(cache, arena_pos, arena_dim)
                .into_iter()
                .map(|s| s.spawn(world, cache, target.new_ref()))
                .filter_map(Result::ok)
                .filter_map(|x| x)
                .collect()
        } else {
            if self.cache.is_none() {
                log::warn!("Attempted to spawn wave {:?} while cache not loaded.", wave);
            }
            if world.is_none() {
                log::warn!("Attempted to spawn wave {:?} with invalid world.", wave);
            }
            vec![]
        }
    }
}
