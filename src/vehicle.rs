


pub struct Vehicle {
    pos: (i16, i16),

    cargo: u32, // TODO

    route: u32,
    // specify where we are on the route?
}

impl Vehicle {
    pub fn new() -> Self {
        Vehicle {
            pos: 0, 0
        }
    }
}

