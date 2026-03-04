.PHONY: build run test check fmt clippy clean help

# 默认目标
help:
	@echo "可用命令:"
	@echo "  make build    - 编译项目"
	@echo "  make run      - 运行项目"
	@echo "  make test     - 运行测试"
	@echo "  make check    - 快速检查（不生成二进制文件）"
	@echo "  make fmt      - 格式化代码"
	@echo "  make clippy   - 运行 clippy lint"
	@echo "  make clean    - 清理构建产物"

build:
	cargo build --release

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

clean:
	cargo clean