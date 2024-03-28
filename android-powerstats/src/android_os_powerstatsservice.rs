use std::{
    any::Any,
    sync::mpsc::{channel, Receiver, Sender},
};

use binder::{
    binder_impl::{BorrowedParcel, Deserialize},
    impl_deserialize_for_parcelable, Parcelable, StatusCode,
};

use crate::{
    bundle::{parcel_read_string8, Bundle, Object},
    result_receiver::{iresultreceiver::IResultReceiver, ResultReceiver},
};

#[path = "android/os/IPowerStatsService.rs"]
pub mod powerstatsservice;

pub(crate) mod mangled {
    pub(crate) use super::powerstatsservice::mangled::*;
}

/// Java-only parcelable
/// <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/PowerMonitor.java;l=40;drc=82bdcd7ff7ba4962274f1d88caac0594ae964bef>
#[derive(Clone, Debug, Default)]
pub(crate) struct PowerMonitor {
    index: i32,
    r#type: i32,
    name: String,
}

impl Parcelable for PowerMonitor {
    fn write_to_parcel(&self, parcel: &mut BorrowedParcel<'_>) -> Result<(), StatusCode> {
        todo!()
    }

    fn read_from_parcel(&mut self, parcel: &BorrowedParcel<'_>) -> Result<(), StatusCode> {
        todo!()
        // self.index = parcel.read()?;
        // self.r#type = parcel.read()?;
        // self.name = parcel.read()?;
        // Ok(())
    }
}

impl Deserialize for PowerMonitor {
    fn deserialize(parcel: &BorrowedParcel<'_>) -> Result<Self, StatusCode> {
        Ok(Self {
            index: parcel.read()?,
            r#type: parcel.read()?,
            name: parcel_read_string8(parcel)?,
        })
    }
}
// impl_deserialize_for_parcelable!(PowerMonitor);

struct ReceiveSupportedPowerMonitors(Sender<Vec<PowerMonitor>>);
impl ReceiveSupportedPowerMonitors {
    fn new() -> (Self, Receiver<Vec<PowerMonitor>>) {
        let (s, r) = channel();
        (Self(s), r)
    }
}
impl binder::Interface for ReceiveSupportedPowerMonitors {}
impl IResultReceiver for ReceiveSupportedPowerMonitors {
    fn r#send(&self, code: i32, data: &Bundle) -> binder::Result<()> {
        assert_eq!(code, 0);
        let Object::ParcelableArray(monitors) = &data.0[powerstatsservice::KEY_MONITORS] else {
            panic!("Must have ParcelableArray")
        };

        let result = monitors
            .into_iter()
            .map(|monitor| {
                // TODO: Need owned box or Any for downcast
                let monitor: &Box<PowerMonitor> = unsafe { std::mem::transmute(monitor) };
                // dbg!(monitor);
                *monitor.clone()
            })
            .collect::<Vec<_>>();

        self.0.send(result).unwrap();

        Ok(())
    }
}

#[derive(Debug)]
struct PowerReading {
    timestampMs: i64,
    energyUws: i64,
}

struct ReceivePowerMonitorReadings(Sender<Vec<PowerReading>>);
impl ReceivePowerMonitorReadings {
    fn new() -> (Self, Receiver<Vec<PowerReading>>) {
        let (s, r) = channel();
        (Self(s), r)
    }
}
impl binder::Interface for ReceivePowerMonitorReadings {}
impl IResultReceiver for ReceivePowerMonitorReadings {
    fn r#send(&self, code: i32, data: &Bundle) -> binder::Result<()> {
        assert_eq!(code, 0);
        let Object::LongArray(timestamps) = &data.0[powerstatsservice::KEY_TIMESTAMPS] else {
            panic!("Must have LongArray")
        };
        let Object::LongArray(energy) = &data.0[powerstatsservice::KEY_ENERGY] else {
            panic!("Must have LongArray")
        };

        let result = timestamps
            .iter()
            .zip(energy)
            .map(|(t, e)| PowerReading {
                timestampMs: *t,
                energyUws: *e,
            })
            .collect::<Vec<_>>();

        self.0.send(result).unwrap();

        Ok(())
    }
}

pub fn run() -> binder::Result<()> {
    let i = binder::get_interface::<dyn powerstatsservice::IPowerStatsService>("powerstats")?;
    dbg!(&i);

    let (receiver, chan) = ReceiveSupportedPowerMonitors::new();
    let receiver = ResultReceiver::new(receiver);
    // TODO: Since we pass a borrow, can we get access to the contents again?
    i.getSupportedPowerMonitors(&receiver)?;
    let monitors = chan.recv().unwrap();

    let monitor_ids = monitors
        .iter()
        .filter(|m| m.name.ends_with(":GPU"))
        .inspect(|m| println!("Querying power stats for {}", m.name))
        .map(|m| m.index)
        .collect::<Vec<_>>();

    let (receiver, chan) = ReceivePowerMonitorReadings::new();
    let receiver = ResultReceiver::new(receiver);
    for _ in 0..10 {
        i.getPowerMonitorReadings(&monitor_ids, &receiver)?;
        let readings = chan.recv().unwrap();
        dbg!(readings);
    }

    Ok(())
}
