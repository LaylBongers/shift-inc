use cgmath::{Vector2, EuclideanVector};
use tiles::Tiles;

#[derive(Debug)]
pub struct WorkEntry {
    id: Option<u32>, // TODO: Sparate work template
    target_tile: Vector2<u32>,
    assigned_robot: Option<u32>,
}

impl WorkEntry {
    pub fn new(target_tile: Vector2<u32>) -> Self {
        WorkEntry {
            id: None,
            target_tile: target_tile,
            assigned_robot: None,
        }
    }

    fn assign(&mut self, robot: &mut Robot) {
        self.assigned_robot = Some(robot.id.unwrap());
        robot.notify_of_work(self);
        println!("Assigned {} at {:?} to {}", self.assigned_robot.unwrap(), self.target_tile, "TODO");
    }

    fn id(&self) -> u32 {
        self.id.unwrap()
    }

    fn target_tile(&self) -> Vector2<u32> {
        self.target_tile
    }
}

pub struct WorkQueue {
    entries: Vec<Option<WorkEntry>>, // TODO: Share functionality with items
}

impl WorkQueue {
    pub fn new() -> WorkQueue {
        WorkQueue {
            entries: Vec::new(),
        }
    }

    pub fn publish(&mut self, mut entry: WorkEntry) {

        // Find an empty slot
        for i in 0..self.entries.len() {
            let slot = &mut self.entries[i];
            if slot.is_some() { continue; }

            // Found a slot
            entry.id = Some(i as u32);
            println!("Publishing: {:?}", entry);
            *slot = Some(entry);
            return;
        }

        // Couldn't find one, add a new one
        entry.id = Some(self.entries.len() as u32);
        println!("Publishing: {:?}", entry);
        self.entries.push(Some(entry));
    }

    pub fn request(&mut self) -> Option<&mut WorkEntry> {
        self.entries.iter_mut()
            .find(|e| e.is_some() && e.as_ref().unwrap().assigned_robot.is_none())
            .map(|e| e.as_mut().unwrap())
    }

    pub fn finish(&mut self, id: u32) {
        self.entries[id as usize] = None;
        println!("Finished work: {}", id);
    }

    pub fn get(&self, id: u32) -> &WorkEntry {
        self.entries.get(id as usize).unwrap().as_ref().unwrap()
    }
}

enum RobotState {
    Waiting,
    Building(Vector2<u32>),
    Moving(Vector2<f32>),
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
    assigned_work: Option<u32>, // id of the work
    current_state: RobotState,
    state_stack: Vec<RobotState>,
}

impl Robot {
    pub fn new(position: Vector2<f32>) -> Self {
        Robot {
            id: None,
            position: position,
            assigned_work: None,
            current_state: RobotState::Waiting,
            state_stack: Vec::new(),
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn notify_of_work(&mut self, work: &WorkEntry) {
        self.assigned_work = Some(work.id());
    }

    fn update(&mut self, delta: f32, tiles: &mut Tiles, work: &mut WorkQueue) {
        match self.current_state {
            RobotState::Waiting => {
                // If we have work, find something to do
                if let Some(work_id) = self.assigned_work {
                    let pos = work.get(work_id).target_tile();
                    self.current_state = RobotState::Building(pos);
                    println!("Robot {} switched to Building", self.id.unwrap());
                } else {
                    let new_pos = self.position + Vector2::new(0.0, 1.0);
                    self.push_state(RobotState::Moving(new_pos));
                    println!("Robot {} is idling", self.id.unwrap());
                }
            },
            RobotState::Building(pos) => {
                // Build instantly
                // TODO: Move there first
                let mut tile = tiles.get_mut(pos.x, pos.y).unwrap();
                let finished = tile.apply_build_time(delta);

                if finished {
                    // Mark the work as done
                    work.finish(self.assigned_work.unwrap());
                    self.assigned_work = None;
                    self.reset_state(RobotState::Waiting);
                }
            },
            RobotState::Moving(pos) => {
                // Check how far we still need to move
                let difference = pos - self.position;
                let move_speed = delta * 0.5;

                // Check if this frame we'll be there
                if difference.magnitude2() <= move_speed*move_speed {
                    self.position = pos;
                    self.pop_state();
                } else {
                    // If not, move the distance we can
                    let direction = difference.normalize();
                    self.position = self.position + (direction * move_speed);
                }
            }
        }
    }

    fn push_state(&mut self, mut state: RobotState) {
        ::std::mem::swap(&mut state, &mut self.current_state);
        self.state_stack.push(state);
    }

    fn pop_state(&mut self) {
        self.current_state = self.state_stack.pop().unwrap();
    }

    fn reset_state(&mut self, state: RobotState) {
        self.state_stack.clear();
        self.current_state = state;
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

    pub fn for_each<F: FnMut(&Robot)>(&self, mut f: F) { // TODO: Perhaps just return a vec
        for robot in &self.robots {
            //if let &Some(ref robot) = robot {
            f(robot);
            //}
        }
    }

    pub fn update(&mut self, delta: f32, tiles: &mut Tiles, work: &mut WorkQueue) {
        self.assign_work(work);

        // Now that all work is assigned, update the robots
        for robot in &mut self.robots {
            robot.update(delta, tiles, work);
        }
    }

    fn assign_work(&mut self, work: &mut WorkQueue) {
        // Get all waiting robots
        let mut waiting_robots: Vec<_> = self.robots.iter_mut().filter(|r| r.current_state.is_waiting()).collect();

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
