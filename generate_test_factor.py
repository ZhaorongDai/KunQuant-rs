#!/usr/bin/env python3
"""
Generate a simple test factor library for Rust binding testing
"""

import os
import sys
from KunQuant.jit import cfake
from KunQuant.Driver import KunCompilerConfig
from KunQuant.Op import Builder, Input, Output
from KunQuant.Stage import Function

def create_simple_test_factor():
    """Create a simple test factor: output = input + input * 2 = input * 3"""
    builder = Builder()
    with builder:
        input_data = Input("input")
        # Create: input * 2
        doubled = input_data * 2.0
        # Create: input + (input * 2) = input * 3
        result = input_data + doubled
        Output(result, "output")

    return Function(builder.ops)

def create_simple_stream_factor():
    """Create a simple streaming factor for testing"""
    builder = Builder()
    with builder:
        # Use standard OHLCV inputs for streaming
        close = Input("close")
        open_price = Input("open")
        high = Input("high")
        low = Input("low")
        volume = Input("volume")
        amount = Input("amount")

        # Simple factor: (close - open) / (high - low + 0.001)
        # This is similar to the test in KunQuant's C API test
        numerator = close - open_price
        denominator = (high - low) + 0.001
        result = numerator / denominator
        Output(result, "simple_stream")

    return Function(builder.ops)

def create_alpha001_factor():
    """Create Alpha001 factor from predefined library"""
    from KunQuant.predefined import Alpha101

    builder = Builder()
    with builder:
        vclose = Input("close")
        low = Input("low")
        high = Input("high")
        vopen = Input("open")
        amount = Input("amount")
        vol = Input("volume")

        all_data = Alpha101.AllData(
            low=low, high=high, close=vclose,
            open=vopen, amount=amount, volume=vol
        )
        Output(Alpha101.alpha001(all_data), "alpha001")

    return Function(builder.ops)

def main():
    # Create output directory with absolute path
    current_dir = os.path.dirname(os.path.abspath(__file__))
    test_libs_dir = os.path.join(current_dir, "test_libs")
    os.makedirs(test_libs_dir, exist_ok=True)

    # Generate simple test factor
    print("Generating simple test factor...")
    simple_factor = create_simple_test_factor()
    print("Simple factor expression:")
    print(simple_factor)

    # Compile simple factor
    simple_lib_path = os.path.join(test_libs_dir, "simple_test_lib")
    simple_lib = cfake.compileit([
        ("simple_test", simple_factor, KunCompilerConfig(input_layout="TS", output_layout="TS"))
    ], simple_lib_path, cfake.CppCompilerConfig())

    print(f"Simple test library compiled successfully!")
    print(f"Library path: {simple_lib_path}.so")

    # Generate Alpha001 factor
    print("\nGenerating Alpha001 factor...")
    alpha001_factor = create_alpha001_factor()
    print("Alpha001 factor expression (first 10 lines):")
    factor_str = str(alpha001_factor)
    lines = factor_str.split('\n')
    for i, line in enumerate(lines[:10]):
        print(line)
    if len(lines) > 10:
        print("...")

    # Compile Alpha001 factor
    alpha001_lib_path = os.path.join(test_libs_dir, "alpha001_lib")
    alpha001_lib = cfake.compileit([
        ("alpha001_test", alpha001_factor, KunCompilerConfig(input_layout="TS", output_layout="TS"))
    ], alpha001_lib_path, cfake.CppCompilerConfig())

    print(f"Alpha001 library compiled successfully!")
    print(f"Library path: {alpha001_lib_path}.so")

    # Generate simple streaming factor
    print("\nGenerating simple streaming factor...")
    simple_stream_factor = create_simple_stream_factor()
    print("Simple streaming factor expression:")
    print(simple_stream_factor)

    # Compile simple streaming factor
    simple_stream_lib_path = os.path.join(test_libs_dir, "simple_stream_lib")
    simple_stream_lib = cfake.compileit([
        ("simple_stream_test", simple_stream_factor,
         KunCompilerConfig(input_layout="TS", output_layout="STREAM"))
    ], simple_stream_lib_path, cfake.CppCompilerConfig())

    print(f"Simple streaming library compiled successfully!")
    print(f"Library path: {simple_stream_lib_path}.so")

    print("\nTest libraries generated successfully!")
    print("You can now run the Rust tests.")

if __name__ == "__main__":
    main()
