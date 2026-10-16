use sketch_core::RenderScene;
use sketch_renderer::{Compositor, RenderRequest, RenderStats};
use skia_safe::{AlphaType, ColorSpace, ColorType, ImageInfo, Surface, surfaces};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};
struct Lane {
    renderer: Compositor,
    surface: Surface,
    cached: bool,
}
impl Lane {
    fn new(cached: bool) -> Self {
        let info = ImageInfo::new(
            (1024, 768),
            ColorType::RGBA8888,
            AlphaType::Premul,
            ColorSpace::new_srgb(),
        );
        Self {
            renderer: Compositor::default(),
            surface: surfaces::raster(&info, None, None).unwrap(),
            cached,
        }
    }
    fn draw(&mut self, scene: &RenderScene, request: &RenderRequest) -> (u64, RenderStats) {
        let start = Instant::now();
        let stats = self
            .renderer
            .render(&mut self.surface, scene, request, None, self.cached)
            .unwrap();
        (start.elapsed().as_nanos() as u64, stats)
    }
    fn pixels(&mut self) -> Vec<u8> {
        let info = self.surface.image_info();
        let mut pixels = vec![0; 1024 * 768 * 4];
        assert!(
            self.surface
                .read_pixels(&info, &mut pixels, 1024 * 4, (0, 0))
        );
        pixels
    }
}
pub struct Measurement {
    lanes: [Lane; 2],
    output: File,
    path: PathBuf,
    pair: usize,
}
impl Measurement {
    pub fn new(path: &Path) -> io::Result<Self> {
        let mut output = File::options().write(true).create_new(true).open(path)?;
        writeln!(
            output,
            "phase,pair,position,mode,input_dispatch_ns,shared_scene_prepare_ns,render_ns,path_hits,path_misses,path_bytes,background_bytes,background_hit"
        )?;
        Ok(Self {
            lanes: [Lane::new(true), Lane::new(false)],
            output,
            path: path.into(),
            pair: 0,
        })
    }
    pub fn clear_renderers(&mut self) {
        for lane in &mut self.lanes {
            lane.renderer.reset().unwrap();
        }
    }
    pub fn frame(
        &mut self,
        phase: &str,
        scene: &RenderScene,
        request: &RenderRequest,
        scene_ns: u64,
        input_ns: u64,
    ) -> io::Result<()> {
        let order = if self.pair.is_multiple_of(2) {
            [0, 1]
        } else {
            [1, 0]
        };
        for (position, index) in order.into_iter().enumerate() {
            let lane = &mut self.lanes[index];
            let (duration, stats) = lane.draw(scene, request);
            let mode = if lane.cached { "cached" } else { "reference" };
            writeln!(
                self.output,
                "{phase},{},{position},{mode},{input_ns},{scene_ns},{duration},{},{},{},{},{}",
                self.pair,
                stats.path_hits,
                stats.path_misses,
                stats.path_bytes,
                stats.background_bytes,
                stats.background_hit
            )?;
        }
        let cached = self.lanes[0].pixels();
        let reference = self.lanes[1].pixels();
        if cached != reference {
            let directory = self.path.with_extension(format!("failure-{}", self.pair));
            std::fs::create_dir(&directory)?;
            std::fs::write(directory.join("cached.rgba"), &cached)?;
            std::fs::write(directory.join("reference.rgba"), &reference)?;
            let diff = cached
                .iter()
                .zip(&reference)
                .map(|(a, b)| a.abs_diff(*b))
                .collect::<Vec<_>>();
            std::fs::write(directory.join("difference.rgba"), diff)?;
            self.output.flush()?;
            return Err(io::Error::other(format!(
                "exact RGBA mismatch: {}",
                directory.display()
            )));
        }
        self.pair += 1;
        Ok(())
    }
    pub fn finish(mut self) -> io::Result<()> {
        self.output.flush()
    }
}
