use gdnative::{NativeClass, init::{ClassBuilder, PropertyUsage}, user_data::MutexData,};

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

