use super::{
    new_dispatcher_thread, new_worker_thread, InputDataVecOption, OutputDataVecOption, Renderer,
};


pub struct ThreadPoolRenderer {
    thread_number: u32,
    depth: u32,
    worker_threads: Option<Vec<JoinHandle<()>>>,

    world: Arc<RwLock<Scene>>,
    is_started: bool,
    num_finished: u32,
}