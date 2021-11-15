use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex, RwLock,
    },
    thread::JoinHandle,
};

use crate::algebra::Vector3d;
use crate::camera::Camera;
use crate::world::World;
use itertools::Itertools;

use super::{InputDataVecOption, OutputDataVecOption, Renderer, new_dispatcher_thread, new_worker_thread};

pub struct ThreadPoolRenderer {
    thread_number: u32,
    depth: u32,
    worker_threads: Option<Vec<JoinHandle<()>>>,

    input_sender: Arc<Mutex<Sender<InputDataVecOption>>>,
    input_receiver: Arc<Mutex<Receiver<InputDataVecOption>>>,

    output_sender: Arc<Mutex<Sender<OutputDataVecOption>>>,
    output_receiver: Receiver<OutputDataVecOption>,

    // control_sender: Sender<()>,
    // control_receiver: Arc<Mutex<Receiver<()>>>,
    parking: Arc<(Mutex<bool>, Condvar)>,

    world: Arc<RwLock<World>>,
    is_started: bool,
}

impl ThreadPoolRenderer {
    pub fn new(world: Arc<RwLock<World>>, thread_number: u32, depth: u32) -> ThreadPoolRenderer {
        let (input_sender, input_receiver) = channel();
        let (output_sender, output_receiver) = channel();
        // let (control_sender, control_receiver) = channel();
        let mut result = ThreadPoolRenderer {
            thread_number,
            depth,
            worker_threads: None,
            input_sender: Arc::new(Mutex::new(input_sender)),
            input_receiver: Arc::new(Mutex::new(input_receiver)),
            output_sender: Arc::new(Mutex::new(output_sender)),
            output_receiver,
            // control_sender,
            // control_receiver: Arc::new(Mutex::new(control_receiver)),
            parking: Arc::new((Mutex::new(false), Condvar::new())),
            world,
            is_started: false,
        };

        let threads = (0..thread_number)
            .map(|i| {
                new_worker_thread(
                    i,
                    result.input_receiver.clone(),
                    result.output_sender.clone(),
                    result.world.clone(),
                    result.parking.clone(),
                    result.depth,
                )
            })
            .collect_vec();

        result.worker_threads = Some(threads);

        result
    }

    pub fn render_step_to(&mut self, camera: Arc<RwLock<Camera>>, buffer: &mut Vec<Vector3d>) {
        let width = camera.read().unwrap().image().width;
        let height = camera.read().unwrap().image().height;

        new_dispatcher_thread(camera, width, height, 1, self.input_sender.clone(), self.thread_number);

        let (lock, cvar) = &*self.parking;
        {
            let mut running = lock.lock().unwrap();
            *running = true;
            cvar.notify_all();
        }

        let mut finished = 0;
        for results in &self.output_receiver {
            let results = match results {
                Some(v) => v,
                None => {
                    finished += 1;
                    if finished == self.thread_number {
                        let mut running = lock.lock().unwrap();
                        *running = false;

                        break;
                    }
                    continue;
                }
            };

            // println!("Received {}", results.len());
            if self.is_started {
                for (index, color) in results {
                    let current_color = &buffer[index as usize];
                    buffer[index as usize] = &color + current_color;
                }
            } else {
                for (index, color) in results {
                    buffer[index as usize] = color;
                }
            }
        }

        if self.is_started {
            self.is_started = true;
        }
    }
}

impl Renderer for ThreadPoolRenderer {
    fn stop_rendering(&mut self) {
        self.is_started = false;
    }

    fn start_rendering(&mut self, camera: Arc<RwLock<Camera>>, samples_number: u32) {
        let width = camera.read().unwrap().image().width;
        let height = camera.read().unwrap().image().height;

        new_dispatcher_thread(
            camera,
            width,
            height,
            samples_number,
            self.input_sender.clone(),
            self.thread_number,
        );
    }

    fn render_step(&mut self, buffer: &mut Vec<Vector3d>) -> bool {
        let (lock, cvar) = &*self.parking;
        {
            let mut running = lock.lock().unwrap();
            *running = true;
            cvar.notify_all();
        }

        let mut finished = 0;
        for results in &self.output_receiver {
            let results = match results {
                Some(v) => v,
                None => {
                    finished += 1;
                    if finished == self.thread_number {
                        let mut running = lock.lock().unwrap();
                        *running = false;

                        // println!("All finished");
                        break;
                    }
                    // println!("{} finished", finished);
                    continue;
                }
            };

            // println!("Received {}", results.len());
            for (index, color) in results {
                buffer[index as usize] = color;
            }
        }

        // println!("Rendering finished");
        // dispather.join().unwrap();

        true
    }
}