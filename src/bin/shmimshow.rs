use anyhow::Result;
use clap::Parser;
use shmimshow::{App, Normalisable, Rect};
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser)]
struct Args {
    /// shared memory name /dev/shm/<name>.im.shm
    name: String,
    #[arg(long)]
    /// minimum value to scale colour axis
    cmin: Option<f32>,
    #[arg(long)]
    /// maximum value to scale colour axis
    cmax: Option<f32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // in order to be able to determine type at runtime, I think the application
    // will have to run inside a match statement based on the opened-type of the
    // shm image. Hideous, and something to rethink in future risio APIs, but it
    // might be unavoidable within the strict typing of rust, if we want to skip
    // copying the data.

    if let Ok(()) = run_app::<u8>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<u16>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<u32>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<u64>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<i8>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<i16>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<i32>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<i64>(&args) {
        return Ok(());
    }
    if let Ok(()) = run_app::<f32>(&args) {
        return Ok(());
    }
    match run_app::<f64>(&args) {
        Ok(()) => Ok(()),
        Err(risio::error::Error::MismatchDataType { found, .. }) => {
            Err(anyhow::anyhow!("unsupported data type: {:?}", found))
        }
        Err(e) => Err(e.into()),
    }
}

fn run_app<T: Normalisable>(args: &Args) -> Result<(), risio::error::Error> {
    let mut rect = Rect::<T>::new(&args.name)?;
    rect.cmin(args.cmin).cmax(args.cmax);
    let mut app = App::new(rect);
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
    Ok(())
}
