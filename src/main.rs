extern crate kompact;

use kompact::default_components::*;
use kompact::*;

pub struct TaskPort;

impl Port for TaskPort {
    type Indication = ();
    type Request = String;
}

#[derive(ComponentDefinition)]
pub struct Task {
    ctx: ComponentContext<Task>,
    task_port: ProvidedPort<TaskPort, Task>,
}

impl Task {
    pub fn new() -> Task {
        Task {
            ctx: ComponentContext::new(),
            task_port: ProvidedPort::new(),
        }
    }
}

impl Provide<ControlPort> for Task {
    fn handle(&mut self, event: ControlEvent) -> () {
        if let ControlEvent::Start = event {
            println!("We are starting up!");
        }
    }
}

impl Actor for Task {
    fn receive_local(&mut self, _sender: ActorRef, msg: &Any) {
        // Any msg can be sent as a Actor msg, hence we need to
        // dynamically check its type
        if let Some(event) = msg.downcast_ref::<String>() {
            println!(" Oh I got a msg: {}", event);
        }
    }
    fn receive_message(&mut self, _sender: ActorPath, ser_id: u64, buf: &mut Buf) {
        // Receive msg from other component over the network.
        // Empty for now..
    }
}

impl Provide<TaskPort> for Task {
    fn handle(&mut self, event: String) {
        println!(" Oh I got a msg: {}", event);
    }
}

fn main() {
    let mut cfg = KompactConfig::new();
    cfg.system_components(DeadletterBox::new, NetworkConfig::default().build());
    let system = KompactSystem::new(cfg).expect("KompactSystem");

    // Create the Component and have it registered within the KompactSystem
    let (task, _) = system.create_and_register(move || Task::new());

    // Access the port inside the Component in order to later trigger on it.
    let task_port = task.on_definition(|c| c.task_port.share());

    // Start the Component
    system.start(&task);

    // Send actor message to the Component
    task.actor_ref()
        .tell(Box::new(String::from("actor_msg")), &task);

    // trigger the Components task port
    // NOTE: normally we do the trigger from other components
    system.trigger_r(String::from("component_msg"), task_port);

    // Blocks until program is terminated...
    system.await_termination();
}
