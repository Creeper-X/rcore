# 环境配置
本操作系统在Ubuntu20.04环境下开发
## Rust 开发环境配置
```
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```
## QEMU 模拟器安装
需要手动编译7.0版本
```
# 安装编译所需的依赖包
sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev libsdl2-dev libslirp-dev \
              git tmux python3 python3-pip ninja-build
# 下载源码包
wget https://download.qemu.org/qemu-7.0.0.tar.xz
# 解压
tar xvJf qemu-7.0.0.tar.xz
# 编译安装并配置 RISC-V 支持
cd qemu-7.0.0
./configure --target-list=riscv64-softmmu,riscv64-linux-user 
make -j$(nproc)
sudo make install
export PATH=$PATH:/path/to/qemu-7.0.0/build
```
# 运行
进入os目录
```
cd os
```
在os目录下
```
make run
```