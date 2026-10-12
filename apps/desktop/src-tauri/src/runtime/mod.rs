pub mod linux;
pub mod traits;
pub mod windows;

pub use linux::LinuxRuntime;
pub use traits::RuntimePlatform;
pub use windows::WindowsRuntime;

pub fn create_default_runtime() -> Box<dyn RuntimePlatform> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsRuntime::new())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(LinuxRuntime::new())
    }
}
