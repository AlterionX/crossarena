use gdnative::{
    Control,
    Container,
    init::{ClassBuilder, Property, PropertyHint, PropertyUsage},
    Label,
    NativeClass,
    NodePath,
    user_data::MutexData,
};

use crate::records::{Record, Records};

pub struct End {
    wave: NodePath,
    scoreboard: NodePath,
}

impl End {
    const WAVE_PATH: &'static str = "";
    const SCOREBOARD_PATH: &'static str = "";
}

impl Default for End {
    fn default() -> Self {
        Self {
            wave: Self::WAVE_PATH.into(),
            scoreboard: Self::SCOREBOARD_PATH.into(),
        }
    }
}

impl NativeClass for End {
    type Base = Control;
    type UserData = MutexData<End>;

    fn class_name() -> &'static str {
        "End"
    }

    fn init(owner: Self::Base) -> Self {
        Self::_init(owner)
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        builder.add_property(Property {
            name: "wave_path",
            default: NodePath::from_str(Self::WAVE_PATH),
            hint: PropertyHint::None,
            getter: |this: &Self| this.wave.new_ref(),
            setter: |this: &mut Self, path| this.wave = path,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "scoreboard_path",
            default: NodePath::from_str(Self::SCOREBOARD_PATH),
            hint: PropertyHint::None,
            getter: |this: &Self| this.scoreboard.new_ref(),
            setter: |this: &mut Self, path| this.scoreboard = path,
            usage: PropertyUsage::DEFAULT,
        });
    }
}

impl End {
    fn add_records(&self, owner: &Control, records: Vec<(bool, Record)>) {
        let scoreboard = unsafe {
            owner.get_node(self.scoreboard.new_ref()).and_then(|n| n.cast::<Container>())
        };
        if let Some(mut scoreboard) = scoreboard {
            for (idx, (_, r)) in records.into_iter().enumerate() {
                log::info!("Adding record to scoreboard.");
                unsafe {
                    let mut pos_label = Label::new();
                    pos_label.set_h_size_flags(Control::SIZE_EXPAND_FILL);
                    pos_label.set_align(Label::ALIGN_CENTER);
                    pos_label.set_text(format!("{}", idx + 1).into());

                    let mut wave_label = Label::new();
                    wave_label.set_h_size_flags(Control::SIZE_EXPAND_FILL);
                    wave_label.set_align(Label::ALIGN_CENTER);
                    wave_label.set_text(format!("{}", r.wave_num).into());

                    scoreboard.add_child(Some(pos_label.to_node()), false);
                    scoreboard.add_child(Some(wave_label.to_node()), false);
                }
            }
        }
    }
}

#[methods]
impl End {
    fn _init(_owner: Control) -> Self {
        Default::default()
    }

    #[export]
    fn _ready(&mut self, owner: Control) {
        let sorted_records = if let Some(records) = Records::get_autoload(unsafe { owner.to_node() }) {
            match records.map_mut(|record, _| record.sorted_records()) {
                Ok(v) => {
                    log::info!("Found records {:?}.", v);
                    v
                },
                Err(e) => {
                    log::info!("Failed to retrieve records! Encountered error: {:?}.", e);
                    vec![]
                },
            }
        } else {
            vec![]
        };

        let mut most_recent = None;
        for (is_most_recent, record) in sorted_records.iter() {
            if *is_most_recent {
                most_recent = Some(record.clone());
            }
        }
        let wave = unsafe {
            owner.get_node(self.wave.new_ref()).and_then(|n| n.cast::<Label>())
        };
        if let (Some(r), Some(mut wave)) = (most_recent, wave) {
            log::info!("Setting wave text.");
            unsafe {
                wave.set_text(format!("You survived until wave {}! Congratulations!", r.wave_num).into());
            }
        }

        self.add_records(&owner, sorted_records);
    }
}
