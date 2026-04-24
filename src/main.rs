use bitflags::bitflags;
use futures_util::StreamExt;
use inputtino::{DeviceDefinition, JoypadButton, JoypadStickPosition, XboxOneJoypad};
use local_ip_address::local_ip;
use qrcode::{QrCode, render::unicode};
use serde::{Deserialize, Serialize};
use std::{i16, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex, time::sleep,
};
use tokio_tungstenite::accept_async;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
enum ClientButton {
    D_LEFT,
    D_RIGHT,
    D_UP,
    D_DOWN,
    A,
    B,
    X,
    Y,
    LB,
    LT,
    RB,
    RT,
    RS,
    LS,
    START,
    BACK,
}

impl ClientButton {
    pub fn to_joypad(self) -> JoypadButton {
        match self {
            ClientButton::D_LEFT => JoypadButton::DPAD_LEFT,
            ClientButton::D_RIGHT => JoypadButton::DPAD_RIGHT,
            ClientButton::D_UP => JoypadButton::DPAD_UP,
            ClientButton::D_DOWN => JoypadButton::DPAD_DOWN,
            ClientButton::A => JoypadButton::A,
            ClientButton::B => JoypadButton::B,
            ClientButton::X => JoypadButton::X,
            ClientButton::Y => JoypadButton::Y,
            ClientButton::LB => JoypadButton::LEFT_BUTTON,
            ClientButton::RB => JoypadButton::RIGHT_BUTTON,
            ClientButton::LT => JoypadButton::PADDLE2_FLAG,
            ClientButton::RT => JoypadButton::PADDLE3_FLAG,
            ClientButton::START => JoypadButton::START,
            ClientButton::BACK => JoypadButton::BACK,
            //this two does no run in any condition
            ClientButton::RS => JoypadButton::BACK,
            ClientButton::LS => JoypadButton::BACK,
        }
    }

    pub fn to_raw(self) -> i32 {
        self.to_joypad() as i32
    }
}
#[derive(Deserialize, Serialize, Debug)]
struct ClientData {
    name: ClientButton,
    is_pressed: Option<bool>,
    x: Option<i16>,
    y: Option<i16>,
}

bitflags! {
    struct ButtonState: i32 {
        const D_UP     = JoypadButton::DPAD_UP as i32;
        const D_DOWN   = JoypadButton::DPAD_DOWN as i32;
        const D_LEFT   = JoypadButton::DPAD_LEFT as i32;
        const D_RIGHT  = JoypadButton::DPAD_RIGHT as i32;
        const START       = JoypadButton::START as i32;
        const A           = JoypadButton::A as i32;
        const B           = JoypadButton::B as i32;
        const X           = JoypadButton::X as i32;
        const Y           = JoypadButton::Y as i32;
        const LB          = JoypadButton::LEFT_BUTTON as i32;
        const RB          = JoypadButton::RIGHT_BUTTON as i32;
        const BACK        = JoypadButton::BACK as i32;
    }
}

struct JoystickState {
    x: i16,
    y: i16,
}

impl JoystickState {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

struct ButtonStateHandle {
    controller: XboxOneJoypad,
    button_state: ButtonState,
    triggers_state: (i16, i16),
    joysticks_state: (JoystickState, JoystickState),
}

impl ButtonStateHandle {
    fn new(device: XboxOneJoypad) -> Self {
        Self {
            controller: device,
            button_state: ButtonState::empty(),
            triggers_state: (0, 0),
            joysticks_state: (JoystickState::new(), JoystickState::new()),
        }
    }

    async fn button_press(&mut self, data: ClientData) {
        let flag = ButtonState::from_bits_truncate(data.name.to_raw());
        if data.is_pressed.unwrap() {
            self.button_state.insert(flag);
            self.controller.set_pressed(self.button_state.bits());
        } else {
            sleep(Duration::from_millis(50)).await;
            self.button_state.remove(flag);
            self.controller.set_pressed(self.button_state.bits());
        }
    }

    async fn triggers_press(&mut self, data: ClientData) {
        if data.is_pressed.unwrap() {
            if data.name == ClientButton::RT {
                self.triggers_state.1 = i16::max_value();
                self.controller
                    .set_triggers(self.triggers_state.0, self.triggers_state.1);
            } else {
                self.triggers_state.0 = i16::max_value();
                self.controller
                    .set_triggers(self.triggers_state.0, self.triggers_state.1);
            }
        } else {
            if data.name == ClientButton::RT {
                self.triggers_state.1 = 0;
                self.controller
                    .set_triggers(self.triggers_state.0, self.triggers_state.1);
            } else {
                self.triggers_state.0 = 0;
                self.controller
                    .set_triggers(self.triggers_state.0, self.triggers_state.1);
            }
        }
    }

    async fn joysticks_drag(&mut self, data: ClientData) {
        if data.name == ClientButton::RS {
            self.joysticks_state.1.x = data.x.unwrap() * 650;
            self.joysticks_state.1.y = data.y.unwrap() * 650;
            self.controller.set_stick(
                JoypadStickPosition::RS,
                self.joysticks_state.1.x,
                self.joysticks_state.1.y,
            );
        } else {
            self.joysticks_state.0.x = data.x.unwrap() * 650;
            self.joysticks_state.0.y = data.y.unwrap() * 650;
            self.controller.set_stick(
                JoypadStickPosition::LS,
                self.joysticks_state.0.x,
                self.joysticks_state.0.y,
            );
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let device = DeviceDefinition::new(
        "Inputtino XBox One controller",
        0x045E,
        0x02DD,
        0x0100,
        "00:11:22:33:44",
        "00:11:22:33:44",
    );
    let xbox = XboxOneJoypad::new(&device).unwrap();
    // xbox.set_stick(JoypadStickPosition::RS, 0, -i16::max_value());
    let controller = Arc::new(Mutex::new(ButtonStateHandle::new(xbox)));
    let listener = TcpListener::bind("0.0.0.0:7879").await.unwrap();
    let local_ip_addr = local_ip().unwrap();
    let code = QrCode::new(format!("{}:{}", local_ip_addr, 7879)).unwrap();
    let qr = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    println!("{}", qr);
    println!(
        "Websocket server is started in this address {} with port 7879",
        local_ip_addr
    );

    while let Ok((socket, _)) = listener.accept().await {
        println!("Websocket is connected");
        let ctrl = Arc::clone(&controller);
        tokio::spawn(handle_ws(socket, ctrl));
    }
}

async fn handle_ws(stream: TcpStream, controller: Arc<Mutex<ButtonStateHandle>>) {
    let ws = accept_async(stream).await.unwrap();
    let (_tx, mut rx) = ws.split();

    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_text() {
            // println!("data from ws {:?}", &msg.clone().into_text());
            match serde_json::from_str::<ClientData>(&msg.into_text().unwrap()) {
                Ok(data) => {
                    let mut ctrl = controller.lock().await;
                    if data.name == ClientButton::LT || data.name == ClientButton::RT {
                        ctrl.triggers_press(data).await;
                        continue;
                    }
                    if data.name == ClientButton::LS || data.name == ClientButton::RS {
                        // println!("detect the json");
                        ctrl.joysticks_drag(data).await;
                        continue;
                    }
                    // println!("button is press {:?}", data);
                    ctrl.button_press(data).await;
                }
                Err(err) => eprintln!("JSON parse error: {:?}", err),
            }
        }
    }
}
