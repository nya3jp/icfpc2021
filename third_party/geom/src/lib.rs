mod point;
mod polygon;

pub use point::Point;
pub use polygon::Polygon;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
