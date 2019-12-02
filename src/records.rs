use gdnative::{
    FromVariant,
    Instance,
    NativeClass,
    Node,
    ToVariant,
};

#[derive(ToVariant, FromVariant)]
#[derive(Debug, Clone)]
pub struct Record {
    pub wave_num: u64,
}

#[derive(Default, Debug)]
#[derive(NativeClass)]
#[inherit(Node)]
pub struct Records {
    records: Vec<Record>,
}

impl Records {
    pub fn get_autoload(node: Node) -> Option<Instance<Self>> {
        let records = unsafe {
            node.get_node("/root/Records".into())?
        };
        Instance::<Records>::try_from_base(records)
    }
}

impl Records {
    pub fn add_record(&mut self, record: Record) {
        self.records.push(record)
    }

    pub fn sorted_records(&self) -> Vec<(bool, Record)> {
        let is_first_iter = std::iter::once(true).chain(std::iter::repeat(false).take(self.records.len() - 1));
        let mut records_with_marker: Vec<_> = is_first_iter.zip(self.records.clone().into_iter()).collect();
        records_with_marker.as_mut_slice().sort_by_key(|o| o.1.wave_num);
        records_with_marker
    }
}

#[gdnative::methods]
impl Records {
    fn _init(_owner: Node) -> Self {
        Default::default()
    }
}
