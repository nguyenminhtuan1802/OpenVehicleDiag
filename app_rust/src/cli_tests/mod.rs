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

    pub const test_img: &[u8] = include_bytes!("../../img/cat.png");

    struct Line {
        start_x: u8,
        start_y: u8,
        end_x: u8,
        end_y: u8,
    }

    // #[test]
    // fn test_cmd() {
    //     const LCD_WIDTH: u32 = 60;
    //     const LCD_HEIGHT: u32 = 100;

    //     let dev = PassthruDevice::find_all().expect("Couldn't find any passthru adapters for test")
    //         [0]
    //     .clone();
    //     let drv = PassthruDrv::load_lib(dev.drv_path.clone()).expect("Couldn't load library");

    //     // Open a Comm server link with my custom Passthru adapter
    //     let mut api = PassthruApi::new(dev, drv).clone_box();
    //     api.open_device().expect("Could not open device!");

    //     // Start ISO-TP KWP2000 session with IC
    //     let server = KWP2000ECU::start_diag_session(
    //         api,
    //         &ISO15765Config {
    //             baud: 500_000,
    //             send_id: 1460,
    //             recv_id: 1268,
    //             block_size: 8,
    //             sep_time: 20,
    //             use_ext_isotp: false,
    //             use_ext_can: false,
    //         },
    //         None,
    //     )
    //     .expect("Error opening connection with IC ECU");

    //     // W203 IC is 56 pixels wide, ~100 tall for the top zone
    //     let img = image::load_from_memory_with_format(test_img, ImageFormat::Png)
    //         .expect("Error loading image");

    //     // get scale bounds for the image
    //     let sf = (img.width() as f32 / LCD_WIDTH as f32) as f32;

    //     let mut lines: Vec<Line> = Vec::new();
    //     // Drawing a large vertical line seems to clear the LCD in test mode
    //     lines.push(Line {
    //         start_x: 0,
    //         start_y: 0,
    //         end_x: 0,
    //         end_y: LCD_HEIGHT as u8,
    //     });

    //     for x in 0..LCD_WIDTH {
    //         let mut new_line = true;
    //         for y in 0..LCD_HEIGHT {
    //             let x_coord = min((x as f32 * sf) as u32, img.width() - 1);
    //             let y_coord = min((y as f32 * sf) as u32, img.height() - 1);
    //             let px_colour = img.get_pixel(x_coord, y_coord);
    //             let rgb = px_colour.0;
    //             if rgb[0] < 128 || rgb[1] < 128 || rgb[2] < 128 {
    //                 if new_line {
    //                     // Create new line
    //                     lines.push(Line {
    //                         start_x: x as u8,
    //                         start_y: y as u8,
    //                         end_x: x as u8,
    //                         end_y: y as u8,
    //                     });
    //                     new_line = false;
    //                 } else {
    //                     // Append to last line in the matrix
    //                     if let Some(line) = lines.last_mut() {
    //                         line.end_x = x as u8;
    //                         line.end_y = y as u8;
    //                     }
    //                 }
    //             } else {
    //                 new_line = true; // We need to draw a new line
    //             }
    //         }
    //     }

    //     for l in lines {
    //         // Send draw line command to LCD
    //         server.run_command(0x31, &[0x03, 0x06, l.start_x, l.start_y, l.end_x, l.end_y]);
    //     }

    //     loop {
    //         server.run_command(0x31, &[03, 06, 00, 00, 00, 00]); // Keep the test active (Stops LCD from clearing after test)
    //     }
    // }

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
