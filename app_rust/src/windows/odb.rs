use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Element, Column, Text, Align, Container, Length, Subscription, Row, Checkbox, Rule, Color, Space, button, Button};
use iced::time;
use std::sync::Arc;
use std::time::Instant;
use iced::widget::checkbox::Style;
use crate::windows::window::WindowMessage;
use iced::widget::button::State;
use crate::commapi::protocols::odb2::{Service09, Service03, Service01};
use crate::commapi::protocols::vin::Vin;

#[derive(Debug, Clone)]
pub enum ODBMessage {
    InitODB
}


#[derive(Debug, Clone)]
pub struct ODBHome {
    server: Box<dyn ComServer>,
    kline_state: button::State,
    can_state: button::State,
    vin: Option<Vin>,
    s1: Option<Service01>,
    s9: Option<Service09>
}

impl ODBHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        let mut ret = Self {
            server,
            kline_state: Default::default(),
            can_state: Default::default(),
            vin: None,
            s1: None,
            s9: None,
        };
        ret
    }

    pub fn update(&mut self, msg: &ODBMessage) -> Option<ODBMessage> {
        match msg {
            ODBMessage::InitODB => {
                if let Ok(s1) = Service01::init(&mut self.server, true) {
                    self.s1 = Some(s1)
                }
                if let Ok(s9) = Service09::init(&mut self.server, true) {
                    if let Ok(vin) = s9.get_vin(&mut self.server, true) {
                        self.vin = Some(vin);
                    }
                    self.s9 = Some(s9)
                }
            }
        }
        None
    }

    pub fn view(&mut self) -> Element<ODBMessage> {
        let odb_btn = Button::new(&mut self.kline_state, Text::new("K-Line not implemented")); // TODO Add K-LINE ODB
        let can_btn = match self.server.get_capabilities().supports_iso15765() {
            Capability::Yes => Button::new(&mut self.can_state, Text::new("ODB over CANBUS")).on_press(ODBMessage::InitODB),
            _ => Button::new(&mut self.can_state, Text::new("No CANBUS Support on adapter"))
        };


        let mut c = Column::new()
            .push(Text::new("ODB Diagnostics"))
            .push(Space::with_height(Length::Units(10)))
            .push(Row::new()
                .push(odb_btn)
                .push(can_btn))
            .align_items(Align::Center);

        if let Some(vin) = &self.vin {
            c = c.push(Text::new(format!("VIN: {}", vin.raw)));
            c = c.push(Text::new(format!("Year: {}", vin.year)));
            c = c.push(Text::new(format!("Manufacture: {}", vin.manufacture_name)));
            c = c.push(Text::new(format!("Location: {}", vin.manufacture_location)));
        }
        c = c.push(Space::with_height(Length::Units(10)));

        c = c.push(Text::new("Supported Services:"));

        let s01 = if self.s1.is_some() { "Yes" } else { "No" };
        let s09 = if self.s1.is_some() { "Yes" } else { "No" };

        c = c.push(Text::new(format!("Service 01: {}", s01)));
        c = c.push(Text::new(format!("Service 09: {}", s09)));

        if let Some(service_01) = self.s1 {
            let mut pid_row = Row::new();
            pid_row = pid_row.push(Text::new("Service 01 supported PIDS: "));
            for pid in service_01.get_supported_pids() {
                pid_row = pid_row.push(Text::new(format!("{:02X} ", pid)));
            }
            c = c.push(pid_row);
        }
        c.into()
    }
}