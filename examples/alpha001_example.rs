use kunquant_rs::{
    BatchParams, BufferNameMap, Executor, Library, Result,
    run_graph,
};
use rand::Rng;
use std::path::Path;

const NUM_STOCKS: usize = 8;
const NUM_TIME: usize = 50;

fn generate_realistic_stock_data() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut rng = rand::thread_rng();
    let size = NUM_STOCKS * NUM_TIME;
    
    let mut open = Vec::with_capacity(size);
    let mut high = Vec::with_capacity(size);
    let mut low = Vec::with_capacity(size);
    let mut close = Vec::with_capacity(size);
    let mut volume = Vec::with_capacity(size);
    let mut amount = Vec::with_capacity(size);
    
    // Generate realistic stock data with some correlation
    for stock in 0..NUM_STOCKS {
        let base_price = 50.0 + (stock as f32) * 10.0; // Different base prices
        let mut current_price = base_price;
        
        for _time in 0..NUM_TIME {
            // Random walk with mean reversion
            let return_rate = rng.gen_range(-0.03..0.03);
            current_price *= 1.0 + return_rate;
            
            let volatility = 0.01;
            let o = current_price * (1.0 + rng.gen_range(-volatility..volatility));
            let h = o * (1.0 + rng.gen_range(0.0..volatility * 2.0));
            let l = o * (1.0 - rng.gen_range(0.0..volatility * 2.0));
            let c = rng.gen_range(l..=h);
            let v = rng.gen_range(100000.0..1000000.0);
            let a = c * v;
            
            open.push(o);
            high.push(h);
            low.push(l);
            close.push(c);
            volume.push(v);
            amount.push(a);
            
            current_price = c; // Update for next iteration
        }
    }
    
    (open, high, low, close, volume, amount)
}

fn main() -> Result<()> {
    println!("KunQuant-rs Alpha001 Example");
    println!("============================");

    // Check if Alpha001 library exists
    let lib_path = "test_libs/alpha001_lib.so";
    if !Path::new(lib_path).exists() {
        eprintln!("Error: Alpha001 library not found at {}", lib_path);
        eprintln!("Please run 'python generate_test_factor.py' first");
        return Ok(());
    }

    // Create executor and load library
    println!("1. Creating executor and loading Alpha001 library...");
    let executor = Executor::multi_thread(2)?; // Use 2 threads for better performance
    let library = Library::load(lib_path)?;
    let module = library.get_module("alpha001_test")?;
    println!("   ✓ Alpha001 library loaded successfully");

    // Generate realistic stock data
    println!("2. Generating realistic stock data...");
    let (mut open, mut high, mut low, mut close, mut volume, mut amount) = generate_realistic_stock_data();
    let mut alpha001_output = vec![0.0f32; NUM_STOCKS * NUM_TIME];
    
    println!("   ✓ Generated data for {} stocks over {} time periods", NUM_STOCKS, NUM_TIME);
    
    // Show sample input data
    println!("   Sample close prices (first 5 time points):");
    for t in 0..5.min(NUM_TIME) {
        print!("     t={}: ", t);
        for s in 0..NUM_STOCKS {
            print!("{:6.2} ", close[s * NUM_TIME + t]);
        }
        println!();
    }

    // Set up buffers
    println!("3. Setting up input/output buffers...");
    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("open", &mut open)?;
    buffers.set_buffer_slice("high", &mut high)?;
    buffers.set_buffer_slice("low", &mut low)?;
    buffers.set_buffer_slice("close", &mut close)?;
    buffers.set_buffer_slice("volume", &mut volume)?;
    buffers.set_buffer_slice("amount", &mut amount)?;
    buffers.set_buffer_slice("alpha001", &mut alpha001_output)?;
    println!("   ✓ All buffers configured");

    // Run Alpha001 computation
    println!("4. Running Alpha001 factor computation...");
    let params = BatchParams::full_range(NUM_STOCKS, NUM_TIME)?;
    run_graph(&executor, &module, &buffers, &params)?;
    println!("   ✓ Alpha001 computation completed");

    // Analyze results
    println!("5. Analyzing Alpha001 results:");
    let mut finite_count = 0;
    let mut nan_count = 0;
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    let mut sum = 0.0;
    
    for &value in &alpha001_output {
        if value.is_finite() {
            finite_count += 1;
            min_val = min_val.min(value);
            max_val = max_val.max(value);
            sum += value;
        } else if value.is_nan() {
            nan_count += 1;
        }
    }
    
    let mean = if finite_count > 0 { sum / finite_count as f32 } else { 0.0 };
    
    println!("   Total values: {}", alpha001_output.len());
    println!("   Finite values: {} ({:.1}%)", finite_count, 
             100.0 * finite_count as f32 / alpha001_output.len() as f32);
    println!("   NaN values: {} ({:.1}%)", nan_count,
             100.0 * nan_count as f32 / alpha001_output.len() as f32);
    
    if finite_count > 0 {
        println!("   Value range: [{:.6}, {:.6}]", min_val, max_val);
        println!("   Mean value: {:.6}", mean);
    }

    // Show sample results (last 10 time points to avoid initial NaNs)
    println!("   Sample Alpha001 values (last 5 time points):");
    let start_time = (NUM_TIME - 5).max(0);
    for t in start_time..NUM_TIME {
        print!("     t={}: ", t);
        for s in 0..NUM_STOCKS {
            let idx = s * NUM_TIME + t;
            let val = alpha001_output[idx];
            if val.is_finite() {
                print!("{:6.4} ", val);
            } else {
                print!("  NaN  ");
            }
        }
        println!();
    }

    // Validate that results are in expected range for rank-based factor
    if finite_count > 0 {
        if min_val >= 0.0 && max_val <= 1.0 {
            println!("   ✅ Alpha001 values are in expected range [0, 1]");
        } else {
            println!("   ⚠️  Alpha001 values outside expected range [0, 1]");
        }
    }

    println!("\nAlpha001 example completed successfully!");
    println!("The Alpha001 factor represents cross-sectional rankings of a complex");
    println!("technical indicator involving price momentum and volatility measures.");
    
    Ok(())
}
