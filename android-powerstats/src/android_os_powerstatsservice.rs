use crate::{
    bundle::Bundle,
    result_receiver::{iresultreceiver::IResultReceiver, ResultReceiver},
};

#[path = "android/os/IPowerStatsService.rs"]
pub mod powerstatsservice;

pub(crate) mod mangled {
    pub(crate) use super::powerstatsservice::mangled::*;
}

struct ReceiveSupportedPowerMonitors;

impl binder::Interface for ReceiveSupportedPowerMonitors {}
impl IResultReceiver for ReceiveSupportedPowerMonitors {
    fn r#send(&self, _arg_resultCode: i32, _arg_resultData: &Bundle) -> binder::Result<()> {
        dbg!(_arg_resultCode, _arg_resultData);
        Ok(())
    }
}

pub fn run() -> binder::Result<()> {
    let i = binder::get_interface::<dyn powerstatsservice::IPowerStatsService>("powerstats")?;
    dbg!(&i);

    let receiver = ResultReceiver::new(ReceiveSupportedPowerMonitors);
    i.getSupportedPowerMonitors(&receiver)?;
    dbg!(&receiver);

    Ok(())
}
