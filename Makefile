.PHONY: build run test check fmt clippy lint clean fix help

# 默认目标
build:
	cargo build --release

help:
	@echo "可用命令:"
	@echo "  make build    - 编译项目"
	@echo "  make run      - 运行项目"
	@echo "  make test     - 运行测试"
	@echo "  make check    - 快速检查（不生成二进制文件）"
	@echo "  make fmt      - 格式化代码"
	@echo "  make lint     - 运行 clippy lint"
	@echo "  make fix      - 自动修复代码问题并格式化"
	@echo "  make clean    - 清理构建产物"

run:
	cargo run

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

lint: clippy

clean:
	cargo clean

fix:
	cargo fix --allow-dirty --all-features && cargo fmt