use std::time::Duration;

use ephemeropt::EphemeralOption;
use proto::types::{BackendMessageType, FrontendMessageType};

const CACHE_DURATION: Duration = Duration::from_millis(1500);

macro_rules! cache {
    ($name:ident, included = [$($key:ident: $discrim:ident),*], excluded = [$($excl:ident),*]) => {
        pub struct $name {
            $( $key: EphemeralOption<BackendMessageType> ),*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $( $key: EphemeralOption::new_empty(CACHE_DURATION), )*
                }
            }

            pub fn get(&self, key: &FrontendMessageType) -> Option<BackendMessageType> {
                match key {
                    $( FrontendMessageType::$discrim => self.$key.get().cloned(), )*
                }
            }

            pub fn insert(&mut self, val: BackendMessageType) {
                match val {
                    $( BackendMessageType::$discrim(_) => { self.$key.insert(val); }, )*
                    $( BackendMessageType::$excl(_) => {} ),*
                };
            }
        }
    };
}

cache!(BackendCache,
    included = [cpu: Cpu, temp: Temp, mem: Mem, disk: Disk, net_io: NetIO],
    excluded = [Handshake]
);
