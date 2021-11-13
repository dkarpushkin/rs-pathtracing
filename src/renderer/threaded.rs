#![allow(dead_code)]

use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
    thread::{spawn, JoinHandle},
};

use itertools::Itertools;

use crate::{
    algebra::Vector3d,
    camera::{Camera, MultisamplerRayCaster},
    world::{Ray, World},
};

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
    world: Arc<RwLock<World>>,
}

impl ThreadPoolRenderer {
    pub fn new(world: Arc<RwLock<World>>, thread_number: u32, depth: u32) -> ThreadPoolRenderer {
        let result = ThreadPoolRenderer {
            thread_number,
            depth,
            world,
        };

        result
    }

    pub fn render_to(&self, camera: Arc<RwLock<Camera>>, buffer: &mut [u8]) {
        let (input_sender, input_receiver) = channel::<InputDataVec>();
        let (output_sender, output_receiver) = channel::<OutputDataVecOption>();
        let shared_input_receiver = Arc::new(Mutex::new(input_receiver));
        let shared_output_sender = Arc::new(Mutex::new(output_sender));

        let width = camera.read().unwrap().image().width;
        let height = camera.read().unwrap().image().height;

        for i in 0..self.thread_number {
            self.new_worker_thread(
                i,
                shared_input_receiver.clone(),
                shared_output_sender.clone(),
            );
        }

        let dispather = self.new_dispatcher_thread(
            camera,
            ((width * height) / self.thread_number / 8) as usize,
            input_sender,
        );

        let mut finished = 0;
        for results in output_receiver {
            let results = match results {
                Some(v) => v,
                None => {
                    finished += 1;
                    if finished == self.thread_number {
                        // println!("All finished");
                        break;
                    }
                    // println!("{} finished", finished);
                    continue;
                }
            };

            // println!("Received {}", results.len());
            for (x, y, color) in results {
                let index = ((y * width + x) * 4) as usize;
                buffer[index + 0] = (color.x * 255.0) as u8;
                buffer[index + 1] = (color.y * 255.0) as u8;
                buffer[index + 2] = (color.z * 255.0) as u8;
                buffer[index + 3] = 255;
            }
        }

        // println!("Rendering finished");
        dispather.join().unwrap();
    }

    fn new_dispatcher_thread(
        &self,
        camera: Arc<RwLock<Camera>>,
        chunk_size: usize,
        input_sender: Sender<InputDataVec>,
    ) -> JoinHandle<()> {
        let input_sender = input_sender.clone();

        spawn(move || {
            let rays = MultisamplerRayCaster::new(&*camera.read().unwrap(), 5);
            for chunk in &rays.chunks(chunk_size) {
                let chunk_vec = chunk.collect_vec();
                input_sender.send(chunk_vec).unwrap();
            }
            println!("Dispatcher finished");
            drop(input_sender);
        })
    }

    fn new_worker_thread(
        &self,
        _thread_id: u32,
        input_receiver: Arc<Mutex<Receiver<InputDataVec>>>,
        output_sender: Arc<Mutex<Sender<OutputDataVecOption>>>,
    ) -> JoinHandle<()> {
        let world = self.world.clone();
        let depth = self.depth;

        spawn(move || {
            // let mut wait_time = time::Duration::microseconds(0);
            // let mut process_time = time::Duration::microseconds(0);
            // let mut process_units = 0;

            loop {
                // let start = time::Instant::now();
                let input = match input_receiver.lock().unwrap().recv() {
                    Ok(v) => v,
                    Err(_) => {
                        // println!("Thread {} is stopping", thread_id);
                        output_sender.lock().unwrap().send(None).unwrap();
                        break;
                    }
                };
                // let end = time::Instant::now();
                // wait_time += end - start;

                // let start = time::Instant::now();
                let world = &*world.read().unwrap();
                // let end = time::Instant::now();
                // wait_time += end - start;

                // let start = time::Instant::now();
                let result = input
                    .iter()
                    .map(|(u, v, rays)| {
                        let colors = rays.iter().map(|ray| ray_color(world, ray, depth));
                        let ln = colors.len() as f64;
                        (*u, *v, colors.sum::<Vector3d>() / ln)
                    })
                    .collect_vec();
                // let end = time::Instant::now();
                // process_time += end - start;

                // let start = time::Instant::now();
                output_sender.lock().unwrap().send(Some(result)).unwrap();
                // let end = time::Instant::now();
                // wait_time += end - start;
            }

            // println!(
            //     "Thread {}; Waiting time: {}; Processing time: {}; Units: {}",
            //     thread_id,
            //     wait_time.whole_milliseconds(),
            //     process_time.whole_milliseconds(),
            //     process_units
            // );
        })
    }
}
