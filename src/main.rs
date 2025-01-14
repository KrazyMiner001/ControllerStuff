
use std::{mem, ops::{Add, AddAssign}, ptr};

use color_space::{Hsl, Hsv, ToRgb};
use sdl2::{controller::Axis, event::{self, Event, EventType}, keyboard::Keycode, pixels::Color, sys::{SDL_CommonEvent, SDL_ControllerSensorEvent, SDL_Event, SDL_GameControllerGetJoystick, SDL_GameControllerOpen, SDL_GameControllerSetSensorEnabled, SDL_PollEvent, SDL_PumpEvents, SDL_QuitEvent, SDL_SensorEvent, SDL_SensorType, SDL_WindowEventID}};

fn main() {
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

    let sdl = sdl2::init().expect("Sdl failed to initialise");

    let mut event_pump = sdl.event_pump().unwrap();
    let window = sdl.video().unwrap().window("Test", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .build()
        .unwrap();

    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();

    let game_controller_subsystem = sdl.game_controller().unwrap();

    let available = game_controller_subsystem
        .num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))
        .unwrap();

    let mut controller = (0..available)
        .find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                println!("{} is not a game controller", id);
                return None;
            }

            Some(game_controller_subsystem.open(id).unwrap())
        })
        .unwrap();

    let raw_controller = unsafe { SDL_GameControllerOpen(0) };

    unsafe { 
        SDL_GameControllerSetSensorEnabled(
            raw_controller, 
            SDL_SensorType::SDL_SENSOR_GYRO, 
            sdl2::sys::SDL_bool::SDL_TRUE
        );
    };

    let mut tracked_gyro_info = TrackedGyroInfo::new(0.0, 0.0, 0.0);

    'running: loop {
        unsafe {
            SDL_PumpEvents();

            while SDL_PollEvent(core::ptr::null_mut()) == 1 {
                let mut raw = mem::MaybeUninit::uninit();

                SDL_PollEvent(raw.as_mut_ptr());

                let event = raw.assume_init();

                match event.csensor {
                    SDL_ControllerSensorEvent {data: [pitch, yaw, roll], type_: 1625, sensor: 2, ..} => {
                        tracked_gyro_info += TrackedGyroInfo::from_f32(
                            yaw / 10.0, 
                            pitch / 300.0, 
                            roll / 100.0
                        );

                        let hsv_color = Hsl::new(
                            tracked_gyro_info.yaw.rem_euclid(360.0),
                            tracked_gyro_info.roll.cos(),
                            tracked_gyro_info.pitch.cos()
                        );

                        let color = Color::RGB(
                            hsv_color.to_rgb().r as u8, 
                            hsv_color.to_rgb().g as u8,
                            hsv_color.to_rgb().b as u8
                        );

                        canvas.set_draw_color(color);
                        canvas.clear();
                    }
                    _ => {}
                }
                
                match Event::from_ll(event) {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
        }

        canvas.present();
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
