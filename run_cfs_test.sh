#!/bin/bash

# CFS 调度器测试脚本

echo "========================================"
echo "CFS Scheduler Test Runner"
echo "========================================"
echo ""

# 编译用户程序
echo "[1/3] Building user programs..."
cd "$(dirname "$0")/../user" || exit 1
cargo build --release
if [ $? -ne 0 ]; then
    echo "Error: Failed to build user programs"
    exit 1
fi
echo "✓ User programs built successfully"
echo ""

# 编译内核
echo "[2/3] Building kernel..."
cd ../os || exit 1
make build
if [ $? -ne 0 ]; then
    echo "Error: Failed to build kernel"
    exit 1
fi
echo "✓ Kernel built successfully"
echo ""

# 运行 QEMU
echo "[3/3] Starting QEMU..."
echo "Once the system boots, type: cfs_test"
echo ""
echo "Press Ctrl+A then X to exit QEMU"
echo "========================================"
echo ""

make run
