use kunquant_rs::{BatchParams, BufferNameMap, Executor, Library, Result, run_graph};
use ndarray::{Array3, Axis, s};
use rand::prelude::*;
use std::path::Path;

/// 创建随机的3D输入数据 [num_stock, num_time, num_factors]
fn create_random_input_data(num_stock: usize, num_time: usize, num_factors: usize) -> Array3<f32> {
    let mut rng = thread_rng();
    Array3::from_shape_fn((num_stock, num_time, num_factors), |_| {
        rng.gen_range(-10.0..10.0)
    })
}

/// 将ndarray数据转换为KunQuant期望的行主序格式
/// KunQuant期望的数据布局: [time, stock] for each factor
fn ndarray_to_kunquant_format(data: &Array3<f32>) -> Vec<f32> {
    let (num_stock, num_time, num_factors) = data.dim();
    let mut result = Vec::with_capacity(num_stock * num_time * num_factors);

    // 对每个因子，按时间-股票的顺序排列数据
    for factor_idx in 0..num_factors {
        for time_idx in 0..num_time {
            for stock_idx in 0..num_stock {
                result.push(data[[stock_idx, time_idx, factor_idx]]);
            }
        }
    }

    result
}

/// 将KunQuant输出格式转换回ndarray格式
fn kunquant_to_ndarray_format(
    data: &[f32],
    num_stock: usize,
    num_time: usize,
    num_factors: usize,
) -> Array3<f32> {
    let mut result = Array3::zeros((num_stock, num_time, num_factors));

    // 从KunQuant的行主序格式转换回来
    for factor_idx in 0..num_factors {
        for time_idx in 0..num_time {
            for stock_idx in 0..num_stock {
                let flat_idx =
                    factor_idx * (num_time * num_stock) + time_idx * num_stock + stock_idx;
                result[[stock_idx, time_idx, factor_idx]] = data[flat_idx];
            }
        }
    }

    result
}

/// 简单的因子计算函数：计算移动平均
fn compute_moving_average(data: &Array3<f32>, window: usize) -> Array3<f32> {
    let (num_stock, num_time, num_factors) = data.dim();
    let mut result = Array3::zeros((num_stock, num_time, num_factors));

    for stock_idx in 0..num_stock {
        for factor_idx in 0..num_factors {
            for time_idx in 0..num_time {
                let start_idx = if time_idx >= window {
                    time_idx - window + 1
                } else {
                    0
                };
                let end_idx = time_idx + 1;

                let sum: f32 = (start_idx..end_idx)
                    .map(|t| data[[stock_idx, t, factor_idx]])
                    .sum();
                let count = end_idx - start_idx;
                result[[stock_idx, time_idx, factor_idx]] = sum / count as f32;
            }
        }
    }

    result
}

#[test]
fn test_ndarray_basic_operations() {
    println!("🧪 Testing basic ndarray operations...");

    let num_stock = 8; // KunQuant要求股票数量是8的倍数
    let num_time = 100;
    let num_factors = 3;

    // 创建随机输入数据
    let input_data = create_random_input_data(num_stock, num_time, num_factors);
    println!("✅ Created input data with shape: {:?}", input_data.dim());

    // 计算移动平均
    let window = 5;
    let output_data = compute_moving_average(&input_data, window);
    println!("✅ Computed moving average with window size: {}", window);

    // 验证输出形状
    assert_eq!(output_data.dim(), (num_stock, num_time, num_factors));
    println!("✅ Output shape verified: {:?}", output_data.dim());

    // 验证移动平均的正确性（检查最后一个时间点）
    let last_time = num_time - 1;
    for stock_idx in 0..num_stock {
        for factor_idx in 0..num_factors {
            let start_idx = last_time - window + 1;
            let expected_avg: f32 = (start_idx..=last_time)
                .map(|t| input_data[[stock_idx, t, factor_idx]])
                .sum::<f32>()
                / window as f32;

            let actual_avg = output_data[[stock_idx, last_time, factor_idx]];
            assert!(
                (expected_avg - actual_avg).abs() < 1e-6,
                "Moving average mismatch for stock {}, factor {}: expected {}, got {}",
                stock_idx,
                factor_idx,
                expected_avg,
                actual_avg
            );
        }
    }
    println!("✅ Moving average computation verified");
}

#[test]
fn test_ndarray_data_conversion() {
    println!("🧪 Testing ndarray to KunQuant data format conversion...");

    let num_stock = 8;
    let num_time = 10;
    let num_factors = 2;

    // 创建测试数据
    let input_data = create_random_input_data(num_stock, num_time, num_factors);
    println!("✅ Created test data with shape: {:?}", input_data.dim());

    // 转换为KunQuant格式
    let kunquant_data = ndarray_to_kunquant_format(&input_data);
    println!(
        "✅ Converted to KunQuant format, length: {}",
        kunquant_data.len()
    );

    // 验证数据长度
    assert_eq!(kunquant_data.len(), num_stock * num_time * num_factors);

    // 转换回ndarray格式
    let recovered_data =
        kunquant_to_ndarray_format(&kunquant_data, num_stock, num_time, num_factors);
    println!("✅ Converted back to ndarray format");

    // 验证数据一致性
    for stock_idx in 0..num_stock {
        for time_idx in 0..num_time {
            for factor_idx in 0..num_factors {
                let original = input_data[[stock_idx, time_idx, factor_idx]];
                let recovered = recovered_data[[stock_idx, time_idx, factor_idx]];
                assert!(
                    (original - recovered).abs() < 1e-6,
                    "Data mismatch at [{}, {}, {}]: original {}, recovered {}",
                    stock_idx,
                    time_idx,
                    factor_idx,
                    original,
                    recovered
                );
            }
        }
    }
    println!("✅ Data conversion round-trip verified");
}

#[test]
fn test_ndarray_statistical_operations() {
    println!("🧪 Testing ndarray statistical operations...");

    let num_stock = 16;
    let num_time = 50;
    let num_factors = 4;

    let data = create_random_input_data(num_stock, num_time, num_factors);
    println!("✅ Created data with shape: {:?}", data.dim());

    // 计算各种统计量
    let mean_by_stock = data.mean_axis(Axis(1)).unwrap(); // [num_stock, num_factors]
    let mean_by_time = data.mean_axis(Axis(0)).unwrap(); // [num_time, num_factors]
    let mean_by_factor = data.mean_axis(Axis(2)).unwrap(); // [num_stock, num_time]

    println!("✅ Computed means:");
    println!("   - Mean by stock shape: {:?}", mean_by_stock.dim());
    println!("   - Mean by time shape: {:?}", mean_by_time.dim());
    println!("   - Mean by factor shape: {:?}", mean_by_factor.dim());

    // 验证形状
    assert_eq!(mean_by_stock.dim(), (num_stock, num_factors));
    assert_eq!(mean_by_time.dim(), (num_time, num_factors));
    assert_eq!(mean_by_factor.dim(), (num_stock, num_time));

    // 计算标准差
    let std_by_stock = data.std_axis(Axis(1), 1.0); // [num_stock, num_factors]
    println!(
        "✅ Computed standard deviation by stock, shape: {:?}",
        std_by_stock.dim()
    );

    // 验证统计量的合理性（标准差应该为正）
    for &std_val in std_by_stock.iter() {
        assert!(
            std_val >= 0.0,
            "Standard deviation should be non-negative, got {}",
            std_val
        );
    }

    println!("✅ Statistical operations completed successfully");
}

#[test]
fn test_ndarray_factor_computation_pipeline() {
    println!("🧪 Testing complete factor computation pipeline with ndarray...");

    let num_stock = 8;
    let num_time = 20;
    let num_factors = 2;

    // 步骤1: 创建输入数据
    let input_data = create_random_input_data(num_stock, num_time, num_factors);
    println!("✅ Step 1: Created input data {:?}", input_data.dim());

    // 步骤2: 应用多种因子计算
    let ma_5 = compute_moving_average(&input_data, 5);
    let ma_10 = compute_moving_average(&input_data, 10);

    // 步骤3: 组合因子结果
    let mut combined_factors = Array3::zeros((num_stock, num_time, num_factors * 3));

    // 原始因子
    combined_factors
        .slice_mut(s![.., .., 0..num_factors])
        .assign(&input_data);
    // 5日移动平均
    combined_factors
        .slice_mut(s![.., .., num_factors..num_factors * 2])
        .assign(&ma_5);
    // 10日移动平均
    combined_factors
        .slice_mut(s![.., .., num_factors * 2..num_factors * 3])
        .assign(&ma_10);

    println!(
        "✅ Step 2-3: Computed and combined factors, final shape: {:?}",
        combined_factors.dim()
    );

    // 步骤4: 计算因子间的相关性
    let final_num_factors = num_factors * 3;
    let mut correlation_matrix = Array3::zeros((num_stock, final_num_factors, final_num_factors));

    for stock_idx in 0..num_stock {
        for i in 0..final_num_factors {
            for j in 0..final_num_factors {
                let factor_i = combined_factors.slice(s![stock_idx, .., i]);
                let factor_j = combined_factors.slice(s![stock_idx, .., j]);

                let mean_i = factor_i.mean().unwrap();
                let mean_j = factor_j.mean().unwrap();

                let covariance: f32 = factor_i
                    .iter()
                    .zip(factor_j.iter())
                    .map(|(&x, &y)| (x - mean_i) * (y - mean_j))
                    .sum::<f32>()
                    / (num_time - 1) as f32;

                let std_i = factor_i.std(1.0);
                let std_j = factor_j.std(1.0);

                let correlation = if std_i > 1e-8 && std_j > 1e-8 {
                    covariance / (std_i * std_j)
                } else {
                    0.0
                };

                correlation_matrix[[stock_idx, i, j]] = correlation;
            }
        }
    }

    println!(
        "✅ Step 4: Computed correlation matrix, shape: {:?}",
        correlation_matrix.dim()
    );

    // 验证对角线元素接近1（自相关）
    for stock_idx in 0..num_stock {
        for i in 0..final_num_factors {
            let self_corr = correlation_matrix[[stock_idx, i, i]];
            assert!(
                (self_corr - 1.0).abs() < 0.1,
                "Self-correlation should be close to 1.0, got {} for stock {}, factor {}",
                self_corr,
                stock_idx,
                i
            );
        }
    }

    println!(
        "✅ Pipeline completed successfully with output shape: {:?}",
        correlation_matrix.dim()
    );
}

/// 生成符合KunQuant要求的股票数据 [num_stock, num_time, num_factors]
/// 其中 num_factors = 6 对应 [open, high, low, close, volume, amount]
fn generate_stock_data_ndarray(num_stock: usize, num_time: usize) -> Array3<f32> {
    let mut rng = thread_rng();
    let mut data = Array3::zeros((num_stock, num_time, 6));

    for stock_idx in 0..num_stock {
        for time_idx in 0..num_time {
            let base_price = rng.gen_range(10.0..200.0);
            let volatility = rng.gen_range(0.01..0.05);

            let open = base_price * (1.0 + rng.gen_range(-volatility..volatility));
            let high = open * (1.0 + rng.gen_range(0.0..volatility));
            let low = open * (1.0 - rng.gen_range(0.0..volatility));
            let close = rng.gen_range(low..=high);
            let volume = rng.gen_range(1000000.0..10000000.0);
            let amount = close * volume;

            data[[stock_idx, time_idx, 0]] = open;
            data[[stock_idx, time_idx, 1]] = high;
            data[[stock_idx, time_idx, 2]] = low;
            data[[stock_idx, time_idx, 3]] = close;
            data[[stock_idx, time_idx, 4]] = volume;
            data[[stock_idx, time_idx, 5]] = amount;
        }
    }

    data
}

/// 将ndarray格式 [num_stock, num_time, num_factors] 转换为KunQuant的行主序格式
/// KunQuant期望: [time0_stock0, time0_stock1, ..., time1_stock0, time1_stock1, ...]
fn ndarray_to_kunquant_buffer(data: &Array3<f32>, factor_idx: usize) -> Vec<f32> {
    let (num_stock, num_time, _) = data.dim();
    let mut buffer = Vec::with_capacity(num_stock * num_time);

    // KunQuant的行主序: 时间优先，然后股票
    for time_idx in 0..num_time {
        for stock_idx in 0..num_stock {
            buffer.push(data[[stock_idx, time_idx, factor_idx]]);
        }
    }

    buffer
}

/// 将KunQuant输出转换为ndarray格式 [num_stock, num_time, 1]
fn kunquant_buffer_to_ndarray(buffer: &[f32], num_stock: usize, num_time: usize) -> Array3<f32> {
    let mut result = Array3::zeros((num_stock, num_time, 1));

    // 从KunQuant的行主序格式转换
    for time_idx in 0..num_time {
        for stock_idx in 0..num_stock {
            let flat_idx = time_idx * num_stock + stock_idx;
            result[[stock_idx, time_idx, 0]] = buffer[flat_idx];
        }
    }

    result
}

#[test]
fn test_kunquant_alpha001_with_ndarray() -> Result<()> {
    println!("🧪 Testing KunQuant Alpha001 factor computation with ndarray format...");

    // 检查测试库是否存在
    let lib_path = "test_libs/alpha001_lib.so";
    if !Path::new(lib_path).exists() {
        panic!("Alpha001 library not found. Please run 'python generate_test_factor.py' first");
    }

    // 设置参数
    let num_stock = 8; // KunQuant要求股票数量是8的倍数
    let num_time = 100;

    // 步骤1: 使用ndarray生成输入数据 [num_stock, num_time, 6]
    let input_data = generate_stock_data_ndarray(num_stock, num_time);
    println!("✅ Generated input data with shape: {:?}", input_data.dim());

    // 步骤2: 转换为KunQuant格式的缓冲区
    let mut open_buffer = ndarray_to_kunquant_buffer(&input_data, 0);
    let mut high_buffer = ndarray_to_kunquant_buffer(&input_data, 1);
    let mut low_buffer = ndarray_to_kunquant_buffer(&input_data, 2);
    let mut close_buffer = ndarray_to_kunquant_buffer(&input_data, 3);
    let mut volume_buffer = ndarray_to_kunquant_buffer(&input_data, 4);
    let mut amount_buffer = ndarray_to_kunquant_buffer(&input_data, 5);
    let mut alpha001_output = vec![0.0f32; num_stock * num_time];

    println!("✅ Converted to KunQuant buffer format");

    // 步骤3: 创建KunQuant执行器和库
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("alpha001_test")?;

    // 步骤4: 设置缓冲区映射
    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("open", &mut open_buffer)?;
    buffers.set_buffer_slice("high", &mut high_buffer)?;
    buffers.set_buffer_slice("low", &mut low_buffer)?;
    buffers.set_buffer_slice("close", &mut close_buffer)?;
    buffers.set_buffer_slice("volume", &mut volume_buffer)?;
    buffers.set_buffer_slice("amount", &mut amount_buffer)?;
    buffers.set_buffer_slice("alpha001", &mut alpha001_output)?;

    println!("✅ Set up KunQuant buffers");

    // 步骤5: 执行Alpha001因子计算
    let params = BatchParams::full_range(num_stock, num_time)?;
    run_graph(&executor, &module, &buffers, &params)?;

    println!("✅ Executed Alpha001 factor computation");

    // 步骤6: 转换输出为ndarray格式 [num_stock, num_time, 1]
    let output_data = kunquant_buffer_to_ndarray(&alpha001_output, num_stock, num_time);
    println!(
        "✅ Converted output to ndarray format: {:?}",
        output_data.dim()
    );

    // 步骤7: 验证结果
    let mut finite_count = 0;
    let mut non_zero_count = 0;
    let mut nan_count = 0;

    for stock_idx in 0..num_stock {
        for time_idx in 0..num_time {
            let value = output_data[[stock_idx, time_idx, 0]];
            if value.is_finite() {
                finite_count += 1;
                if value != 0.0 {
                    non_zero_count += 1;
                }
                // Alpha001 应该产生 [0, 1] 范围内的值（由于rank操作）
                assert!(
                    value >= 0.0 && value <= 1.0,
                    "Alpha001 value out of range [0, 1] at [{}, {}]: {}",
                    stock_idx,
                    time_idx,
                    value
                );
            } else if value.is_nan() {
                nan_count += 1;
            }
        }
    }

    println!("📊 Alpha001 Results Analysis:");
    println!("   Total values: {}", num_stock * num_time);
    println!("   Finite values: {}", finite_count);
    println!("   Non-zero values: {}", non_zero_count);
    println!("   NaN values: {}", nan_count);

    // 打印一些样本值
    println!("📋 Sample values from output ndarray:");
    for stock_idx in 0..num_stock.min(3) {
        for time_idx in (0..num_time.min(10)).step_by(2) {
            let value = output_data[[stock_idx, time_idx, 0]];
            println!(
                "   stock[{}], time[{}] = {:8.4}",
                stock_idx, time_idx, value
            );
        }
    }

    // 计算一些统计量
    let finite_values: Vec<f32> = output_data
        .iter()
        .filter(|&&x| x.is_finite())
        .cloned()
        .collect();

    if !finite_values.is_empty() {
        let mean = finite_values.iter().sum::<f32>() / finite_values.len() as f32;
        let min_val = finite_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = finite_values
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        println!("📈 Statistics of finite values:");
        println!("   Mean: {:.4}", mean);
        println!("   Min:  {:.4}", min_val);
        println!("   Max:  {:.4}", max_val);
    }

    // 验证基本要求
    assert!(
        finite_count > (num_stock * num_time) / 2,
        "Too many non-finite values: {}/{}",
        (num_stock * num_time) - finite_count,
        num_stock * num_time
    );

    println!("✅ KunQuant Alpha001 with ndarray test passed!");
    Ok(())
}

#[test]
fn test_ndarray_batch_factor_computation() -> Result<()> {
    println!("🧪 Testing batch factor computation with ndarray format...");

    // 检查测试库是否存在
    let lib_path = "test_libs/simple_test_lib.so";
    if !Path::new(lib_path).exists() {
        panic!("Simple test library not found. Please run 'python generate_test_factor.py' first");
    }

    // 设置参数
    let num_stock = 16; // 测试更多股票
    let num_time = 50;
    let num_input_factors = 1; // 简单测试只有一个输入因子
    let _num_output_factors = 1; // 一个输出因子

    // 步骤1: 创建输入数据 [num_stock, num_time, num_input_factors]
    let mut input_data = Array3::zeros((num_stock, num_time, num_input_factors));
    let mut rng = thread_rng();

    // 填充随机数据
    for stock_idx in 0..num_stock {
        for time_idx in 0..num_time {
            input_data[[stock_idx, time_idx, 0]] = rng.gen_range(-10.0..10.0);
        }
    }

    println!("✅ Generated input data with shape: {:?}", input_data.dim());

    // 步骤2: 转换为KunQuant格式
    let mut input_buffer = ndarray_to_kunquant_buffer(&input_data, 0);
    let mut output_buffer = vec![0.0f32; num_stock * num_time];

    // 步骤3: 执行计算 (input * 3)
    let executor = Executor::single_thread()?;
    let library = Library::load(lib_path)?;
    let module = library.get_module("simple_test")?;

    let mut buffers = BufferNameMap::new()?;
    buffers.set_buffer_slice("input", &mut input_buffer)?;
    buffers.set_buffer_slice("output", &mut output_buffer)?;

    let params = BatchParams::full_range(num_stock, num_time)?;
    run_graph(&executor, &module, &buffers, &params)?;

    println!("✅ Executed simple factor computation (input * 3)");

    // 步骤4: 转换输出为ndarray格式
    let output_data = kunquant_buffer_to_ndarray(&output_buffer, num_stock, num_time);
    println!(
        "✅ Converted output to ndarray format: {:?}",
        output_data.dim()
    );

    // 步骤5: 验证计算结果
    let mut correct_count = 0;
    let tolerance = 1e-6;

    for stock_idx in 0..num_stock {
        for time_idx in 0..num_time {
            let input_val = input_data[[stock_idx, time_idx, 0]];
            let expected = input_val * 3.0;
            let actual = output_data[[stock_idx, time_idx, 0]];

            if (expected - actual).abs() < tolerance {
                correct_count += 1;
            } else {
                println!(
                    "❌ Mismatch at [{}, {}]: input={:.4}, expected={:.4}, actual={:.4}",
                    stock_idx, time_idx, input_val, expected, actual
                );
            }
        }
    }

    println!("📊 Computation Results:");
    println!("   Total values: {}", num_stock * num_time);
    println!("   Correct values: {}", correct_count);
    println!(
        "   Accuracy: {:.2}%",
        (correct_count as f32 / (num_stock * num_time) as f32) * 100.0
    );

    // 打印一些样本值进行验证
    println!("📋 Sample input/output pairs:");
    for stock_idx in 0..num_stock.min(3) {
        for time_idx in (0..num_time.min(6)).step_by(2) {
            let input_val = input_data[[stock_idx, time_idx, 0]];
            let output_val = output_data[[stock_idx, time_idx, 0]];
            println!(
                "   stock[{}], time[{}]: {:.4} -> {:.4} (expected: {:.4})",
                stock_idx,
                time_idx,
                input_val,
                output_val,
                input_val * 3.0
            );
        }
    }

    // 验证所有计算都正确
    assert_eq!(
        correct_count,
        num_stock * num_time,
        "Not all computations are correct: {}/{}",
        correct_count,
        num_stock * num_time
    );

    println!("✅ Batch factor computation with ndarray test passed!");
    Ok(())
}
