pub mod point;
pub mod polygon;
pub mod schema;

pub use point::*;
pub use polygon::*;

#[derive(Debug, PartialEq, Eq)]
pub enum CCWResult {
    Clockwise,
    CounterClockwise,
    OnLine,
}

pub fn ccw(p1: &Point, p2: &Point, p3: &Point) -> CCWResult {
    let b = *p2 - *p1;
    let c = *p3 - *p1;
    let cr = Point::cross(b, c);
    if cr < 0. {
        CCWResult::CounterClockwise
    } else if cr > 0. {
        CCWResult::Clockwise
    } else {
        CCWResult::OnLine
    }
}

// Returns true iff p1-p2 crosses p3-p4. Returns false if it is crossing on a vertex.
pub fn is_crossing(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> bool {
    let ccw1 = ccw(p1, p2, p3);
    let ccw2 = ccw(p1, p2, p4);
    let ccw3 = ccw(p3, p4, p1);
    let ccw4 = ccw(p3, p4, p2);
    if ccw1 == CCWResult::OnLine || ccw2 == CCWResult::OnLine || ccw3 == CCWResult::OnLine || ccw4 == CCWResult::OnLine {
        return false
    }

    ccw1 != ccw2 && ccw3 != ccw4
}

pub fn is_crossing_with_hint(p1: &Point, p2: &Point, p3: &Point, p4: &Point, ccw_hint: CCWResult) -> (bool, CCWResult) {
    let ccw2 = ccw(p1, p2, p4);
    if ccw_hint == CCWResult::OnLine || ccw2 == CCWResult::OnLine || ccw_hint == ccw2 {
        return (false, ccw2);
    }
    let ccw3 = ccw(p3, p4, p1);
    if ccw3 == CCWResult::OnLine {
        return (false, ccw2);
    }
    let ccw4 = ccw(p3, p4, p2);
    if ccw4 == CCWResult::OnLine {
        return (false, ccw2);
    }
    (ccw3 == ccw4, ccw2)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
