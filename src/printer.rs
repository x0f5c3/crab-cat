use std::sync::mpsc;
use std::time::Duration;
use crate::command::Command;
use crate::{Error, Result};
use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, CentralEvent, Manager as _, Characteristic, Peripheral, WriteType, ScanFilter};
// #[cfg(target_os = "linux")]
// use btleplug::bluez::{manager::Manager, peripheral::Peripheral as NativePeripheral};
// #[cfg(target_os = "windows")]
use btleplug::platform::{Manager, Peripheral as NativePeripheral};
use color_eyre::eyre::ContextCompat;
use futures::StreamExt;
use uuid::Uuid;

pub const COMMAND_SERVICE_UUID: Uuid = uuid_from_u16(0xaf30);
pub const COMMAND_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xae01);

pub async fn find_printer() -> Result<Printer<NativePeripheral>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().wrap_err("No adapters")?;

    let mut central_recv = central.events().await?;
    central.start_scan(ScanFilter::default()).await?;

    let (addr_send, addr_recv) = mpsc::channel();
    tokio::spawn(async move  {
        loop {
            match central_recv.next().await {
                Some(CentralEvent::ServicesAdvertisement {
                       id, services, ..
                   }) => {
                    if services.contains(&COMMAND_SERVICE_UUID) {
                        addr_send
                            .send(id)
                            .expect("could not send address to main thread");
                    }
                }
                None =>{},
                _ => {},
            }
        }
    });

    let address = addr_recv
        .recv_timeout(Duration::from_secs(10))
        .map_err(|e| Error::PrinterNotFound(e.to_string()))?;

    central.stop_scan().await?;

    let device = central
        .peripherals()
        .await?
        .into_iter()
        .find(|p| p.id() == address)
        .ok_or(Error::PrinterNotFound(format!("No printer with {address}")))?;

    Printer::new(device).await
}

pub struct Printer<D: Peripheral> {
    device: D,
    command_characteristic: Characteristic,
}

impl<D: Peripheral> Printer<D> {
    pub async fn new(device: D) -> Result<Self> {
        device.connect().await?;

        let characteristics = device.characteristics();
        let command_characteristic = characteristics
            .iter()
            .find(|c| c.uuid == COMMAND_CHARACTERISTIC_UUID)
            .ok_or(Error::PrinterNotFound(format!("Printer not found by {COMMAND_CHARACTERISTIC_UUID}")))?;
        let command_characteristic = command_characteristic.clone();

        Ok(Printer {
            device,
            command_characteristic,
        })
    }

    pub async fn send(&self, command: &Command) -> Result<()> {
        self.send_bytes(&command.as_bytes()).await
    }

    pub async fn send_all(&self, command: &[Command]) -> Result<()> {
        let buf = command
            .iter()
            .map(Command::as_bytes)
            .flatten()
            .collect::<Vec<_>>();
        self.send_bytes(&buf).await
    }

    async fn send_bytes(&self, bytes: &[u8]) -> Result<()> {
        // TODO: this is the MTU that's negotiated for my device - is this true
        // for all of them?
        const MTU: usize = 248;

        // 4 bytes required for L2CAP header
        for chunk in bytes.chunks(MTU - 4) {
            self.device.write(
                &self.command_characteristic,
                chunk,
                WriteType::WithoutResponse,
            ).await?;
        }

        Ok(())
    }
}