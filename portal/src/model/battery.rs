use metapower_framework::model::{Battery, BatteryRole};

pub trait BatteryPortal {
    fn get_battery_roles() -> Vec<BatteryRole>;    
}

impl BatteryPortal for Battery {
    fn get_battery_roles() -> Vec<BatteryRole> {
        vec![BatteryRole{name: "admin".to_string()}]
    }
}
