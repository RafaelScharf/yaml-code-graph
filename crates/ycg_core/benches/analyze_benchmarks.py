#!/usr/bin/env python3
"""
Performance Benchmark Analysis Tool

This script analyzes benchmark results and validates performance requirements:
- Requirement 10.1: Level 1 ≤ 110% of Level 0
- Requirement 10.2: Level 2 ≤ 125% of Level 0

Usage:
    python3 analyze_benchmarks.py benchmark_results.txt
"""

import sys
import re
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional


class BenchmarkResult:
    """Represents a single benchmark result"""
    
    def __init__(self, name: str, time_ms: float, change_pct: Optional[float] = None):
        self.name = name
        self.time_ms = time_ms
        self.change_pct = change_pct
    
    def __repr__(self):
        return f"BenchmarkResult({self.name}, {self.time_ms:.2f}ms, {self.change_pct}%)"


class BenchmarkAnalyzer:
    """Analyzes benchmark results and validates requirements"""
    
    def __init__(self):
        self.results: Dict[str, BenchmarkResult] = {}
    
    def parse_criterion_output(self, output: str) -> None:
        """Parse Criterion benchmark output"""
        
        # Pattern to match benchmark results
        # Example: "processing_time_simple_ts/level/0_default"
        #          "                        time:   [45.234 ms 45.891 ms 46.612 ms]"
        
        lines = output.split('\n')
        current_bench = None
        
        for line in lines:
            # Match benchmark name
            if '/' in line and not line.strip().startswith('time:'):
                current_bench = line.strip()
            
            # Match time result
            time_match = re.search(r'time:\s+\[[\d.]+ \w+ ([\d.]+) (\w+)', line)
            if time_match and current_bench:
                time_value = float(time_match.group(1))
                time_unit = time_match.group(2)
                
                # Convert to milliseconds
                if time_unit == 'ms':
                    time_ms = time_value
                elif time_unit == 'µs':
                    time_ms = time_value / 1000.0
                elif time_unit == 's':
                    time_ms = time_value * 1000.0
                else:
                    time_ms = time_value
                
                self.results[current_bench] = BenchmarkResult(current_bench, time_ms)
            
            # Match change percentage
            change_match = re.search(r'change:\s+\[.*?\s+([\+\-]?[\d.]+)%', line)
            if change_match and current_bench and current_bench in self.results:
                change_pct = float(change_match.group(1))
                self.results[current_bench].change_pct = change_pct
    
    def validate_requirements(self) -> Dict[str, bool]:
        """Validate performance requirements"""
        
        validation = {
            'req_10_1': None,  # Level 1 ≤ 110% of Level 0
            'req_10_2': None,  # Level 2 ≤ 125% of Level 0
        }
        
        # Find Level 0, 1, and 2 results for each project
        projects = ['simple_ts', 'nestjs']
        
        for project in projects:
            level_0 = None
            level_1 = None
            level_2 = None
            
            for name, result in self.results.items():
                if project in name:
                    if '0_default' in name:
                        level_0 = result
                    elif '1_signatures' in name:
                        level_1 = result
                    elif '2_logic' in name:
                        level_2 = result
            
            if level_0 and level_1:
                overhead_1 = (level_1.time_ms / level_0.time_ms) * 100
                passes_10_1 = overhead_1 <= 110.0
                
                print(f"\n{project.upper()} - Requirement 10.1:")
                print(f"  Level 0: {level_0.time_ms:.2f}ms")
                print(f"  Level 1: {level_1.time_ms:.2f}ms")
                print(f"  Overhead: {overhead_1:.1f}%")
                print(f"  Status: {'✅ PASS' if passes_10_1 else '❌ FAIL'} (requirement: ≤ 110%)")
                
                if validation['req_10_1'] is None:
                    validation['req_10_1'] = passes_10_1
                else:
                    validation['req_10_1'] = validation['req_10_1'] and passes_10_1
            
            if level_0 and level_2:
                overhead_2 = (level_2.time_ms / level_0.time_ms) * 100
                passes_10_2 = overhead_2 <= 125.0
                
                print(f"\n{project.upper()} - Requirement 10.2:")
                print(f"  Level 0: {level_0.time_ms:.2f}ms")
                print(f"  Level 2: {level_2.time_ms:.2f}ms")
                print(f"  Overhead: {overhead_2:.1f}%")
                print(f"  Status: {'✅ PASS' if passes_10_2 else '❌ FAIL'} (requirement: ≤ 125%)")
                
                if validation['req_10_2'] is None:
                    validation['req_10_2'] = passes_10_2
                else:
                    validation['req_10_2'] = validation['req_10_2'] and passes_10_2
        
        return validation
    
    def generate_report(self) -> str:
        """Generate a detailed analysis report"""
        
        report = []
        report.append("=" * 60)
        report.append("PERFORMANCE BENCHMARK ANALYSIS")
        report.append("=" * 60)
        report.append("")
        
        # Validate requirements
        validation = self.validate_requirements()
        
        report.append("")
        report.append("=" * 60)
        report.append("REQUIREMENTS VALIDATION SUMMARY")
        report.append("=" * 60)
        report.append("")
        
        for req, status in validation.items():
            if status is not None:
                status_str = "✅ PASS" if status else "❌ FAIL"
                req_name = req.replace('_', '.').upper()
                report.append(f"{req_name}: {status_str}")
        
        report.append("")
        report.append("=" * 60)
        report.append("ALL BENCHMARK RESULTS")
        report.append("=" * 60)
        report.append("")
        
        for name, result in sorted(self.results.items()):
            report.append(f"{name}")
            report.append(f"  Time: {result.time_ms:.2f}ms")
            if result.change_pct is not None:
                report.append(f"  Change: {result.change_pct:+.1f}%")
            report.append("")
        
        return "\n".join(report)


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 analyze_benchmarks.py benchmark_results.txt")
        sys.exit(1)
    
    results_file = Path(sys.argv[1])
    
    if not results_file.exists():
        print(f"Error: File not found: {results_file}")
        sys.exit(1)
    
    # Read benchmark results
    with open(results_file, 'r') as f:
        output = f.read()
    
    # Analyze results
    analyzer = BenchmarkAnalyzer()
    analyzer.parse_criterion_output(output)
    
    # Generate and print report
    report = analyzer.generate_report()
    print(report)
    
    # Save report
    report_file = results_file.parent / "benchmark_analysis.txt"
    with open(report_file, 'w') as f:
        f.write(report)
    
    print(f"\nReport saved to: {report_file}")


if __name__ == '__main__':
    main()
