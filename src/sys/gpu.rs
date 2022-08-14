#![allow(dead_code)]

use num_derive::FromPrimitive;
use std::sync::atomic::{AtomicIsize, Ordering};

use cfg_if::cfg_if;
use lazy_static::lazy_static;

pub fn init() {
    env_logger::init();
}

#[derive(FromPrimitive, PartialEq, Debug)]
enum RenderApi {
    Vulkan,
    Dx12,
    Dx11,
    Dx10,
    Dx9,
    Metal,
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        const RENDER_API_DEFAULT: RenderApi = RenderApi::Dx12;
    } else {
        const RENDER_API_DEFAULT: RenderApi = RenderApi::Vulkan;
    }
}

lazy_static! {
    static ref RENDER_API: AtomicIsize =
        AtomicIsize::new(RENDER_API_DEFAULT as isize);
}

macro_rules! render_api {
    () => {{
        let r: RenderApi =
            num::FromPrimitive::from_isize(RENDER_API.load(Ordering::SeqCst))
                .unwrap();
        r
    }};
}

macro_rules! render_api_implemented_by_wgpu {
    () => {
        if render_api!() == RenderApi::Vulkan
            || render_api!() == RenderApi::Dx12
            || render_api!() == RenderApi::Metal
        {
            true
        } else {
            false
        }
    };
}

#[derive(FromPrimitive, PartialEq, Debug)]
enum WindowCreator {
    Winit,
}

const WINDOW_CREATOR_DEFAULT: WindowCreator = WindowCreator::Winit;

lazy_static! {
    static ref WINDOW_CREATOR: AtomicIsize =
        AtomicIsize::new(WINDOW_CREATOR_DEFAULT as isize);
}

macro_rules! window_creator {
    () => {{
        let w: WindowCreator = num::FromPrimitive::from_isize(
            WINDOW_CREATOR.load(Ordering::SeqCst),
        )
        .unwrap();
        w
    }};
}

pub struct Window {
    winit_window: winit::window::Window,
}

impl Window {
    fn new() -> Self {
        if window_creator!() == WindowCreator::Winit {
            todo!();
        } else {
            todo!("gpu::Window not yet implemented for {:?}.", render_api!());
        }
    }
}

pub struct Instance {
    wgpu_instance: Option<wgpu::Instance>,
}

impl Instance {
    pub fn new() -> Self {
        if render_api_implemented_by_wgpu!() {
            Instance {
                wgpu_instance: Some(wgpu::Instance::new(
                    wgpu::Backends::VULKAN
                        | wgpu::Backends::DX12
                        | wgpu::Backends::METAL,
                )),
            }
        } else {
            todo!("gpu::Instance not yet implemented for {:?}.", render_api!());
        }
    }
}

pub struct Surface {
    wgpu_surface: Option<wgpu::Surface>,
}

impl Surface {
    pub fn new(instance: &Instance, window: &Window) -> Self {
        if render_api_implemented_by_wgpu!() {
            if window_creator!() == WindowCreator::Winit {
                Surface {
                    wgpu_surface: Some(unsafe {
                        instance
                            .wgpu_instance
                            .as_ref()
                            .unwrap()
                            .create_surface(&window.winit_window)
                    }),
                }
            } else {
                todo!(
                    "gpu::Instance not yet implemented for {:?}.",
                    window_creator!()
                );
            }
        } else {
            todo!("gpu::Instance not yet implemented for {:?}.", render_api!());
        }
    }
}

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum DeviceType {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

pub struct AdapterInfo {
    pub name: String,
    pub device_type: DeviceType,
}

impl AdapterInfo {
    pub fn new(name: String, device_type: DeviceType) -> Self {
        Self { name, device_type }
    }
}

pub struct Adapter {
    wgpu_adapter: Option<wgpu::Adapter>,
}

impl Adapter {
    pub async fn new(instance: &Instance, surface: Option<&Surface>) -> Self {
        if render_api_implemented_by_wgpu!() {
            if surface.is_some() {
                Adapter {
                    wgpu_adapter: Some(
                        instance
                            .wgpu_instance
                            .as_ref()
                            .unwrap()
                            .request_adapter(&wgpu::RequestAdapterOptions {
                                power_preference:
                                    wgpu::PowerPreference::default(),
                                compatible_surface: Some(
                                    surface
                                        .unwrap()
                                        .wgpu_surface
                                        .as_ref()
                                        .unwrap(),
                                ),
                                force_fallback_adapter: false,
                            })
                            .await,
                    )
                    .unwrap(),
                }
            } else {
                Adapter {
                    wgpu_adapter: Some(
                        instance
                            .wgpu_instance
                            .as_ref()
                            .unwrap()
                            .request_adapter(&wgpu::RequestAdapterOptions {
                                power_preference:
                                    wgpu::PowerPreference::default(),
                                compatible_surface: None,
                                force_fallback_adapter: false,
                            })
                            .await,
                    )
                    .unwrap(),
                }
            }
        } else {
            todo!("gpu::Instance not yet implemented for {:?}.", render_api!());
        }
    }

    pub fn get_info(&self) -> AdapterInfo {
        if render_api_implemented_by_wgpu!() {
            let info = self.wgpu_adapter.as_ref().unwrap().get_info();
            AdapterInfo::new(
                info.name,
                num::FromPrimitive::from_isize(info.device_type as isize)
                    .unwrap(),
            )
        } else {
            todo!("gpu::Instance not yet implemented for {:?}.", render_api!());
        }
    }
}

struct Device {
    wgpu_device: Option<wgpu::Device>,
}

impl Device {
    pub async fn new(adapter: &Adapter) -> Self {
        if render_api_implemented_by_wgpu!() {
            let (device, _) = adapter
                .wgpu_adapter
                .as_ref()
                .unwrap()
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None, // Trace path
                )
                .await
                .unwrap();

            Device {
                wgpu_device: Some(device),
            }
        } else {
            todo!("gpu::Device not yet implemented for {:?}.", render_api!());
        }
    }
}

struct Queue {
    wgpu_queue: Option<wgpu::Queue>,
}

impl Queue {
    pub async fn new(adapter: &Adapter) -> Self {
        if render_api_implemented_by_wgpu!() {
            let (_, queue) = adapter
                .wgpu_adapter
                .as_ref()
                .unwrap()
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None, // Trace path
                )
                .await
                .unwrap();

            Queue {
                wgpu_queue: Some(queue),
            }
        } else {
            todo!("gpu::Device not yet implemented for {:?}.", render_api!());
        }
    }
}

struct Config {
    wgpu_config: Option<wgpu::SurfaceConfiguration>,
}

impl Config {
    pub async fn new(
        surface: &Surface,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) -> Self {
        if render_api_implemented_by_wgpu!() {
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface
                    .wgpu_surface
                    .as_ref()
                    .unwrap()
                    .get_supported_formats(
                        adapter.wgpu_adapter.as_ref().unwrap(),
                    )[0],
                width,
                height,
                present_mode: wgpu::PresentMode::AutoNoVsync,
            };

            Self {
                wgpu_config: Some(config),
            }
        } else {
            todo!("gpu::Config not yet implemented for {:?}.", render_api!());
        }
    }
}

struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    //config: Config,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        if render_api_implemented_by_wgpu!() {
            let instance = Instance::new();
            let surface = Surface::new(&instance, window);
            let adapter = Adapter::new(&instance, Some(&surface)).await;
            let device = Device::new(&adapter).await;
            let queue = Queue::new(&adapter).await;
            //let config = Config::new(&surface, &adapter, width, height).await;
            //surface.wgpu_surface.unwrap().configure(&device.wgpu_device.unwrap(), &config.wgpu_config.unwrap());

            Self {
                surface,
                device,
                queue,
                //config
            }
        } else {
            todo!("gpu::State not yet implemented for {:?}.", render_api!());
        }
    }
}
