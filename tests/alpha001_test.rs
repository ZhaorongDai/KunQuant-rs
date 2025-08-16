use kunquant_rs::{BatchParams, BufferNameMap, Executor, Library, Result, run_graph};
use rand::Rng;
use std::path::Path;

const NUM_STOCKS: usize = 1; // TS memory layout
const NUM_TIME: usize = 100;

fn generate_stock_data() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut rng = rand::thread_rng();
    let size = NUM_STOCKS * NUM_TIME;

    // Generate realistic stock data
    let mut open = Vec::with_capacity(size);
    let mut high = Vec::with_capacity(size);
    let mut low = Vec::with_capacity(size);
    let mut close = Vec::with_capacity(size);
    let mut volume = Vec::with_capacity(size);
    let mut amount = Vec::with_capacity(size);

    for _ in 0..size {
        let base_price = rng.gen_range(10.0..200.0);
        let volatility = rng.gen_range(0.01..0.05);

        let o = base_price * (1.0 + rng.gen_range(-volatility..volatility));
        let h = o * (1.0 + rng.gen_range(0.0..volatility));
        let l = o * (1.0 - rng.gen_range(0.0..volatility));
        let c = rng.gen_range(l..=h);
        let v = rng.gen_range(1000000.0..10000000.0);
        let a = c * v;

        open.push(o);
        high.push(h);
        low.push(l);
        close.push(c);
        volume.push(v);
        amount.push(a);
    }

    (open, high, low, close, volume, amount)
}

#[test]
fn test_alpha001_factor() -> Result<()> {
    // Check if test library exists
    let lib_path = "test_libs/alpha001_lib.so";
    if !Path::new(lib_path).exists() {
        panic!("Alpha001 library not found. Please run 'python generate_test_factor.py' first");
    }

    // Create executor and load library
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("alpha001_test")?;

    // Generate stock data
    let (mut open, mut high, mut low, mut close, mut volume, mut amount) = generate_stock_data();
    let mut alpha001_output = vec![0.0f32; NUM_STOCKS * NUM_TIME];

    // Set up buffers
    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("open", &mut open)?;
    buffers.set_buffer_slice("high", &mut high)?;
    buffers.set_buffer_slice("low", &mut low)?;
    buffers.set_buffer_slice("close", &mut close)?;
    buffers.set_buffer_slice("volume", &mut volume)?;
    buffers.set_buffer_slice("amount", &mut amount)?;
    buffers.set_buffer_slice("alpha001", &mut alpha001_output)?;

    // Run computation
    let params = BatchParams::full_range(NUM_STOCKS, NUM_TIME)?;
    run_graph(&executor, &module, &buffers, &params)?;

    // Basic validation - check that we got some non-zero, finite results
    let mut non_zero_count = 0;
    let mut finite_count = 0;
    let mut nan_count = 0;

    for &value in &alpha001_output {
        if value.is_finite() {
            finite_count += 1;
            if value != 0.0 {
                non_zero_count += 1;
            }
        } else if value.is_nan() {
            nan_count += 1;
        }
    }

    println!("Alpha001 results:");
    println!("  Total values: {}", alpha001_output.len());
    println!("  Finite values: {}", finite_count);
    println!("  Non-zero values: {}", non_zero_count);
    println!("  NaN values: {}", nan_count);

    // Print some sample values
    println!("  Sample values:");
    for i in (0..alpha001_output.len().min(20)).step_by(4) {
        println!("    [{:2}] = {:8.4}", i, alpha001_output[i]);
    }

    // Alpha001 typically produces values in [0, 1] range due to rank operation
    // Some NaN values are expected at the beginning due to windowed operations
    assert!(
        finite_count > alpha001_output.len() / 2,
        "Too many non-finite values: {}/{}",
        alpha001_output.len() - finite_count,
        alpha001_output.len()
    );

    // Check that finite values are in reasonable range (rank should be [0, 1])
    for &value in &alpha001_output {
        if value.is_finite() {
            assert!(
                value >= 0.0 && value <= 1.0,
                "Alpha001 value out of expected range [0, 1]: {}",
                value
            );
        }
    }

    println!("✓ Alpha001 factor test passed!");
    Ok(())
}

#[test]
fn test_alpha001_partial_range() -> Result<()> {
    let lib_path = "test_libs/alpha001_lib.so";
    if !Path::new(lib_path).exists() {
        panic!("Alpha001 library not found. Please run 'python generate_test_factor.py' first");
    }

    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("alpha001_test")?;

    // Generate stock data
    let (mut open, mut high, mut low, mut close, mut volume, mut amount) = generate_stock_data();

    // Test partial range computation (skip first 30 time points, compute 40 points)
    let start_time = 30;
    let compute_length = 40;
    let mut alpha001_output = vec![0.0f32; NUM_STOCKS * compute_length];

    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("open", &mut open)?;
    buffers.set_buffer_slice("high", &mut high)?;
    buffers.set_buffer_slice("low", &mut low)?;
    buffers.set_buffer_slice("close", &mut close)?;
    buffers.set_buffer_slice("volume", &mut volume)?;
    buffers.set_buffer_slice("amount", &mut amount)?;
    buffers.set_buffer_slice("alpha001", &mut alpha001_output)?;

    // Run partial computation
    let params = BatchParams::new(NUM_STOCKS, NUM_TIME, start_time, compute_length)?;
    run_graph(&executor, &module, &buffers, &params)?;

    // Validate results
    let finite_count = alpha001_output.iter().filter(|&&x| x.is_finite()).count();
    println!(
        "Partial range Alpha001 results: {}/{} finite values",
        finite_count,
        alpha001_output.len()
    );

    assert!(
        finite_count > 0,
        "No finite values in partial range computation"
    );

    println!("✓ Alpha001 partial range test passed!");
    Ok(())
}
