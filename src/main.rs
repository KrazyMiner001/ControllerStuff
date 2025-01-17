
use std::{f64::consts::PI, mem, ops::{Add, AddAssign}, time::Duration};

use color_space::{Hsl, ToRgb};
use sdl3::{event::Event, gamepad::Button, keyboard::Keycode, pixels::Color, sys::{events::{SDL_Event, SDL_GamepadDeviceEvent, SDL_GamepadSensorEvent, SDL_PollEvent, SDL_PumpEvents}, gamepad::{SDL_GetGamepads, SDL_IsGamepad, SDL_OpenGamepad, SDL_SetGamepadSensorEnabled}, sensor::{SDL_Sensor, SDL_SensorID, SDL_SensorType, SDL_SENSOR_GYRO}}};

fn main() {
    //sdl3::hint::set("SDL_JOYSTICK_THREAD", "1");

    let sdl = sdl3::init().expect("Sdl failed to initialise");

    let window = sdl.video().unwrap().window("Test", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();

    let gamepad_subsystem =  sdl.gamepad().unwrap();
    let joystick_subsystem = sdl.joystick().unwrap();

    for joystick in joystick_subsystem.joysticks().unwrap() {
        if gamepad_subsystem.is_game_controller(joystick.id) {
            unsafe  {
                let raw_controller = SDL_OpenGamepad(joystick.id);

                SDL_SetGamepadSensorEnabled(
                    raw_controller,
                    SDL_SENSOR_GYRO,
                    true
                );
            }
        }
    };

    let mut tracked_gyro_info = TrackedGyroInfo::new(0.0, 0.0, 0.0);

    'running: loop {
        unsafe {
            SDL_PumpEvents();

            while SDL_PollEvent(core::ptr::null_mut()) {
                let mut raw = mem::MaybeUninit::uninit();

                SDL_PollEvent(raw.as_mut_ptr());

                let event = raw.assume_init();

                match event.gsensor {
                    SDL_GamepadSensorEvent {data: [pitch, yaw, roll], sensor: 2, ..} => {
                        tracked_gyro_info += TrackedGyroInfo::from_f32(
                            yaw / 10.0, 
                            pitch / 300.0, 
                            roll / 100.0
                        );
                    }
                    _ => {

                    }
                }

                match Event::from_ll(event) {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    Event::ControllerButtonDown { button: Button::South, .. } => {
                        tracked_gyro_info.clear();
                    },
                    _ => {

                    }
                }
            }
        }

        let hsv_color = Hsl::new(
            tracked_gyro_info.yaw.rem_euclid(360.0),
            tracked_gyro_info.roll.cos() / 2.0 + 0.5,
            ((tracked_gyro_info.pitch - PI / 2.0).cos() + 1.0) / 2.0
        );

        let color = Color::RGB(
            hsv_color.to_rgb().r as u8, 
            hsv_color.to_rgb().g as u8,
            hsv_color.to_rgb().b as u8
        );

        canvas.set_draw_color(color);
        canvas.clear();

        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[derive(Debug)]
struct TrackedGyroInfo {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64
}

impl TrackedGyroInfo {
    pub fn new(yaw: f64, pitch: f64, roll: f64) -> TrackedGyroInfo {
        TrackedGyroInfo {
            yaw,
            pitch,
            roll
        }
    }

    pub fn from_f32(yaw: f32, pitch: f32, roll: f32) -> TrackedGyroInfo {
        TrackedGyroInfo {
            yaw: yaw.into(),
            pitch: pitch.into(),
            roll: roll.into()
        }
    }

    pub fn clear(&mut self) {
        self.pitch = 0.0;
        self.roll = 0.0;
        self.yaw = 0.0;
    }
}

impl Add for TrackedGyroInfo {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            yaw: self.yaw + rhs.yaw,
            pitch: self.pitch + rhs.pitch,
            roll: self.roll + rhs.roll
        }
    }
}

impl AddAssign for TrackedGyroInfo {
    fn add_assign(&mut self, rhs: Self) {
        let other = Self {
            yaw: self.yaw + rhs.yaw,
            pitch: self.pitch + rhs.pitch,
            roll: self.roll + rhs.roll
        };

        self.yaw = other.yaw;
        self.pitch = other.pitch;
        self.roll = other.roll;
    }
}
