use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex, RwLock,
    },
    thread::{spawn, JoinHandle},
};

use crate::algebra::Vector3d;
use crate::camera::{Camera, ray_caster::MultisamplerRayCaster};
use crate::world::{Ray, World};
use itertools::Itertools;

use super::ray_color;

type InputData = (u32, u32, Vec<Ray>);
type InputDataVec = Vec<InputData>;
type InputDataVecOption = Option<InputDataVec>;

type OutputData = (u32, u32, Vector3d);
type OutputDataVec = Vec<OutputData>;
type OutputDataVecOption = Option<OutputDataVec>;

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
            .map(|i| result.new_worker_thread(i))
            .collect_vec();

        result.worker_threads = Some(threads);

        result
    }

    pub fn reset(&mut self) {
        self.is_started = false;
    }

    pub fn render_to(&self, camera: Arc<RwLock<Camera>>, buffer: &mut Vec<Vector3d>) {
        let width = camera.read().unwrap().image().width;
        let height = camera.read().unwrap().image().height;

        self.new_dispatcher_thread(
            camera,
            ((width * height) / self.thread_number / 8) as usize,
            5,
        );

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
            for (x, y, color) in results {
                buffer[(y * width + x) as usize] = color;
            }
        }

        // println!("Rendering finished");
        // dispather.join().unwrap();
    }

    pub fn render_step_to(&mut self, camera: Arc<RwLock<Camera>>, buffer: &mut Vec<Vector3d>) {
        let width = camera.read().unwrap().image().width;
        let height = camera.read().unwrap().image().height;

        self.new_dispatcher_thread(
            camera,
            ((width * height) / self.thread_number / 8) as usize,
            1,
        );

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
                for (x, y, color) in results {
                    let current_color = &buffer[(y * width + x) as usize];
                    buffer[(y * width + x) as usize] = &color + current_color;
                }
            } else {
                for (x, y, color) in results {
                    buffer[(y * width + x) as usize] = color;
                }
            }
        }

        if self.is_started {
            self.is_started = true;
        }
    }

    fn new_dispatcher_thread(
        &self,
        camera: Arc<RwLock<Camera>>,
        chunk_size: usize,
        samples_number: u32,
    ) -> JoinHandle<()> {
        let input_sender = self.input_sender.clone();
        let thread_number = self.thread_number;

        spawn(move || {
            let rays = MultisamplerRayCaster::new(&*camera.read().unwrap(), samples_number);
            for chunk in &rays.chunks(chunk_size) {
                let chunk_vec = chunk.collect_vec();
                input_sender.lock().unwrap().send(Some(chunk_vec)).unwrap();
            }
            for _ in 0..thread_number {
                input_sender.lock().unwrap().send(None).unwrap();
            }
            // println!("Dispatcher finished");
        })
    }

    fn new_worker_thread(&self, thread_id: u32) -> JoinHandle<()> {
        let input_receiver = self.input_receiver.clone();
        let output_sender = self.output_sender.clone();
        let world = self.world.clone();
        let parking = self.parking.clone();
        let depth = self.depth;

        spawn(move || {
            // let mut wait_time = time::Duration::microseconds(0);
            // let mut process_time = time::Duration::microseconds(0);
            // let mut process_units = 0;
            let (lock, cvar) = &*parking;
            let world = &*world.read().unwrap();
            loop {
                // let start = time::Instant::now();
                let input = match input_receiver.lock().unwrap().recv() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Thread {} is stopping", thread_id);
                        break;
                    } // Err(mpsc::TryRecvError::Empty) => continue,
                      // Err(mpsc::TryRecvError::Disconnected) => {
                      //     println!("Thread {} is stopping", thread_id);
                      //     break;
                      // }
                };
                // let end = time::Instant::now();
                // wait_time += end - start;

                match input {
                    Some(v) => {
                        // let start = time::Instant::now();
                        let result = v
                            .iter()
                            .map(|(u, v, rays)| {
                                let samples_colors = rays.iter().map(|ray| ray_color(world, ray, depth));
                                let ln = samples_colors.len() as f64;
                                (*u, *v, samples_colors.sum::<Vector3d>() / ln)
                            })
                            .collect_vec();
                        // let end = time::Instant::now();
                        // process_time += end - start;

                        // let start = time::Instant::now();
                        output_sender.lock().unwrap().send(Some(result)).unwrap();
                        // let end = time::Instant::now();
                        // wait_time += end - start;
                    }
                    None => {
                        // println!("Sent None");
                        output_sender.lock().unwrap().send(None).unwrap();
                        // println!(
                        //     "Thread {}; Processing time: {}; Wait time: {}; Units: {}",
                        //     thread_id,
                        //     process_time.whole_milliseconds(),
                        //     wait_time.whole_milliseconds(),
                        //     process_units
                        // );
                        // process_time = time::Duration::microseconds(0);
                        // wait_time = time::Duration::microseconds(0);
                        // process_units = 0;

                        let running = lock.lock().unwrap();
                        cvar.wait(running).unwrap();
                    }
                }
            }
        })
    }
}
