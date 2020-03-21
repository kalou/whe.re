extern crate gtfs_structures;
extern crate quadtree;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::error;

use std::thread;
use std::fmt;
use std::time;

#[derive(Clone, Copy)]
pub struct TripCursor<'a> {
    current_seq: u16,  // Index in the trip stops below // Mut self iterator?
    trip: &'a gtfs_structures::Trip
}

impl<'a> TripCursor<'a> {
    fn stop(&self) -> Option<&'a gtfs_structures::StopTime> {
        self.trip.stop_times.iter()
            .find(|st| st.stop_sequence == self.current_seq)
    }

    fn next_stop(&self) -> Option<&'a gtfs_structures::StopTime> {
        self.trip.stop_times.iter()
            .find(|st| st.stop_sequence == self.current_seq + 1)
    }

    // How long until next stop
    fn next_time(&self) -> u32 {
        if let Some(cur) = self.stop() {
            if let Some(next) = self.next_stop() {
                return next.arrival_time.unwrap_or(0) -
                        cur.departure_time.unwrap_or(0);
            }
        }

        return 0;
    }
}

impl<'a> fmt::Display for TripCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.stop() {
            Some(st) => write!(f, "{:?} at {} on {}", st.arrival_time,
                               st.stop.name, self.trip),
            None => write!(f, "Done on {}", self.trip)
        }
    }
}

impl<'a> fmt::Debug for TripCursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

// StopTime:
//arrival_time: Some(57679), stop: Stop { id: "3573", code: Some("13573"), name: "45th Ave &
//Cabrillo St", description: "", location_type: StopPoint, parent_station: None, longitude:
//Some(-122.505831), latitude: Some(37.773538), timezone: None, wheelchair_boarding:
//InformationNotAvailable }, departure_time: Some(57679), pickup_type: None, drop_off_type: None,
//stop_sequence: 53

impl<'a> Iterator for TripCursor<'a> {
    type Item = &'a gtfs_structures::StopTime;

    // Is there a next stop?
    fn next(&mut self) -> Option<&'a gtfs_structures::StopTime> {
        self.current_seq += 1;
        self.stop()
    }
}

struct IndexedGtfs {
    db: gtfs_structures::Gtfs, // The underlying GTFS db
    trips: HashMap<String, Vec<String>>, // HashMap from stop to all trip_ids from stop
}

// Each stop has a local ptr to its gtfs index for trip lookups
pub struct Stop(Arc<IndexedGtfs>, Arc<gtfs_structures::Stop>);

impl Stop {
    pub fn get_lon(&self) -> f64 {
        self.1.longitude.unwrap_or(0.00)
    }

    pub fn get_lat(&self) -> f64 {
        self.1.latitude.unwrap_or(0.00)
    }
}

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1.name)
    }
}

// And we keep all potential Gtfs stops in a big quadtree so they're
// indexed by position
impl quadtree::Point for Stop {
    fn get_x(&self) -> f64 {
        self.get_lon()
    }

    fn get_y(&self) -> f64 {
        self.get_lat()
    }
}

pub struct TransitMap {
    index: quadtree::QuadTree<Stop>,
}

impl TransitMap {
    pub fn new() -> Self {
        Self {
            index: quadtree::QuadTree::new(-180.0, -90.0, 360.0, 180.0),
        }
    }

    pub fn load(&mut self, file: &str) -> Result<usize, &str> {
        if let Ok(gtfs) = gtfs_structures::Gtfs::new(file) {
            let mut trips = HashMap::new();

            for (_, trip) in gtfs.trips.iter() {
                for stop_time in trip.stop_times.iter() {
                    let stop_trips = trips.entry(stop_time.stop.id.to_string()).or_insert(Vec::new());
                    if !stop_trips.contains(&trip.id) {
                        stop_trips.push(trip.id.to_string());
                    }
                }
            }

            let gtfs = Arc::new(IndexedGtfs{
                db: gtfs,
                trips: trips,
            });

            for (_, stop) in gtfs.db.stops.iter() {
                self.index.insert(Stop(Arc::clone(&gtfs),
                    Arc::clone(stop)))
            }

            Ok(gtfs.db.stops.len())
        } else {
            Err("Can't load it")
        }
    }

    pub fn get_stops(&self, lat: f64, lon: f64)
        -> impl Iterator<Item=&Stop> {
        self.index.get(lon, lat)
    }
}

impl Stop {
    // 0 db
    // 1 stop
    fn stop(&self) -> &gtfs_structures::Stop {
        self.1.as_ref()
    }

    // Next trip cursors for a given stop
    pub fn next_trips(&self) -> Vec<TripCursor> {
        self.0.trips.get(&self.1.id).unwrap_or(&vec![])
            .iter()
            .filter_map(|trip_id| self.0.db.trips.get(trip_id))
            .map(|trip| {
                // Instanciate the trip - advance it to current stop
                let mut trip_curs = TripCursor {
                    current_seq: 1,
                    trip: trip
                };
                // println!("Trip {}", trip_curs);
                // This advances the cursor to next stop
                // Ignore the result
                let _ = trip_curs.find(|stop_time|
                   stop_time.stop.id == self.1.id);
                trip_curs
            }).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    fn simple_load_file() -> super::TransitMap {
        let mut now = Instant::now();
        let mut tm = super::TransitMap::new();
        assert!(tm.load("gtfs.zip").is_ok());
        println!("Load in {:?}", now.elapsed());
        tm
    }

    #[test]
    fn test_time() {
        let mut tm = simple_load_file();
        let stops = tm.get_stops(37.7790697, -122.4436235).take(1);
        for st in stops {
            println!("Checking stop: {:?}", st.stop());
            for trip in st.next_trips().iter().take(10) {
                println!("Trip {} will reach {:?} in {} secs", trip,
                         trip.next_stop(), trip.next_time());
            }
        }
    }

    fn test_stops() {
        let mut now = Instant::now();
        let tm = simple_load_file();
        now = Instant::now();
        // stairs
        // let stops = tm.get_stops(37.7790697, -122.4436235).take(1);
        let stops = tm.get_stops(37.789489, -122.401263).take(1);
        println!("Stops in {:?}", now.elapsed());
        now = Instant::now();
        let mut cnt = 0;
        for st in stops {
            println!("Checking stop: {:?}", st.stop());
            now = Instant::now();
            for trip in st.next_trips() {
                println!("Trip {} in {:?}", trip, now.elapsed());
                //for stop in trip {
                //   println!("-- and next stop is {:?}", stop);
                //}
            }
        }

        //std::thread::sleep(std::time::Duration::from_secs(100));
    }
}
