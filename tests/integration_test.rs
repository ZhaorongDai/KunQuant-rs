use kunquant_rs::{
    BatchParams, BufferNameMap, Executor, Library, Result, StreamContext, run_graph,
};
use rand::Rng;
use std::path::Path;

const NUM_STOCKS: usize = 8;
const NUM_TIME: usize = 100;

fn generate_random_data(size: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen_range(1.0..100.0)).collect()
}

#[test]
fn test_simple_factor_batch() -> Result<()> {
    // Check if test library exists
    let lib_path = "test_libs/simple_test_lib.so";
    if !Path::new(lib_path).exists() {
        panic!("Test library not found. Please run 'python generate_test_factor.py' first");
    }

    // Create executor and load library
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_test")?;

    // Prepare input data
    let mut input_data = generate_random_data(NUM_STOCKS * NUM_TIME);
    let mut output_data = vec![0.0f32; NUM_STOCKS * NUM_TIME];

    // Set up buffers
    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("input", &mut input_data)?;
    buffers.set_buffer_slice("output", &mut output_data)?;

    // Run computation
    let params = BatchParams::full_range(NUM_STOCKS, NUM_TIME)?;
    run_graph(&executor, &module, &buffers, &params)?;

    // Verify results: output should be input * 3
    let tolerance = 1e-5;
    for i in 0..input_data.len() {
        let expected = input_data[i] * 3.0;
        let actual = output_data[i];
        let diff = (expected - actual).abs();

        if diff > tolerance {
            panic!(
                "Mismatch at index {}: expected {}, got {}, diff {}",
                i, expected, actual, diff
            );
        }
    }

    println!("✓ Simple factor batch test passed!");
    Ok(())
}

#[test]
fn test_simple_factor_stream() -> Result<()> {
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

    // Create stream context
    let mut stream = StreamContext::new(&executor, &module, NUM_STOCKS)?;

    // Test streaming computation
    let tolerance = 1e-5;
    for time_step in 0..10 {
        // Generate OHLC data for this time step (only what's needed)
        let mut rng = rand::thread_rng();
        let mut close_data = Vec::with_capacity(NUM_STOCKS);
        let mut open_data = Vec::with_capacity(NUM_STOCKS);
        let mut high_data = Vec::with_capacity(NUM_STOCKS);
        let mut low_data = Vec::with_capacity(NUM_STOCKS);

        for _ in 0..NUM_STOCKS {
            let base_price = rng.gen_range(50.0..100.0);
            let open = base_price * (1.0 + rng.gen_range(-0.01..0.01));
            let high = open * (1.0 + rng.gen_range(0.0..0.02));
            let low = open * (1.0 - rng.gen_range(0.0..0.02));
            let close = rng.gen_range(low..=high);

            close_data.push(close);
            open_data.push(open);
            high_data.push(high);
            low_data.push(low);
        }

        // Push data and run computation (only required inputs)
        stream.push_data("close", &close_data)?;
        stream.push_data("open", &open_data)?;
        stream.push_data("high", &high_data)?;
        stream.push_data("low", &low_data)?;

        stream.run()?;

        // Get output data
        let output_data = stream.get_current_buffer("simple_stream")?;

        // Verify results: output should be (close - open) / (high - low + 0.001)
        for i in 0..NUM_STOCKS {
            let expected = (close_data[i] - open_data[i]) / (high_data[i] - low_data[i] + 0.001);
            let actual = output_data[i];
            let diff = (expected - actual).abs();

            if diff > tolerance {
                panic!(
                    "Stream mismatch at time {} index {}: expected {}, got {}, diff {}",
                    time_step, i, expected, actual, diff
                );
            }
        }
    }

    println!("✓ Simple factor stream test passed!");
    Ok(())
}

#[test]
fn test_multi_thread_executor() -> Result<()> {
    let lib_path = "test_libs/simple_test_lib.so";
    if !Path::new(lib_path).exists() {
        panic!("Test library not found. Please run 'python generate_test_factor.py' first");
    }

    // Create multi-thread executor
    let executor = Executor::multi_thread(4)?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_test")?;

    // Prepare data
    let mut input_data = generate_random_data(NUM_STOCKS * NUM_TIME);
    let mut output_data = vec![0.0f32; NUM_STOCKS * NUM_TIME];

    // Set up buffers
    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("input", &mut input_data)?;
    buffers.set_buffer_slice("output", &mut output_data)?;

    // Run computation
    let params = BatchParams::full_range(NUM_STOCKS, NUM_TIME)?;
    run_graph(&executor, &module, &buffers, &params)?;

    // Verify results
    let tolerance = 1e-5;
    for i in 0..input_data.len() {
        let expected = input_data[i] * 3.0;
        let actual = output_data[i];
        let diff = (expected - actual).abs();

        if diff > tolerance {
            panic!(
                "Multi-thread mismatch at index {}: expected {}, got {}, diff {}",
                i, expected, actual, diff
            );
        }
    }

    println!("✓ Multi-thread executor test passed!");
    Ok(())
}
