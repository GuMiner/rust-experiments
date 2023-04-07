pub struct Config {
    pub num_width: i32,
    pub num_height: i32,
    pub num_days: i32,
    pub num_colors: i32,

    last_width: i32,
    last_height: i32,
    last_days: i32,
}

const PIXELS_PER_DAY_AVG: f64 = 80.0;

impl Config {
    pub fn recalculate_columns(&mut self) {
        if self.last_width != self.num_width || self.last_height != self.num_height {
            // Recalculate based on image size
            self.num_days = (self.num_width * self.num_height) / (PIXELS_PER_DAY_AVG as i32);
            self.sync_columns();
        }

        if self.last_days != self.num_days {
            // Figure out aspect ratio and scale up/down from there.
            let aspect_ratio = (self.num_height as f64) / (self.num_width as f64);

            // width * height = days * PIXELS_PER_DAY_AVG
            // height / width = aspect_ratio
            // width^2 * aspect_ratio = days * PIXELS_PER_DAY_AVG
            let width = f64::sqrt((self.num_days as f64) * PIXELS_PER_DAY_AVG / aspect_ratio);
            let height = aspect_ratio * width;

            self.num_width = width as i32;
            self.num_height = height as i32;
            self.sync_columns();
        }
    }

    fn sync_columns(&mut self) {
        self.last_width = self.num_width;
        self.last_height = self.num_height;
        self.last_days = self.num_days;
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut default_config = Self {
            num_width: 40,
            num_height: 30,
            num_days: 15,
            num_colors: 24,
            
            last_width: -1,
            last_height: -1,
            last_days: -1,
        };

        default_config.sync_columns();

        default_config
    }
}
