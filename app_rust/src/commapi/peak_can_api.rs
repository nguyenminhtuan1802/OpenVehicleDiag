
use std::{
    borrow::Borrow,
    sync::{Arc, PoisonError, RwLock},
    time::Instant,
};

use crate::commapi::comm_api::{
    CanFrame, ComServerError, DeviceCapabilities, FilterType, ISO15765Data,
};
use crate::{commapi, main};
use commapi::comm_api::ComServer;

use super::comm_api::Capability;

//#[derive(Debug)]
enum TPCANMessageType {
    /// The PCAN message is a CAN Standard Frame (11-bit identifier)
    PCAN_MESSAGE_STANDARD = 0x00,
    /// The PCAN message is a CAN Remote-Transfer-Request Frame
    PCAN_MESSAGE_RTR = 0x01,
    /// The PCAN message is a CAN Extended Frame (29-bit identifier)
    PCAN_MESSAGE_EXTENDED = 0x02,
    /// The PCAN message represents a FD frame in terms of CiA Specs
    PCAN_MESSAGE_FD = 0x04,
    /// The PCAN message represents a FD bit rate switch (CAN data at a higher bit rate)
    PCAN_MESSAGE_BRS = 0x08,
    /// The PCAN message represents a FD error state indicator(CAN FD transmitter was error active)
    PCAN_MESSAGE_ESI = 0x10,
    /// The PCAN message represents an echo CAN Frame
    PCAN_MESSAGE_ECHO = 0x20,
    /// The PCAN message represents an error frame
    PCAN_MESSAGE_ERRFRAME = 0x40,
    /// The PCAN message represents a PCAN status message
    PCAN_MESSAGE_STATUS = 0x80,
}

#[repr(C)]
//#[derive(Debug, Default)]
struct TPCANMsg {
    ID: u32,                 // 11/29-bit message identifier
    MSGTYPE: TPCANMessageType, // Type of the message
    LEN: u8,                 // Data Length Code of the message (0..8)
    DATA: [u8; 8],           // Data of the message (DATA[0]..DATA[7])
}

impl TPCANMsg {
    fn new(ID: u32, MSGTYPE: TPCANMessageType, LEN: u8, DATA: [u8; 8]) -> Self { TPCANMsg{ID, MSGTYPE, LEN, DATA}}
}

#[repr(C)]
#[derive(Debug, Default)]
struct TPCANTimestamp {
    millis: u32,            // Base-value: milliseconds: 0.. 2^32-1
    millis_overflow: u16,  // Roll-arounds of millis
    micros: u16,           // Microseconds: 0..999
}

impl TPCANTimestamp {
    fn new(millis: u32, millis_overflow: u16, micros: u16) -> Self { TPCANTimestamp{millis, millis_overflow, micros}}
}

#[cfg(target_os = "windows")]
#[link(name = "PCANBasic")]
extern {
    fn CAN_Initialize(Channel: u16, Btr0Btr1: u16, HwType: u8, IOPort: u32, Interrupt: u16) -> u32;
    fn CAN_Read(Channel: u16, MessageBuffer: *mut TPCANMsg, TimestampBuffer: *mut TPCANTimestamp) -> u32;
    fn CAN_Write(Channel: u16, MessageBuffer: *mut TPCANMsg) -> u32;
}

#[cfg(target_os = "linux")]
#[link(name = "pcanbasic")]
extern {
    fn CAN_Initialize(Channel: u16, Btr0Btr1: u16, HwType: u8, IOPort: u32, Interrupt: u16) -> u32;
    fn CAN_Read(Channel: u16, MessageBuffer: *mut TPCANMsg, TimestampBuffer: *mut TPCANTimestamp) -> u32;
    fn CAN_Write(Channel: u16, MessageBuffer: *mut TPCANMsg) -> u32;
}

#[derive(Clone, Copy)]
struct PeakCANSocket {
    handle: u16,
    baudRate: u16,
}

impl PeakCANSocket {
    fn new(handle: u16, baudRate: u16) -> Self { PeakCANSocket{handle, baudRate} }
}

#[derive(Clone)]
pub struct PeakCanAPI {
    iface: String,
    peakcan_iface: Arc<RwLock<Option<PeakCANSocket>>>,
    //can_filters: [Option<CANFilter>; 10],
}

impl std::fmt::Debug for PeakCanAPI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeakCanAPI")
            .field("iface", &self.iface)
            .finish()
    }
}

impl PeakCanAPI {
    pub fn new(iface: String) -> Self {
        PeakCanAPI { iface,
            peakcan_iface: Arc::new(RwLock::new(None))
        }
    }
}

#[allow(unused_variables)]
impl ComServer for PeakCanAPI 
{
    fn open_device(&mut self) -> Result<(), ComServerError> {
        Ok(()) // Device isn't opened in pcan, jsut the interface
    }

    fn close_device(&mut self) -> Result<(), ComServerError> {
        Ok(())
    }

    fn send_can_packets(
        &mut self,
        data: &[CanFrame],
        timeout_ms: u32,        
    ) -> Result<usize, ComServerError> {
        if let Some(socket) = self.peakcan_iface.write().unwrap().as_ref() {
            for frame in data {
                let mut can_data = TPCANMsg::new(frame.id,
                    TPCANMessageType::PCAN_MESSAGE_STANDARD,
                    frame.dlc,
                    frame.data);
                let status = unsafe { CAN_Write(socket.handle, &mut can_data)};
                if (status != 0) {
                    return Err(ComServerError {
                        err_code: 3,
                        err_desc: "PeakCAN write error".into(),
                    });
                }
            }
                Ok(data.len())
        } else {
            Err(ComServerError {
                err_code: 2,
                err_desc: "PeakCAN interface not open".into(),
            })
        }
    }

    fn read_can_packets(
        &self,
        timeout_ms: u32,
        max_msgs: usize,
    ) -> Result<Vec<CanFrame>,  ComServerError> {
        let mut res: Vec<CanFrame> = Vec::with_capacity(max_msgs);
        if let Some(socket) = self.peakcan_iface.read().unwrap().as_ref() {
            let start = Instant::now();
            while start.elapsed().as_millis() <= timeout_ms as u128 {
                let mut can_data = TPCANMsg::new(0,
                    TPCANMessageType::PCAN_MESSAGE_STANDARD,
                    0,
                    [0,0,0,0,0,0,0,0]);
                let mut time_stamp = TPCANTimestamp::new(0,0,0);
                let status = unsafe { CAN_Read(socket.handle, &mut can_data, &mut time_stamp)};
                if (status != 0) {
                    return Err(ComServerError {
                        err_code: status,
                        err_desc: "PeakCAN read error".into(),
                    });
                } else {
                    let canFrame = CanFrame::newWithData(can_data.ID, can_data.LEN, can_data.DATA);
                    res.push(canFrame);
                    if res.len() == max_msgs {
                        return Ok(res);
                    }
                }
            }
            return Ok(res);
        } else {
            Err(ComServerError {
                err_code: 2,
                err_desc: "PeakCAN interface not open".into(),
            })
        }
    }

    fn send_iso15765_data(
        &mut self,
        data: &[ISO15765Data],
        _timeout_ms: u32,
    ) -> Result<usize, ComServerError> {
        // Err(ComServerError {
        //     err_code: 1,
        //     err_desc: "Peak CAN Interface not supported!".into(),
        // })

        let canFrame: Vec<CanFrame> = data
            .iter()
            .map(|t| CanFrame {
                id: t.id,
                data: {
                    let mut array = [0u8; 8];
                    let len = std::cmp::min(t.data.len(), array.len());
                    array[..len].copy_from_slice(&t.data[..len]);
                    array
                },
                dlc: std::cmp::min(t.data.len(), 8) as u8,
            })
            .collect();
        return self.send_can_packets(&canFrame, _timeout_ms);
    }

    fn read_iso15765_packets(
        &self,
        timeout_ms: u32,
        max_msgs: usize,
    ) -> Result<Vec<ISO15765Data>, ComServerError> {
        // Err(ComServerError {
        //     err_code: 1,
        //     err_desc: "Peak CAN Interface not supported!".into(),
        // })
        let mut msg =  self.read_can_packets(timeout_ms, max_msgs);
        msg.map(|can_frames| {
            can_frames
                .into_iter()
                .map(|can_frame| ISO15765Data {
                    id: can_frame.id,
                    data: can_frame.data.to_vec(), // Convert the array to a Vec<u8>
                    pad_frame: false, // Adjust as needed
                    ext_addressing: false, // Adjust as needed
                })
                .collect()
        })
    }

    fn open_can_interface(
        &mut self,
        bus_speed: u32,
        is_ext_can: bool,
    ) -> Result<(), ComServerError> {
        if self.peakcan_iface.read().unwrap().is_some() {
            self.close_can_interface()?;
        }

        // baud rate should be 0x001C
        let pcan_socket = PeakCANSocket::new(0x51, bus_speed as u16);
        *self.peakcan_iface.write().unwrap() = Some(pcan_socket);
        self.iface = String::from("Peak-CAN");

        let status = unsafe { CAN_Initialize(pcan_socket.handle, bus_speed as u16, 0, 0, 0)};
        if (status != 0) {
            return Err(ComServerError {
                err_code: status,
                err_desc: "PeakCAN read error".into(),
            });
        }

        println!("PCAN Init success");
        Ok(())
    }


    fn close_can_interface(&mut self) -> Result<(), ComServerError> {
        if self.peakcan_iface.read().unwrap().is_none() {
            return Ok(()); // No socket to close
        }
        self.iface = String::from("");
        drop(self.peakcan_iface.write().unwrap()); // Dropping the socketCAN Iface closes it
        Ok(())
    }

    fn open_iso15765_interface(
        &mut self,
        bus_speed: u32,
        is_ext_can: bool,
        ext_addressing: bool,
    ) -> Result<(), ComServerError> {
        // Err(ComServerError {
        //     err_code: 1,
        //     err_desc: "Peak CAN Interface not supported!".into(),
        // })

        return self.open_can_interface(bus_speed, is_ext_can);
    }

    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError> {
        // Err(ComServerError {
        //     err_code: 1,
        //     err_desc: "Peak CAN Interface not supported!".into(),
        // })

        return self.close_can_interface();
    }

    fn add_can_filter(&mut self, f: FilterType) -> Result<u32, ComServerError> {
        Ok((1))
    }

    fn rem_can_filter(&mut self, filter_idx: u32) -> Result<(), ComServerError> {
        Ok(())
    }

    fn add_iso15765_filter(&mut self, f: FilterType) -> Result<u32, ComServerError> {
        Ok((1))
    }

    fn rem_iso15765_filter(&mut self, filter_idx: u32) -> Result<(), ComServerError> {
        Ok(())
    }

    fn set_iso15765_params(
        &mut self,
        separation_time_min: u32,
        block_size: u32,
    ) -> Result<(), ComServerError> {
        //unimplemented!()
        Ok(()) // SocketCAN will not do this - It can auto negotiate with the ECU
    }

    fn clear_can_rx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn clear_can_tx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn clear_iso15765_rx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn clear_iso15765_tx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn read_battery_voltage(&self) -> Result<f32, ComServerError> {
        // Socket CAN cannot measure battery voltage, so return -1.0 so user knows its not supported
        // rather than spitting out an error.
        Ok(-1.0)
    }

    fn clone_box(&self) -> Box<dyn ComServer> {
        Box::new(self.clone())
    }

    fn get_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            name: self.iface.clone(),
            vendor: "Unknown".into(),
            library_path: "N/A".into(),
            device_fw_version: "N/A".into(),
            library_version: "N/A".into(),
            j1850vpw: Capability::NA,
            j1850pwm: Capability::NA,
            can: Capability::Yes,
            iso15765: Capability::Yes,
            iso9141: Capability::NA,
            iso14230: Capability::NA,
            ip: Capability::NA,
            battery_voltage: Capability::NA,
        }
    }

    fn get_api(&self) -> &str {
        "Peak CAN"
    }

    fn is_connected(&self) -> bool {
        if self.peakcan_iface.read().unwrap().is_some() {
            true
        } else {
            false
        }
    }

}