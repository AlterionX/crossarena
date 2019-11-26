use nalgebra as na;
use gdnative::{
    Node,
    PackedScene,
    ProjectSettings,
    ResourceLoader,
    GodotString,
};
use std::{fs::File, path::Path, io::{self, BufReader}, sync::{Arc, Mutex}};

use crate::{
    entity::{
        arena::wave::Wave,
        enemy::Cfg as EnemyCfg
    },
    util::error::JsonIOError,
};

#[derive(Debug)]
pub(super) struct Data {
    pub scene: Arc<Mutex<PackedScene>>,
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

    fn read_enemy_cfg(arena_cfg: &Cfg, dir: &Path) -> Result<EnemyCfg, JsonIOError> {
        let enemy_cfg_path = dir.join(&arena_cfg.cfg_file);
        log::info!("Loading enemy values from: {:?}.", &enemy_cfg_path);
        let file = File::open(enemy_cfg_path)?;
        let buf_reader = BufReader::new(file);
        Ok(json::from_reader(buf_reader)?)
    }

    pub(super) fn load_from(arena_cfg: &Cfg, dir: &Path) -> Result<Self, JsonIOError> {
        let scene = Self::load_scene(dir).and_then(|opt_scene| if let Some(scene) = opt_scene {
            Ok(scene)
        } else {
            Err(io::ErrorKind::NotFound.into())
        })?;
        Ok(Self {
            scene: Arc::new(Mutex::new(scene)),
            cfg: Self::read_enemy_cfg(arena_cfg, dir)?,
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
        let project_settings = ProjectSettings::godot_singleton();
        // Convert to system path.
        let sys_path = (&self.dir).into();
        self.dir = project_settings.globalize_path(sys_path).to_string();
    }
}

pub(super) struct Cache {
    templates: Vec<Data>,
}

impl Cache {
    // TODO Don't load everything at once.
    pub fn load_with(cfg: &Cfg) -> Result<Self, JsonIOError> {
        use std::fs::read_dir;
        log::info!("Loading cache from: {:?}", cfg.dir);
        let templates = read_dir(&cfg.dir)?
            .map(|entry| entry.map(|e| e.path()).map_err(|e| e.into()))
            .map(|path| path.and_then(|p| {
                log::info!("Attempting to scan for path {:?}", p);
                Data::load_from(
                    cfg,
                    &p.canonicalize()?
                )
            }))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            templates,
        })
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
    ) -> Vec<Node> {
        if let (Some(cache), Some(world)) = (self.cache.as_ref(), &mut world) {
            wave.generate_spawns(cache, arena_pos, arena_dim)
                .into_iter()
                .map(|s| s.spawn(world, cache))
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
