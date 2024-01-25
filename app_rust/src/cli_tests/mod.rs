#[cfg(test)]
pub mod draw_routine {
    use std::cmp::min;

    use image::{GenericImageView, ImageFormat};

    use crate::{
        commapi::{comm_api::ComServer, passthru_api::PassthruApi, protocols::kwp2000::KWP2000ECU, protocols::uds::UDSECU},
        commapi::{comm_api::ISO15765Config, protocols::ProtocolServer, protocols::DiagCfg},
        commapi::{peak_can_api::{self, PeakCanAPI}, comm_api::CanFrame},
        commapi::iface,
        passthru::{PassthruDevice, PassthruDrv},
        themes::images::{TRAY_ICON, TRAY_ICON_DARK},
    };

    #[test]
    fn test_pcan() {
        let mut dev = PeakCanAPI::new(String::from("PeakCan"));

        if let Err(e) = dev.open_device() {
            panic!("Failed to open device: {:?}", e);
        } else {
            assert!(true);
        }

        if let Err(e) = dev.open_can_interface(0x001C, false) {
            panic!("Failed to open interface: {:?}", e);
        } else {
            assert!(true);
        }

        let can_frame = CanFrame::newWithData(0x784, 8,
            [0x02, 0x11, 0x01, 0x55, 0x55, 0x55, 0x55, 0x55]);
        let can_frames = vec![can_frame];

        if let Err(e) = dev.send_can_packets(&can_frames, 0) {
            panic!("Failed to send can data: {:?}", e);
        } else {
            assert!(true);
        }
    }

    #[test]
    fn test_uds_pcan_server() {
        let mut dev = PeakCanAPI::new(String::from("PeakCan"));

        if let Err(e) = dev.open_device() {
            panic!("Failed to open device: {:?}", e);
        } else {
            assert!(true);
        }

        // Start ISO-TP KWP2000 session with IC
        UDSECU::start_diag_session_test(
            &dev.clone_box(),
            iface::InterfaceType::IsoTp,
            iface::InterfaceConfig::from_iso15765(ISO15765Config {
                baud: 0x001C,
                send_id: 0x784,
                recv_id: 0x7F0,
                block_size: 8,
                sep_time: 20,
                use_ext_isotp: false,
                use_ext_can: false,
            }),
            None,
            DiagCfg { // Not used
                send_id: 0,
                recv_id: 0,
                global_id: None,
            }
        );
        //.expect("Error opening connection with IC ECU");  

        
    }
}
