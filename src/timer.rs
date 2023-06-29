use std::time::Instant;

pub struct FpsCounter {
    time: f32,
    count: u32,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            time: 0.0,
            count: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.count += 1;
        self.time += dt;

        if self.time >= 1.0 {
            //println!("{}", self.count as f32 / self.time);
            self.count = 0;
            self.time = 0.0;
        }
    }
}

pub struct Timer {
    now: Instant,
    earlier: Instant,
    active: bool,
    speed: f32,
    elapsed: f32,
    dt: f32,
    dt_min_cap: Option<f32>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            now: Instant::now(),
            earlier: Instant::now(),
            active: true,
            speed: 1_f32,
            elapsed: 0_f32,
            dt: 0_f32,
            dt_min_cap: None,
        }
    }

    pub fn set_delta_time_min_cap(&mut self, cap: f32) {
        self.dt_min_cap = Some(cap);
    }

    pub fn get_delta_time(&self) -> f32 {
        self.dt
    }

    pub fn get_elapsed_time(&self) -> f32 {
        self.elapsed
    }

    pub fn toggle_pause(&mut self) {
        self.active = !self.active;
    }

    pub fn update(&mut self) -> f32 {
        let mut dt = 0.0;

        if self.active == true {
            self.earlier = self.now;
            self.now = Instant::now();

            let delta = self.now.duration_since(self.earlier);
            let secs = delta.as_secs() as f32;
            let nanos = delta.subsec_nanos() as f32 * 1e-9;

            self.dt = (secs + nanos) * self.speed;
            self.elapsed += self.dt;

            dt = match self.dt_min_cap {
                None => self.dt,
                Some(c) => self.dt.min(c),
            };
        }

        dt
    }
}
