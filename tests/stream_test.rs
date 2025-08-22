use kunquant_rs::{Executor, Library, Result, StreamContext};
use std::path::Path;

const NUM_STOCKS_ALIGNED: usize = 64;
const NUM_STOCKS_UNALIGNED: usize = 63;

#[test]
fn test_stream_creation_only() -> Result<()> {
    // Check if streaming test library exists
    let lib_path = "test_libs/simple_stream_lib.so";
    if !Path::new(lib_path).exists() {
        panic!(
            "Streaming test library not found. Please run 'python generate_test_factor.py' first"
        );
    }

    // Create executor and load library
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_stream_test")?;

    // Just try to create stream context
    let _stream = StreamContext::new(&executor, &module, NUM_STOCKS_ALIGNED)?;

    println!("✓ Stream context created successfully!");
    Ok(())
}

#[test]
fn test_stream_buffer_handles() -> Result<()> {
    let lib_path = "test_libs/simple_stream_lib.so";
    if !Path::new(lib_path).exists() {
        panic!(
            "Streaming test library not found. Please run 'python generate_test_factor.py' first"
        );
    }

    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_stream_test")?;

    let mut stream = StreamContext::new(&executor, &module, NUM_STOCKS_ALIGNED)?;

    // Try to get buffer handles (only the ones actually used by the factor)
    let input_names = ["close", "open", "high", "low"];
    let output_names = ["simple_stream"];

    println!("Testing input buffer handles:");
    for name in &input_names {
        match stream.get_buffer_handle(name) {
            Ok(handle) => println!("  {} -> handle {}", name, handle),
            Err(e) => println!("  {} -> ERROR: {:?}", name, e),
        }
    }

    println!("Testing output buffer handles:");
    for name in &output_names {
        match stream.get_buffer_handle(name) {
            Ok(handle) => println!("  {} -> handle {}", name, handle),
            Err(e) => println!("  {} -> ERROR: {:?}", name, e),
        }
    }

    println!("✓ Buffer handle test completed!");
    Ok(())
}

#[test]
fn test_stream_single_step() -> Result<()> {
    let lib_path = "test_libs/simple_stream_lib.so";
    if !Path::new(lib_path).exists() {
        panic!(
            "Streaming test library not found. Please run 'python generate_test_factor.py' first"
        );
    }

    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_stream_test")?;

    let mut stream = StreamContext::new(&executor, &module, NUM_STOCKS_ALIGNED)?;

    // Create simple test data
    let mut close_data = Vec::new();
    let mut open_data = Vec::new();
    let mut high_data = Vec::new();
    let mut low_data = Vec::new();
    for _ in 0..NUM_STOCKS_ALIGNED {
        let rand_datas = vec![
            rand::random::<f32>() * 10.0,
            rand::random::<f32>() * 10.0,
            rand::random::<f32>() * 10.0,
            rand::random::<f32>() * 10.0,
        ];
        close_data.push(100.0 + rand_datas[0] as f32);
        open_data.push(99.0 + rand_datas[1] as f32);
        high_data.push(101.0 + rand_datas[2] as f32);
        low_data.push(98.0 + rand_datas[3] as f32);
    }
    println!("Pushing input data...");

    // Push data step by step with error checking (only the required inputs)
    println!("  Pushing close data...");
    stream.push_data("close", &close_data)?;

    println!("  Pushing open data...");
    stream.push_data("open", &open_data)?;

    println!("  Pushing high data...");
    stream.push_data("high", &high_data)?;

    println!("  Pushing low data...");
    stream.push_data("low", &low_data)?;

    println!("Running computation...");
    stream.run()?;

    println!("Getting output data...");
    let output_data = stream.get_current_buffer("simple_stream")?;

    println!("{}", output_data.len());
    println!("Results:");
    for i in 0..NUM_STOCKS_ALIGNED {
        let expected = (close_data[i] - open_data[i]) / (high_data[i] - low_data[i] + 0.001);
        let actual = output_data[i];
        println!(
            "  Stock {}: expected {:.6}, actual {:.6}",
            i, expected, actual
        );
    }

    println!("✓ Single step stream test completed!");

    Ok(())
}

#[test]
fn test_stream_single_step_unaligned() -> Result<()> {
    let lib_path = "test_libs/simple_stream_lib.so";
    if !Path::new(lib_path).exists() {
        panic!(
            "Streaming test library not found. Please run 'python generate_test_factor.py' first"
        );
    }

    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_stream_test")?;

    let mut stream = StreamContext::new(&executor, &module, NUM_STOCKS_UNALIGNED)?;

    // Create simple test data
    let mut close_data = Vec::new();
    let mut open_data = Vec::new();
    let mut high_data = Vec::new();
    let mut low_data = Vec::new();
    for _ in 0..NUM_STOCKS_UNALIGNED {
        let rand_datas = vec![
            rand::random::<f32>() * 10.0,
            rand::random::<f32>() * 10.0,
            rand::random::<f32>() * 10.0,
            rand::random::<f32>() * 10.0,
        ];
        close_data.push(100.0 + rand_datas[0] as f32);
        open_data.push(99.0 + rand_datas[1] as f32);
        high_data.push(101.0 + rand_datas[2] as f32);
        low_data.push(98.0 + rand_datas[3] as f32);
    }
    println!("Pushing input data...");

    // Push data step by step with error checking (only the required inputs)
    println!("  Pushing close data...");
    stream.push_data("close", &close_data)?;

    println!("  Pushing open data...");
    stream.push_data("open", &open_data)?;

    println!("  Pushing high data...");
    stream.push_data("high", &high_data)?;

    println!("  Pushing low data...");
    stream.push_data("low", &low_data)?;

    println!("Running computation...");
    stream.run()?;

    println!("Getting output data...");
    let output_data = stream.get_current_buffer("simple_stream")?;

    println!("{}", output_data.len());
    println!("Results:");
    for i in 0..NUM_STOCKS_UNALIGNED {
        let expected = (close_data[i] - open_data[i]) / (high_data[i] - low_data[i] + 0.001);
        let actual = output_data[i];
        println!(
            "  Stock {}: expected {:.6}, actual {:.6}",
            i, expected, actual
        );
    }

    println!("✓ Single step stream test completed!");

    Ok(())
}
