use cgmath::Vector2;

pub struct Robot {
    position: Vector2<f32>
}

impl Robot {
    pub fn new(position: Vector2<f32>) -> Self {
        Robot {
            position: position
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }
}

pub struct Robots {
    robots: Vec<Robot>,
}

impl Robots {
    pub fn new() -> Self {
        Robots {
            robots: Vec::new(),
        }
    }

    pub fn add(&mut self, robot: Robot) {
        self.robots.push(robot);
    }

    pub fn for_each<F: FnMut(&Robot)>(&self, mut f: F) {
        for robot in &self.robots {
            //if let &Some(ref robot) = robot {
            f(robot);
            //}
        }
    }
}
