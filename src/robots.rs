use cgmath::Vector2;

#[derive(Debug)]
pub struct WorkEntry {
    target_tile: Vector2<u32>,
    assigned_robot: Option<u32>,
}

impl WorkEntry {
    pub fn new(target_tile: Vector2<u32>) -> Self {
        WorkEntry {
            target_tile: target_tile,
            assigned_robot: None,
        }
    }

    fn assign(&mut self, robot: &mut Robot) {
        self.assigned_robot = Some(robot.id.unwrap());
        println!("Assigned {} at {:?} to {}", self.assigned_robot.unwrap(), self.target_tile, "TODO");
    }
}

pub struct WorkQueue {
    entries: Vec<WorkEntry>,
}

impl WorkQueue {
    pub fn new() -> WorkQueue {
        WorkQueue {
            entries: Vec::new(),
        }
    }

    pub fn publish(&mut self, entry: WorkEntry) {
        println!("Published: {:?}", entry);
        self.entries.push(entry);
    }

    pub fn request(&mut self) -> Option<&mut WorkEntry> {
        self.entries.iter_mut().find(|e| e.assigned_robot.is_none())
    }
}

enum RobotState {
    Waiting,
    _DoNotMatch,
}

impl RobotState {
    fn is_waiting(&self) -> bool {
        if let &RobotState::Waiting = self {
            true
        } else {
            false
        }
    }
}

pub struct Robot {
    id: Option<u32>, // TODO: Sparate robot template
    position: Vector2<f32>,
    state: RobotState,
}

impl Robot {
    pub fn new(position: Vector2<f32>) -> Self {
        Robot {
            id: None,
            position: position,
            state: RobotState::Waiting,
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

    pub fn add(&mut self, mut robot: Robot) {
        robot.id = Some(self.robots.len() as u32);
        self.robots.push(robot);
    }

    pub fn for_each<F: FnMut(&Robot)>(&self, mut f: F) {
        for robot in &self.robots {
            //if let &Some(ref robot) = robot {
            f(robot);
            //}
        }
    }

    pub fn update(&mut self, work: &mut WorkQueue) {
        // Get all waiting robots
        let mut waiting_robots: Vec<_> = self.robots.iter_mut().filter(|r| r.state.is_waiting()).collect();

        // Go over all available work items
        let mut entries = 0;
        while waiting_robots.len() != 0 {
            let work = {
                if let Some(work) = work.request() {
                    work
                } else {
                    break;
                }
            };

            // Assign the first available robot
            // TODO: Find closest
            let robot = waiting_robots.swap_remove(0);

            // Assign the robot to the work item
            work.assign(robot);
            entries += 1;
        }

        if entries != 0 {
            println!("Processed {} work entries", entries);
        }
    }
}
