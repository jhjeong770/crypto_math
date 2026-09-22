# 🧮 Crypto Math: High-Performance BigInt in Rust

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Rayon](https://img.shields.io/badge/Rayon-Parallel_Computing-blue?style=for-the-badge)

이 프로젝트는 거대 정수(BigInt)의 대수적 연산과 카라츠바(Karatsuba) 알고리즘을 밑바닥부터 직접 구현하며, 컴퓨터 구조적 최적화를 연구한 Rust 레포지토리입니다.

## 프로젝트 목적
단순히 수학 공식을 코드로 옮기는 것을 넘어, **메모리 안전성(Ownership), 힙 할당 최소화, 그리고 멀티스레딩**을 통해 CPU 하드웨어의 성능을 극한으로 끌어내는 시스템 엔지니어링을 실험합니다.

## 핵심 구현 사항
- **Custom BigInt**: `Vec<u32>`를 활용한 가변 길이 정수 자료구조 및 기초 연산(Add, Sub, Rem)
- **Karatsuba Algorithm**: 일반적인 $O(n^2)$ 곱셈의 시간 복잡도를 $O(n^{1.585})$로 단축한 분할 정복 연산
- **Multi-threading Optimization**: `rayon` 라이브러리의 Work-stealing 기법을 활용하여, 거대한 수의 곱셈 트리를 다중 CPU 코어에 병렬로 분산 처리

## 실행 방법
Rust와 Cargo가 설치된 환경에서 아래 명령어를 통해 즉시 실행할 수 있습니다.

## 향후 연구 과제 (To-Do)
[ ] 스크래치패드(Scratchpad)를 도입하여 재귀 호출 시 힙 메모리 할당(Zero-Copy) 완벽 제거

[ ] 나눗셈 연산 속도를 비약적으로 높이는 몽고메리 감산(Montgomery Reduction) 도입

```bash
cargo run --release
