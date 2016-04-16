use cgmath::{Vector2, EuclideanVector};
use rand::{StdRng, Rng};
use tiles::Tiles;
use items::{Items, ItemState};

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
        println!("Work entry {} was finished", id);
    }

    pub fn get(&self, id: u32) -> &WorkEntry {
        self.entries.get(id as usize).unwrap().as_ref().unwrap()
    }
}

#[derive(Debug)]
enum RobotState {
    Waiting,
    Building(Vector2<u32>),
    Moving(Vector2<f32>, f32), // target, speed multiplier
    Sleep(f32), // time
    PickUp(u32), // target item
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
    inventory: Option<u32>, // an item's id
}

impl Robot {
    pub fn new(position: Vector2<f32>) -> Self {
        Robot {
            id: None,
            position: position,
            assigned_work: None,
            current_state: RobotState::Waiting,
            state_stack: Vec::new(),
            inventory: None,
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    fn notify_of_work(&mut self, work: &WorkEntry) {
        self.assigned_work = Some(work.id());
    }

    fn update(&mut self, delta: f32, items: &mut Items, tiles: &mut Tiles, work: &mut WorkQueue, rng: &mut StdRng) {
        // If we have an inventory, make sure it's following us
        if let Some(target) = self.inventory {
            let mut item = items.get_mut(target).unwrap();
            item.position = self.position + Vector2::new(0.0, -0.3);
        }

        // Update for the specific state
        match self.current_state {
            RobotState::Waiting => {
                // If we have work, find something to do
                if let Some(work_id) = self.assigned_work {
                    // First, get the position where the work is
                    let pos = work.get(work_id).target_tile();

                    // Set the state to building and tell the robot to move there
                    self.current_state = RobotState::Building(pos);

                    println!("Robot {} switched to Building", self.id.unwrap());
                } else {
                    // Clamp the position
                    let tile = self.position.cast::<u32>().cast::<f32>();

                    // Add a bit of randomness
                    let new_pos = tile +
                        Vector2::new(rng.gen_range(0.0, 0.4), rng.gen_range(0.0, 0.4)) +
                        Vector2::new(0.3, 0.3);

                    // Finally, start the movement
                    self.push_state(RobotState::Sleep(0.5));
                    self.push_state(RobotState::Moving(new_pos, 0.25));
                    println!("Robot {} is idling", self.id.unwrap());
                }
            },
            RobotState::Building(pos) => {
                let mut tile = tiles.get_mut(pos.x, pos.y).unwrap();
                assert!(tile.is_under_construction());

                // Check if the building needs more resources
                if tile.construction_needs_resources() {
                    // This will be needed if more than one robot can construct a thing at a time
                    //tile.promise_resource();

                    // Check if we currently have an item
                    if self.inventory.is_some() {
                        // We have a resource, check if we're already at the building site
                        if self.position.cast::<u32>() == pos {
                            // We are, deposit the resource
                            items.remove(self.inventory.unwrap());
                            self.inventory = None;
                            tile.apply_resource();
                        } else {
                            // We're not, move there
                            self.push_state(RobotState::Moving(pos.cast::<f32>() + Vector2::new(0.5, 0.5), 1.0));
                        }
                    } else {
                        // Find a resource to claim
                        // TODO: Move this to a start behavior for the PickUp state
                        let claimed = items.claim_resource(self.position);
                        println!("Robot {} claimed item {:?}", self.id.unwrap(), claimed);

                        if claimed.is_none() {
                            // If we didn't find anything, just wait a bit
                            self.push_state(RobotState::Sleep(1.0));
                        } else {
                            // If we did find something, move to pick it up
                            self.push_state(RobotState::PickUp(claimed.unwrap()));
                        }
                    }
                } else {
                    let finished = tile.apply_build_time(delta);

                    if finished {
                        // Mark the work as done
                        work.finish(self.assigned_work.unwrap());
                        self.assigned_work = None;
                        self.reset_state(RobotState::Waiting);
                        println!("Robot {} finished building", self.id.unwrap());
                    }
                }
            },
            RobotState::Moving(pos, speed_multiplier) => {
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
                    self.position = self.position + (direction * move_speed * speed_multiplier);
                }
            }
            RobotState::Sleep(mut amount) => {
                amount -= delta;
                if amount <= 0.0 {
                    self.pop_state();
                } else {
                    self.current_state = RobotState::Sleep(amount);
                }
            },
            RobotState::PickUp(target) => {
                // Check if we're within pickup range of the target
                let item = items.get_mut(target).unwrap();
                let pos = item.position() + Vector2::new(0.0, 0.3); // A bit above because it looks better
                let distance = pos - self.position;

                if distance.magnitude2() < 0.1*0.1 {
                    self.inventory = Some(target);
                    item.state = ItemState::Carried;
                    self.pop_state();
                } else {
                    // We aren't close enough, move to it
                    self.push_state(RobotState::Moving(pos, 1.0));
                }
            }
        }
    }

    fn push_state(&mut self, mut state: RobotState) {
        println!("State being pushed: {:?}", state);
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

    pub fn update(&mut self, delta: f32, items: &mut Items, tiles: &mut Tiles, work: &mut WorkQueue, rng: &mut StdRng) {
        self.assign_work(work);

        // Now that all work is assigned, update the robots
        for robot in &mut self.robots {
            robot.update(delta, items, tiles, work, rng);
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
