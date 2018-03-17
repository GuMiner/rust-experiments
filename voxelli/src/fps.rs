// Defines a simple module for FPS tracking.
extern crate time;

struct Period {
    frames: u32,
    elapsed_time: f64
}

pub struct Fps {
    pub fps: f64,
    last_frame_start: f64,
    current_period: Period
}

impl Period {
    fn new() -> Period {
        Period {
            frames: 0,
            elapsed_time: 0.0
        }
    }

    fn fps(&self) -> f64 {
        (self.frames as f64) / self.elapsed_time
    }

    fn reset(&mut self) {
        self.frames = 0;
        self.elapsed_time = 0.0;
    }
}

impl Fps {
    pub fn new() -> Fps {
        Fps {
            fps: 30.0,
            last_frame_start: time::precise_time_s(),
            current_period: Period::new()
        }
    }

    pub fn update(&mut self) {
        let now = time::precise_time_s();
        
        self.current_period.frames += 1;
        self.current_period.elapsed_time += now - self.last_frame_start;
    
        self.last_frame_start = now;

        if self.current_period.elapsed_time > 1.0 {
            self.fps = self.current_period.fps();
            self.current_period.reset();
        }
    }
}