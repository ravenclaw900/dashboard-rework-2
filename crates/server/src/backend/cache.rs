use std::time::Duration;

use ephemeropt::EphemeralOption;
use proto::types::{DataRequestType, DataResponseType};

const CACHE_DURATION: Duration = Duration::from_millis(1500);

macro_rules! cache {
    ($name:ident, [$($key:ident: $discrim:ident),*]) => {
        pub struct $name {
            $( $key: EphemeralOption<DataResponseType> ),*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $( $key: EphemeralOption::new_empty(CACHE_DURATION), )*
                }
            }

            pub fn get(&self, key: &DataRequestType) -> Option<DataResponseType> {
                match key {
                    $( DataRequestType::$discrim => self.$key.get().cloned(), )*
                }
            }

            pub fn insert(&mut self, val: DataResponseType) {
                match val {
                    $( DataResponseType::$discrim(_) => self.$key.insert(val), )*
                };
            }
        }
    };
}

cache!(BackendCache, [cpu: Cpu, temp: Temp, mem: Mem, disk: Disk, net_io: NetIO]);
