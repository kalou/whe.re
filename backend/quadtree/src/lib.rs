#[macro_use]
extern crate log;
extern crate env_logger;

use std::fmt::Debug;
use std::iter;
use std::borrow::Borrow;
use std::cmp::Ordering::Equal;

// A point in our hash
pub trait Point where Self: Sized {
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
}

#[derive(Debug)]
pub struct QuadCell<T: Point> {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub xsize: f64,
    pub ysize: f64,
    left: Option<Box<QuadCell<T>>>,
    right: Option<Box<QuadCell<T>>>,
    values: Option<Vec<T>>,
    capacity: usize // Max number of rows per cell -
                    // also impacts precision/distance of get
}

#[derive(Debug)]
pub struct QuadTree<T: Point> {
    root: QuadCell<T>,
    cell_capacity: usize
}

impl<T: Point> QuadTree<T> {
    pub fn new(x:f64, y: f64, xsize: f64, ysize: f64) -> Self {
        debug!("Initializing QuadTree {},{} _ size {},{}", x, y,
               xsize, ysize);
        Self {
            root: QuadCell::new(0, x, y, xsize, ysize, 200),
            cell_capacity: 200
        }
    }

    pub fn of_capacity(x: f64, y: f64, xsize: f64, ysize: f64, capacity: usize)
        -> Self {
        debug!("Initializing QuadTree with capa={} {},{} _ size {},{}", 
               capacity, x, y, xsize, ysize);
        Self {
            root: QuadCell::new(0, x, y, xsize, ysize, capacity),
            cell_capacity: capacity
        }
    }

    pub fn insert(&mut self, pt: T) {
        self.root.insert(pt);
    }

    pub fn get(&self, x: f64, y: f64) -> impl Iterator<Item=&T> {
        self.root.get(x, y)
    }

    pub fn get_cell(&self, x: f64, y: f64) -> &QuadCell<T> {
        self.root.get_cell(x, y)
    }

    pub fn walk(&self) -> impl Iterator<Item=&QuadCell<T>> {
        self.root.walk()
    }
}

impl<T: Point> QuadCell<T> {
    // Not self, helper allocates
    fn new(id: u64, x: f64, y: f64, xsize: f64,
           ysize: f64, capacity: usize) -> Self {
        QuadCell {
            id: id,
            x: x,
            y: y,
            xsize: xsize,
            ysize: ysize,
            left: None,
            right: None,
            values: Some(Vec::new()),
            capacity: capacity
        }
    }

    pub fn walk<'a>(&'a self) -> Box<dyn Iterator<Item = &'a QuadCell<T>> + 'a> {
        let left_iter = match &self.left {
                Some(quad) => quad.walk(),
                None => Box::new(iter::once(self))
        };

        let right_iter = match &self.right {
                Some(quad) => quad.walk(),
                None => Box::new(iter::empty())
        };

        // If I'm a leaf, I'll emit self chained with empty
        // otherwise, emit descent to children
        Box::new(left_iter.chain(right_iter))
    }

    // Add a value into the current v
    fn insert(&mut self, v: T) {
        trace!("inserting T({},{}) in {},{}/{},{}", &v.get_x(), &v.get_y(),
               self.x, self.y, self.xsize, self.ysize);

        if self.x > v.get_x() || self.y > v.get_y() ||
            self.x + self.xsize < v.get_x() ||
            self.y + self.ysize < v.get_y() {
            warn!("Not inserting out of bounds - should be {}<{}<{}, {}<{}<{}",
                  self.x, v.get_x(), self.x+self.xsize,
                  self.y,v.get_y(), self.y+self.ysize);
        }
        /*
        assert!(self.x < v.get_x());
        assert!(self.y < v.get_y());
        assert!(self.x + self.xsize > v.get_x());
        assert!(self.y + self.ysize > v.get_y());
        */


        if self.len() >= self.capacity {
            self.split();
        }
        match self.values.as_mut() {
            Some(vec) => {
                vec.push(v);
            },
            None => {
                // We have no values: it means we split - we must have children
                // let's recursively call insert on them
                trace!("insert going down into split");
                self.step_towards_mut(v.get_x(), v.get_y())
                    .as_mut().map(|cell| cell.insert(v));
            }
        }
    }

    fn len(&self) -> usize {
        self.values.as_ref().map(|x| x.len()).unwrap_or(0)
    }

    // Get values from the cell -- otherwise, we'll follow children
    fn get<'a> (&'a self, x: f64, y: f64) -> impl Iterator<Item=&T> {
        let mut vals: Vec<&T> = self.get_cell(x, y).values.as_ref().unwrap()
            .iter()
            .collect();

        let square_dist_to_xy = |pt: &T, x: f64, y: f64|
         (pt.get_x() - x).powf(2.0) + (pt.get_y() -y).powf(2.0);

        vals.sort_unstable_by(|a, b|
          square_dist_to_xy(a, x, y)
          .partial_cmp(&square_dist_to_xy(b, x, y)).unwrap_or(Equal));

        vals.into_iter()
    }

    fn get_cell(&self, x: f64, y: f64) -> &Self {
        match self.values.as_ref() {
            Some(_) => {
                self
            },
            None => {
                // We have no values: it means we split - we must have children
                // let's recursively call values on the nearest
                self.step_towards(x, y)
                    .as_ref().map(|cell| cell.get_cell(x, y)).unwrap()
            }
        }
    }

    // Split our existing values into two sub-children
    // We will become non-leaf
    fn split(&mut self) {
        // This removes values and creates our children
        trace!("splitting");
        let vals = self.values.take();
        self.extend();
        match vals {
            Some(values) => {
                for v in values {
                    self.step_towards_mut(v.get_x(), v.get_y())
                        .as_mut()
                        .map(|cell| {
                            cell.insert(v);
                        });
                }
            },
            None => {
                warn!("split on empty values");
            }
        }
    }

    // Jump towards leaf - extending
    fn extend(&mut self) {
        let half_x:f64 = self.xsize/2.0;
        let half_y:f64 = self.ysize/2.0;
        // This has to match the getter below!
        if self.xsize >= self.ysize {
            self.left = Some(Box::new(
                    Self::new((self.id << 1) | 0, self.x, self.y, half_x,
                              self.ysize, self.capacity))
            );
            self.right = Some(Box::new(
                    Self::new((self.id << 1) | 1, self.x + half_x, self.y,
                              half_x, self.ysize, self.capacity))
            );
        } else {
            self.left = Some(Box::new(
                    Self::new((self.id << 1) | 0, self.x, self.y, self.xsize,
                              half_y, self.capacity))
            );
            self.right = Some(Box::new(
                    Self::new((self.id << 1) | 1, self.x, self.y + half_y,
                              self.xsize, half_y, self.capacity))
            );
        }
    }

    // Jump towards leaf - read-only
    fn step_towards(&self, x: f64, y: f64) -> &Option<Box<QuadCell<T>>> {
        let half_x:f64 = self.xsize/2.0;
        let half_y:f64 = self.ysize/2.0;

        if self.xsize >= self.ysize {
            if x <= self.x + half_x {
                &self.left
            } else {
                &self.right
            }
        } else {
            if y <= self.y + half_y {
                &self.left
            } else {
                &self.right
            }
        }
    }

    // Jump towards leaf - mut
    fn step_towards_mut(&mut self, x: f64, y: f64) -> &mut Option<Box<QuadCell<T>>> {
        let half_x:f64 = self.xsize/2.0;
        let half_y:f64 = self.ysize/2.0;

        if self.xsize >= self.ysize {
            if x <= self.x + half_x {
                &mut self.left
            } else {
                &mut self.right
            }
        } else {
            if y <= self.y + half_y {
                &mut self.left
            } else {
                &mut self.right
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::QuadTree;

    #[derive(Debug)]
    struct SomePoint {
        x: f64,
        y: f64
    }

    impl super::Point for &SomePoint {
        fn get_x(&self) -> f64 {
            self.x
        }
        fn get_y(&self) -> f64 {
            self.y
        }
    }
    impl super::Point for SomePoint {
        fn get_x(&self) -> f64 {
            self.x
        }
        fn get_y(&self) -> f64 {
            self.y
        }
    }

    // Cargo test -- --nocapture should work if you call init
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn create_empty() {
        let x = QuadTree::<&SomePoint>::new(0.0, 0.0, 0.39, 0.22);
        assert_eq!(x.root.len(), 0)
    }

    #[test]
    fn insert_once() {
        let pt = SomePoint{x: 1.0, y: 1.0};
        let mut x = QuadTree::<SomePoint>::new(0.0, 0.0, 2.39, 2.22);
        x.insert(pt);
        assert_eq!(x.root.len(), 1)
    }

    #[test]
    fn split_ok() {
        let pt1 = SomePoint{x: 0.5, y: 0.5};
        let pt2 = SomePoint{x: 0.5, y: 0.5};
        let pt3 = SomePoint{x: 1.4, y: 1.4};
        let pt4 = SomePoint{x: 1.4, y: 1.4};
        let mut x = QuadTree::<SomePoint>::new(0.0, 0.0, 2.0, 2.0);
        x.root.capacity = 2;
        x.insert(pt1);
        x.insert(pt2);
        x.insert(pt3);
        x.insert(pt4);
        assert_eq!(x.get(1.4, 1.4).collect::<Vec<&SomePoint>>().len(), 2)
    }

    #[test]
    fn test_sorted() {
        let pt1 = SomePoint{x: 0.5, y: 0.5};
        let pt2 = SomePoint{x: 0.6, y: 0.6};
        let pt3 = SomePoint{x: 1.4, y: 1.4};
        let pt4 = SomePoint{x: 1.9, y: 1.9};
        let pt5 = SomePoint{x: 2.9, y: 2.9};
        let mut x = QuadTree::<SomePoint>::new(0.0, 0.0, 3.0, 3.0);
        x.root.capacity = 6;
        x.insert(pt1);
        x.insert(pt2);
        x.insert(pt3);
        x.insert(pt4);
        x.insert(pt5);
        assert_eq!(x.get(1.4, 1.4).map(|x| x.x).collect::<Vec<f64>>(),
            vec![1.4, 1.9, 0.6, 0.5, 2.9]);
    }


    #[test]
    fn first_box_halfed() {
        let pt = SomePoint{x: 1.0, y: 1.0};
        let mut x = QuadTree::<SomePoint>::new(0.0, 0.0, 8.0, 8.0);
        x.insert(pt);
        for x in x.get(1.0, 1.0) {
            trace!("parsing {:?}", x)
        }
    }

    #[test]
    fn iterate() {
        init();

        // Perfect 4x4 with 1 children per square
        let mut x = QuadTree::<SomePoint>::of_capacity(0.0, 0.0, 4.0, 4.0, 1);
        // Should lead to 1 value  per cell
        for a in 0..4 {
            for b in 0..4 {
                x.insert(SomePoint {
                    x: 0.5 + a as f64, y: 0.5 + b as f64
                });
            }
        }

        for cell in x.walk() {
            debug!("cell id={}, {},{} of size {},{} has {:?}",
                   cell.id, cell.x, cell.y, cell.xsize, cell.ysize,
                   cell.values);
            assert_eq!(cell.len(), 1)
        }

        assert_eq!(x.walk()
           .collect::<Vec<&super::QuadCell<SomePoint>>>().len(), 16)
    }
}
