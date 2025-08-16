use kunquant_rs::{Executor, Library, Result, StreamContext};
use rand::Rng;
use std::path::Path;

const NUM_STOCKS: usize = 8;

fn generate_tick_data() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut rng = rand::thread_rng();
    let mut close = Vec::with_capacity(NUM_STOCKS);
    let mut open = Vec::with_capacity(NUM_STOCKS);
    let mut high = Vec::with_capacity(NUM_STOCKS);
    let mut low = Vec::with_capacity(NUM_STOCKS);
    
    for _ in 0..NUM_STOCKS {
        let base_price = rng.gen_range(50.0..150.0);
        let o = base_price * (1.0 + rng.gen_range(-0.02..0.02));
        let h = o * (1.0 + rng.gen_range(0.0..0.03));
        let l = o * (1.0 - rng.gen_range(0.0..0.03));
        let c = rng.gen_range(l..=h);
        
        open.push(o);
        high.push(h);
        low.push(l);
        close.push(c);
    }
    
    (close, open, high, low)
}

fn main() -> Result<()> {
    println!("KunQuant-rs Streaming Example");
    println!("=============================");

    // Check if streaming library exists
    let lib_path = "test_libs/simple_stream_lib.so";
    if !Path::new(lib_path).exists() {
        eprintln!("Error: Streaming library not found at {}", lib_path);
        eprintln!("Please run 'python generate_test_factor.py' first");
        return Ok(());
    }

    // Create executor and load library
    println!("1. Setting up streaming context...");
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_stream_test")?;
    
    let mut stream = StreamContext::new(&executor, &module, NUM_STOCKS)?;
    println!("   ✓ Streaming context created for {} stocks", NUM_STOCKS);

    // Simulate real-time data processing
    println!("2. Simulating real-time tick data processing...");
    println!("   Factor formula: (close - open) / (high - low + 0.001)");
    println!();

    for tick in 0..10 {
        // Generate new tick data
        let (close, open, high, low) = generate_tick_data();
        
        println!("Tick {}: Processing new market data", tick + 1);
        
        // Show sample input data
        println!("  Sample prices (first 3 stocks):");
        for i in 0..3.min(NUM_STOCKS) {
            println!("    Stock {}: O={:.2} H={:.2} L={:.2} C={:.2}", 
                     i, open[i], high[i], low[i], close[i]);
        }
        
        // Push new data to the stream
        stream.push_data("close", &close)?;
        stream.push_data("open", &open)?;
        stream.push_data("high", &high)?;
        stream.push_data("low", &low)?;
        
        // Run computation
        stream.run()?;
        
        // Get results
        let factor_values = stream.get_current_buffer("simple_stream")?;
        
        // Display results
        println!("  Factor values:");
        for i in 0..NUM_STOCKS {
            let expected = (close[i] - open[i]) / (high[i] - low[i] + 0.001);
            let actual = factor_values[i];
            println!("    Stock {}: {:.6} (expected: {:.6})", i, actual, expected);
            
            // Verify correctness
            let diff = (expected - actual).abs();
            if diff > 1e-5 {
                eprintln!("    ⚠️  Mismatch detected! Diff: {:.8}", diff);
            }
        }
        
        println!();
        
        // Simulate delay between ticks
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("3. Streaming example completed successfully!");
    println!();
    println!("Key features demonstrated:");
    println!("  ✓ Real-time factor computation");
    println!("  ✓ Efficient buffer management");
    println!("  ✓ Low-latency processing");
    println!("  ✓ Memory-safe streaming operations");
    println!();
    println!("In a real application, you would:");
    println!("  - Connect to market data feeds");
    println!("  - Process thousands of stocks simultaneously");
    println!("  - Use more complex factors (Alpha101, etc.)");
    println!("  - Implement risk management and portfolio optimization");

    Ok(())
}
