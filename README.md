# CryptoMath: Pure Rust High-Performance Cryptography Engine

## Overview
**CryptoMath**는 공부를 위해 외부 의존성(Crate) 없이 100% 순수 Rust 표준 라이브러리만으로 밑바닥부터 구현한 고성능 거대 정수(BigInt) 및 암호학 연산 엔진입니다. 
단순한 기능 구현을 넘어, 수학적 알고리즘(대수학/정수론)과 시스템 프로그래밍(메모리 최적화)의 결합을 통해 CPU 사이클과 메모리 사용량을 극한으로 최적화하도록 설계되었습니다.

## Key Features

* **Zero-Copy Karatsuba Multiplication ($O(n^{1.585})$)**
  * 힙 메모리 할당(`Vec::new()`)을 완벽히 제거한 In-place 슬라이스(`&mut [u32]`) 기반 연산 적용
  * 재귀 호출 시 Scratchpad 메모리 분할(Partitioning) 기법을 도입하여 병목 현상 제거
* **Montgomery Reduction (REDC)**
  * 값비싼 모듈로 나눗셈(`%`) 연산을 컴퓨터가 처리하기 쉬운 비트 시프트(`>>`)와 덧셈으로 완전 치환
  * 현실 세계의 정수와 몽고메리 거울 공간(Domain) 간의 $O(1)$ 변환 포털 구현
* **High-Speed Modular Exponentiation**
  * Square-and-Multiply 기법과 몽고메리 연산을 결합하여 나눗셈 회로 없이 $O(\log B)$ 시간 복잡도로 거듭제곱 수행
* **Miller-Rabin Primality Test**
  * 유한체 위에서 1의 자명하지 않은 제곱근(Non-trivial square root) 성질을 활용한 초고속 거대 소수 판별기

## Engineering Philosophy
이 프로젝트는 두 가지 철학을 바탕으로 구축되었습니다:
1. **System Engineering:** 가비지 컬렉터(GC)가 없는 Rust의 소유권(Ownership) 및 Borrow Checker 규칙을 활용해, 캐시 적중률을 높이고 운영체제의 개입(메모리 재할당)을 차단합니다.
2. **Abstract Algebra:** 정수론의 합동식(Congruence) 이론을 소프트웨어 구조로 매핑하여 알고리즘의 본질적인 시간 복잡도를 낮춥니다.

## Usage
프로젝트를 복제한 후, Rust의 최적화 컴파일 플래그를 켜서 실행하면 가장 압도적인 성능을 확인할 수 있습니다.

```bash
cargo run --release
