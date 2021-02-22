use crate::config::EPSILON;

pub trait Approx<T> {
    fn approx(&self, other: &T) -> bool;
}

impl Approx<bool> for bool {
    fn approx(&self, other: &bool) -> bool {
        self == other
    }
}

impl Approx<usize> for usize {
    fn approx(&self, other: &usize) -> bool {
        self == other
    }
}

impl Approx<f64> for f64 {
    fn approx(&self, other: &f64) -> bool {
        (self - other).abs() < EPSILON
    }
}

impl<T> Approx<Vec<T>> for Vec<T>
where
    T: Approx<T>,
{
    fn approx(&self, other: &Vec<T>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for i in 0..self.len() {
            if !self[i].approx(&other[i]) {
                return false;
            }
        }

        true
    }
}
