use kunquant_rs::{
    BatchParams, BufferNameMap, Executor, Library, Result,
    run_graph,
};
use std::path::Path;

fn main() -> Result<()> {
    println!("KunQuant-rs Simple Example");
    println!("==========================");

    // Check if test library exists
    let lib_path = "test_libs/simple_test_lib.so";
    if !Path::new(lib_path).exists() {
        eprintln!("Error: Test library not found at {}", lib_path);
        eprintln!("Please run 'python generate_test_factor.py' first");
        return Ok(());
    }

    // Create executor and load library
    println!("1. Creating executor and loading library...");
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_test")?;
    println!("   ✓ Library loaded successfully");

    // Prepare input data (8 stocks, 1 time point)
    println!("2. Preparing input data...");
    let mut input_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let mut output_data = vec![0.0f32; 8];
    
    println!("   Input data:  {:?}", input_data);

    // Set up buffers
    println!("3. Setting up buffers...");
    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("input", &mut input_data)?;
    buffers.set_buffer_slice("output", &mut output_data)?;
    println!("   ✓ Buffers configured");

    // Run computation
    println!("4. Running factor computation...");
    let params = BatchParams::full_range(8, 1)?;
    run_graph(&executor, &module, &buffers, &params)?;
    println!("   ✓ Computation completed");

    // Display results
    println!("5. Results:");
    println!("   Output data: {:?}", output_data);
    println!("   Expected:    {:?}", input_data.iter().map(|x| x * 3.0).collect::<Vec<_>>());
    
    // Verify results
    let tolerance = 1e-5;
    let mut all_correct = true;
    for i in 0..input_data.len() {
        let expected = input_data[i] * 3.0;
        let actual = output_data[i];
        let diff = (expected - actual).abs();
        
        if diff > tolerance {
            println!("   ❌ Mismatch at index {}: expected {}, got {}", i, expected, actual);
            all_correct = false;
        }
    }
    
    if all_correct {
        println!("   ✅ All results are correct!");
    }

    println!("\nExample completed successfully!");
    Ok(())
}
