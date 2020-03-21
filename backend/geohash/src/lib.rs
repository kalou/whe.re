#[macro_use]
extern crate log;
extern crate env_logger;

pub mod base32;

#[derive(Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64
}

impl Range {
    /// Create a new Range suitable for splits
    fn new(min: f64, max: f64) -> Self {
        Range { min, max }
    }

    /// Perform a "binary split" of range.
    fn split(&mut self, v: f64) -> u64 {
        let middle = (self.max + self.min) / 2.0;
        if v >= middle {
            self.min = middle;
            return 1;
        } else {
            self.max = middle;
            return 0;
        }
    }

    /// Perform a decoding split : split for 1 or 0
    fn split_bin(&mut self, v: u64) {
        let middle = (self.max + self.min) / 2.0;
        if v == 1 {
            self.min = middle;
        } else {
            self.max = middle;
        }
    }
}

#[derive(Debug)]
pub struct GeoBox {
    pub x_range: Range,
    pub y_range: Range,
}

impl GeoBox {
    fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        GeoBox {
            x_range: Range::new(min_x, max_x),
            y_range: Range::new(min_y, max_y),
        }
    }

    fn geohash(&mut self, lat: f64, lon: f64, precision: usize) -> u64 {
        let mut val = 0;

        for n in 0..precision*5 {
            if n % 2 == 0 {
                val = (val << 1) | self.x_range.split(lon);
            } else {
                val = (val << 1) | self.y_range.split(lat);
            }
        }

        return val;
    }

    fn dehash(&mut self, value: u64, precision: usize) {
        let mut prec = (precision * 5);
        for i in 0..prec {
            if i % 2 == 0 {
                self.x_range.split_bin((value >> prec-i-1) & 1);
            } else {
                self.y_range.split_bin((value >> prec-i-1) & 1);
            }
        }
    }
}

pub fn val(lat: f64, lon: f64, precision: usize) -> u64 {
    let mut geobox = GeoBox::new(-180.0, 180.0, -90.0, 90.0);
    geobox.geohash(lat, lon, precision)
}

/// Precision should be between 1 and 12
pub fn hash(lat: f64, lon: f64, precision: usize) -> String {
    base32::encode(val(lat, lon, precision), precision)
}

pub fn geobox_for_hash(h: &str) -> GeoBox {
    let val = base32::decode(h);
    let mut geobox = GeoBox::new(-180.0, 180.0, -90.0, 90.0);
    geobox.dehash(base32::decode(h), h.len());
    geobox
}

#[cfg(test)]
mod tests {
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    #[test]
    fn test_range_25() {
        assert_eq!(super::val(37.77564, -122.41365, 5),
                   0b01001_10110_01000_11110_11110)
    }
    #[test]
    fn test_decode() {
        assert_eq!(super::hash(37.77926, -122.41923, 11),
                   "9q8yym901hw");
    }

    #[test]
    fn test_decode_1() {
        assert_eq!(super::hash(51.50479, -0.07871, 11),
                    "gcpvn0ntjut");
    }

    #[test]
    fn test_decode_2() {
        assert_eq!(super::hash(51.47651, 0.00283, 11),
                   "u10hb5403uy");
    }

    #[test]
    fn test_decode_3() {
        assert_eq!(super::hash(34.05366, -118.24276, 11),
                   "9q5ctr60zyr");
    }


    #[test]
    fn test_decode_4() {
        assert_eq!(super::hash(37.86947, -122.27093, 11),
                   "9q9p3tvj8uf");
    }

    #[test]
    fn test_from_hash() {
        let gbox = super::geobox_for_hash("9q8yym901hw");
        assert_eq!(gbox.x_range.min, -122.41923004388809);
        assert_eq!(gbox.y_range.min, 37.779259979724884);
        assert!((gbox.x_range.max - gbox.x_range.min) < 0.0002);
        assert!((gbox.y_range.max - gbox.y_range.min) < 0.0002);
    }

    #[test]
    fn test_from_hash_big() {
        let gbox = super::geobox_for_hash("9q8y");
        assert_eq!((gbox.x_range.max - gbox.x_range.min), 0.3515625);
        assert_eq!((gbox.y_range.max - gbox.y_range.min), 0.17578125);
    }

    #[test]
    fn test_short_box() {
        let gbox = super::geobox_for_hash("9q8ytn");
        assert_eq!(gbox.x_range.min, -122.4755859375);
        assert_eq!(gbox.y_range.max, 37.7435302734375);
    }
    //      "top": 37.781982421875,
    //      "bottom": 37.780609130859375,
    //      "left": -122.46185302734375,
    //      "right": -122.46047973632812,
    //
    #[test]
    fn test_happy_lounge() {
        assert_eq!(super::hash(37.7809921, -122.4600502, 7),
                   "9q8yvmg");
        let gbox = super::geobox_for_hash("9q8yvmg");
        assert_eq!(gbox.y_range.min, 37.780609130859375);
        assert_eq!(gbox.y_range.max, 37.781982421875);
        assert_eq!(gbox.x_range.min, -122.46047973632812);
    }
    #[test]
    fn test_9q8yvmu() {
        assert_eq!(super::hash(37.781303, -122.458461, 7),
                   "9q8yvmu");
        assert_eq!(super::val(37.781303, -122.458461, 7),
                   10411273850);
        assert_eq!(super::base32::decode("9q8yvmu"), 10411273850);
        let gbox = super::geobox_for_hash("9q8yvmu");
        assert_eq!(gbox.x_range.min, -122.4591064453125);
    }

}
