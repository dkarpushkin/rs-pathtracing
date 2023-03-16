use rayon::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread::{scope, spawn},
};

use itertools::{Chunks, IntoChunks, Itertools};

use crate::{
    algebra::Vector3d,
    camera::{
        ray_caster::{ImageParams, MultisamplerRayCaster},
        Camera,
    },
    world::Scene,
};

use super::{ray_color, Renderer, trace_pixel_samples_group};

pub struct ThreadPoolRenderer {
    depth: u32,

    img_params: ImageParams,
    threads_num: u32,

    world: Arc<RwLock<Scene>>,

    chunk_size: usize,
    rays: Option<MultisamplerRayCaster>,
    thread_pool: rayon::ThreadPool,
}

impl ThreadPoolRenderer {
    pub fn new(world: Arc<RwLock<Scene>>, threads_num: u32, depth: u32) -> ThreadPoolRenderer {
        let result = Self {
            depth,
            img_params: ImageParams {
                width: 0,
                height: 0,
            },
            threads_num,
            world,
            chunk_size: 0,
            rays: None,
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(threads_num as usize)
                .build()
                .unwrap(),
        };

        result
    }
}

impl Renderer for ThreadPoolRenderer {
    fn start_rendering(
        &mut self,
        camera: Arc<RwLock<Camera>>,
        img_params: &ImageParams,
        samples_number: u32,
    ) {
        self.img_params = img_params.clone();

        self.chunk_size =
            ((self.img_params.width * self.img_params.height) / self.threads_num / 32) as usize;
        let rays =
            MultisamplerRayCaster::new(&*camera.read().unwrap(), &self.img_params, samples_number);
        self.rays = Some(rays);
    }

    fn render_step(&mut self, buffer: &mut Vec<Vector3d>) -> bool {
        let rays = self.rays.as_mut().unwrap();
        let slices = buffer.chunks_mut(self.chunk_size);

        // =============================================================================================================
        
        let rays_vec = slices.zip(
            rays.chunks(self.chunk_size)
                .into_iter()
                .map(|c| c.map(|(_, _, v)| (0, v)).collect_vec())
                .collect_vec(),
        );

        let world = &*self.world.read().unwrap();
        rays_vec.par_bridge().for_each(|(buffer_chunk, ray_chunk)| {

            let b = trace_pixel_samples_group(&ray_chunk, world, self.depth);

            for (x, v) in b.iter().zip(buffer_chunk) {
                *v = x.1;
            }

            // for (v, (_, _, rays)) in buffer_chunk.iter_mut().zip(ray_chunk) {
            //     let samples_colors = rays.iter().map(|ray| ray_color(world, ray, self.depth));
            //     let ln = samples_colors.len() as f64;
            //     *v = samples_colors.sum::<Vector3d>() / ln;
            // }
        });

        // =============================================================================================================

        // let rays_vec = rays
        //     .chunks(self.chunk_size)
        //     .into_iter()
        //     .map(|a| a.collect_vec())
        //     .collect_vec();

        // buffer
        //     .par_chunks_mut(self.chunk_size)
        //     .zip(rays_vec)
        //     .for_each(|(buffer_chunk, ray_chunk)| {
        //         let world = &*self.world.read().unwrap();

        //         for (v, (_, _, rays)) in buffer_chunk.iter_mut().zip(ray_chunk) {
        //             let samples_colors = rays.iter().map(|ray| ray_color(world, ray, self.depth));
        //             let ln = samples_colors.len() as f64;
        //             *v = samples_colors.sum::<Vector3d>() / ln;
        //         }
        //     });

        // =============================================================================================================

        // scope(|s| {
        //     let rays = self.rays.as_mut().unwrap();

        //     for (buffer_chunk, ray_chunk) in slices.zip(rays.chunks(self.chunk_size).into_iter()) {
        //         let chunk_vec = ray_chunk.collect_vec();
        //         s.spawn(|| {
        //             let world = &*self.world.read().unwrap();
        //             for (v, (_, _, rays)) in buffer_chunk.iter_mut().zip(chunk_vec) {
        //                 let samples_colors = rays.iter().map(|ray| ray_color(world, ray, self.depth));
        //                 let ln = samples_colors.len() as f64;
        //                 *v = samples_colors.sum::<Vector3d>() / ln;
        //             }
        //         });
        //     }
        // });

        // =============================================================================================================
        
        // let rays = self.rays.as_mut().unwrap();
        // self.thread_pool.scope(|s| {
            
        //     for (buffer_chunk, ray_chunk) in slices.zip(rays.chunks(self.chunk_size).into_iter()) {
        //         let chunk_vec = ray_chunk.collect_vec();
        //         let world = &*self.world.read().unwrap();
        //         s.spawn(|s| {
        //             for (v, (_, _, rays)) in buffer_chunk.iter_mut().zip(chunk_vec) {
        //                                 // let samples_colors = rays.iter().map(|ray| ray_color(world, ray, self.depth));
        //                                 // let ln = samples_colors.len() as f64;
        //                                 // *v = samples_colors.sum::<Vector3d>() / ln;
        //             }
        //         })
        //     }
        // });

        return true;
    }

    fn stop_rendering(&mut self) {
        self.rays = None;
    }
}
