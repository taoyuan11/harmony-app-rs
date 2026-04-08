use pollster::block_on;
use winit_core::application::ApplicationHandler;
use winit_core::event::{StartCause, WindowEvent};
use winit_core::event_loop::ActiveEventLoop as CoreActiveEventLoop;
use winit_core::window::{Window as CoreWindow, WindowAttributes, WindowId};
use winit_ohos::{Window as OhosWindow, export_ohos_winit_app};

#[derive(Default)]
struct SmokeApp {
    window: Option<Box<dyn CoreWindow>>
}

impl SmokeApp {
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn ensure_window(
        &mut self,
        event_loop: &dyn CoreActiveEventLoop,
    ) -> Result<&dyn CoreWindow, String> {
        if self.window.is_none() {
            let window = event_loop
                .create_window(WindowAttributes::default().with_title("winit-ohos triangle"))
                .map_err(|err| format!("create_window failed: {err}"))?;
            self.window = Some(window);
        }

        Ok(self.window.as_deref().expect("window was just created"))
    }

}

impl ApplicationHandler for SmokeApp {
    fn new_events(&mut self, _event_loop: &dyn CoreActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            eprintln!("winit-smoke booting renderer");
        }
    }

    fn resumed(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke surface resumed");
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn CoreActiveEventLoop) {

    }

    fn window_event(
        &mut self,
        event_loop: &dyn CoreActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::SurfaceResized(size) => {
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.request_redraw();
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                eprintln!("winit-smoke window closing");
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke surface suspended");
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke renderer released");
    }

    fn memory_warning(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke memory warning");
    }
}

export_ohos_winit_app!(SmokeApp::default);
