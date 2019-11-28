use gdnative::{NativeClass, init::{ClassBuilder, PropertyUsage}, user_data::MutexData,};

pub mod melee;
pub mod aim;
pub mod dash;

pub mod health;

pub mod inventory;

lazy_static::lazy_static! {
    pub static ref DEFAULT_USAGE: PropertyUsage = PropertyUsage::SCRIPT_VARIABLE | PropertyUsage::STORAGE | PropertyUsage::EDITOR;
}

// TODO Make a derive macro for this, perhaps.
pub trait EditorCfg {
    fn register_properties<T, G, GM>(
        builder: &ClassBuilder<T>,
        get: G,
        get_mut: GM,
    )
        where
            T: Send + NativeClass<UserData = MutexData<T>>,
            G: Clone + Fn(&T) -> &Self,
            GM: Clone + Fn(&mut T) -> &mut Self,
    ;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SystemViewError {
    MissingCache(&'static str),
    MissingData(&'static str),
}

impl std::fmt::Display for SystemViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCache(source) => write!(f, "Could not find cache in {} system!", source),
            Self::MissingData(source) => write!(f, "Could not find data in {} system!", source),
        }
    }
}

impl std::error::Error for SystemViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait System {
    type Cache;
    type Cfg;
    type Data;

    fn cfg(&self) -> &Self::Cfg {
        self.view().0
    }
    fn cfg_mut(&mut self) -> &mut Self::Cfg {
        self.view_mut().0
    }
    fn cache(&self) -> Option<&Self::Cache> {
        self.view().1
    }
    fn cache_mut(&mut self) -> Option<&mut Self::Cache> {
        self.view_mut().1
    }
    fn data(&self) -> Option<&Self::Data> {
        self.view().2
    }
    fn data_mut(&mut self) -> Option<&mut Self::Data> {
        self.view_mut().2
    }

    fn opt_view(&self) -> Option<(&Self::Cfg, &Self::Cache, &Self::Data)> {
        self.res_view().ok()
    }
    fn opt_view_mut(&mut self) -> Option<(&mut Self::Cfg, &mut Self::Cache, &mut Self::Data)> {
        self.res_view_mut().ok()
    }

    fn res_view(&self) -> Result<(&Self::Cfg, &Self::Cache, &Self::Data), SystemViewError> {
        let (cfg, cache, data) = self.view();
        let cache = cache.ok_or(SystemViewError::MissingCache(std::any::type_name::<Self>()))?;
        let data = data.ok_or(SystemViewError::MissingData(std::any::type_name::<Self>()))?;
        Ok((cfg, cache, data))
    }
    fn res_view_mut(&mut self) -> Result<(&mut Self::Cfg, &mut Self::Cache, &mut Self::Data), SystemViewError> {
        let (cfg, cache, data) = self.view_mut();
        let cache = cache.ok_or(SystemViewError::MissingCache(std::any::type_name::<Self>()))?;
        let data = data.ok_or(SystemViewError::MissingData(std::any::type_name::<Self>()))?;
        Ok((cfg, cache, data))
    }

    fn view(&self) -> (&Self::Cfg, Option<&Self::Cache>, Option<&Self::Data>);
    fn view_mut(&mut self) -> (&mut Self::Cfg, Option<&mut Self::Cache>, Option<&mut Self::Data>);
}

impl<Cfg: EditorCfg, Sys: 'static + System<Cfg = Cfg>> EditorCfg for Sys {
    fn register_properties<T, G, GM>(
        builder: &ClassBuilder<T>,
        get: G,
        get_mut: GM,
    )
        where
            T: Send + NativeClass<UserData = MutexData<T>>,
            G: Clone + Fn(&T) -> &Self,
            GM: Clone + Fn(&mut T) -> &mut Self,
    {
        <Self as System>::Cfg::register_properties(
            builder,
            |this| get(this).cfg(),
            |this| get_mut(this).cfg_mut(),
        )
    }
}
