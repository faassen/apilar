struct Rectangle {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

struct Connection {
    from_rect: Rectangle,
    from_id: usize,
    to_rect: Rectangle,
    to_id: usize,
    transmit_frequency: u32, // use some kind of type, newtype? time-based?
}

struct Island {
    connections: Vec<Connection>,
    // configurables
}

struct World {
    islands: Vec<Island>,
}
