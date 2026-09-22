use std::cmp::Ordering;
use rayon::prelude::*; // 병렬 처리 라이브러리 도입

#[derive(Clone)]
struct BigInt {
    digits: Vec<u32>,
}

impl BigInt {
    fn new(num_str: &str) -> Self {
        let digits = num_str.chars()
            .rev()
            .filter_map(|c| c.to_digit(10))
            .collect();
        BigInt { digits }
    }

    // --- 1. 기초 연산 --- //
    
    fn cmp(&self, other: &BigInt) -> Ordering {
        if self.digits.len() != other.digits.len() {
            return self.digits.len().cmp(&other.digits.len());
        }
        for (a, b) in self.digits.iter().rev().zip(other.digits.iter().rev()) {
            if a != b {
                return a.cmp(b);
            }
        }
        Ordering::Equal
    }

    fn add(&self, other: &BigInt) -> Self {
        let mut result_digits = Vec::new();
        let mut carry = 0;
        let max_len = std::cmp::max(self.digits.len(), other.digits.len());
        
        for i in 0..max_len {
            let a = if i < self.digits.len() { self.digits[i] } else { 0 };
            let b = if i < other.digits.len() { other.digits[i] } else { 0 };
            let sum = a + b + carry;
            result_digits.push(sum % 10);
            carry = sum / 10;
        }
        if carry > 0 {
            result_digits.push(carry);
        }
        BigInt { digits: result_digits }
    }

    fn sub(&self, other: &BigInt) -> Self {
        let mut result_digits = Vec::new();
        let mut borrow = 0;
        for i in 0..self.digits.len() {
            let a = self.digits[i];
            let b = if i < other.digits.len() { other.digits[i] } else { 0 };
            
            if a < b + borrow {
                result_digits.push(a + 10 - b - borrow);
                borrow = 1;
            } else {
                result_digits.push(a - b - borrow);
                borrow = 0;
            }
        }
        while result_digits.len() > 1 && *result_digits.last().unwrap() == 0 {
            result_digits.pop();
        }
        BigInt { digits: result_digits }
    }

    // 모듈러 연산 (기초 장제법)
    fn rem(&self, other: &BigInt) -> Self {
        if self.cmp(other) == Ordering::Less {
            return self.clone();
        }
        let mut remainder = BigInt::new("0");
        for &digit in self.digits.iter().rev() {
            if remainder.digits.len() == 1 && remainder.digits[0] == 0 {
                remainder.digits[0] = digit;
            } else {
                remainder.digits.insert(0, digit);
            }
            while remainder.cmp(other) != Ordering::Less {
                remainder = remainder.sub(other);
            }
        }
        remainder
    }

    // --- 2. 카라츠바 곱셈 관련 보조 도구 --- //

    // O(n^2) 일반 곱셈 (크기가 작을 때 탈출용)
    fn normal_mul(&self, other: &BigInt) -> Self {
        let mut result = vec![0; self.digits.len() + other.digits.len()];
        for i in 0..self.digits.len() {
            let mut carry = 0;
            for j in 0..other.digits.len() {
                let sum = result[i + j] + self.digits[i] * other.digits[j] + carry;
                result[i + j] = sum % 10;
                carry = sum / 10;
            }
            if carry > 0 {
                result[i + other.digits.len()] += carry;
            }
        }
        while result.len() > 1 && *result.last().unwrap() == 0 {
            result.pop();
        }
        BigInt { digits: result }
    }

    // x 10^m 자릿수 밀기
    fn shift_left(&self, m: usize) -> Self {
        if self.digits.len() == 1 && self.digits[0] == 0 { return self.clone(); }
        let mut new_digits = vec![0; m];
        new_digits.extend_from_slice(&self.digits);
        BigInt { digits: new_digits }
    }

    // 절반 쪼개기
    fn split_at(&self, m: usize) -> (Self, Self) {
        if m >= self.digits.len() {
            return (BigInt::new("0"), self.clone());
        }
        let (lower, upper) = self.digits.split_at(m);
        (
            BigInt { digits: upper.to_vec() },
            BigInt { digits: lower.to_vec() }
        )
    }

    fn print(&self) {
        for digit in self.digits.iter().rev() {
            print!("{}", digit);
        }
        println!();
    }
}

// --- 3. 멀티스레딩 카라츠바 핵심 로직 --- //

fn parallel_karatsuba(x: &BigInt, y: &BigInt) -> BigInt {
    let len_x = x.digits.len();
    let len_y = y.digits.len();

    // 1. 기저 조건: 길이가 32 이하면 스레드를 생성하지 않고 즉시 일반 곱셈으로 끝냅니다.
    // (스레드를 생성하는 오버헤드를 막기 위함)
    if len_x < 32 || len_y < 32 {
        return x.normal_mul(y);
    }

    let m = std::cmp::max(len_x, len_y) / 2;
    let (a, b) = x.split_at(m);
    let (c, d) = y.split_at(m);

    // 2. Rayon의 join 함수를 통한 병렬 처리
    // join(|| { 함수1 }, || { 함수2 })는 함수1과 함수2를 각기 다른 코어에 던져서 '동시에' 실행합니다.
    let (ac, bd) = rayon::join(
        || parallel_karatsuba(&a, &c),  // 코어 A가 담당
        || parallel_karatsuba(&b, &d)   // 코어 B가 담당
    );
    
    // (a+b)와 (c+d)는 크기가 작으니 그냥 현재 코어에서 진행
    let a_plus_b = a.add(&b);
    let c_plus_d = c.add(&d);
    
    // 세 번째 K 연산은 다른 연산이 다 끝난 뒤에 들어갑니다.
    let k = parallel_karatsuba(&a_plus_b, &c_plus_d);

    let ad_plus_bc = k.sub(&ac).sub(&bd);
    let ac_shifted = ac.shift_left(2 * m);
    let mid_shifted = ad_plus_bc.shift_left(m);

    ac_shifted.add(&mid_shifted).add(&bd)
}

fn main() {
    let x = BigInt::new("1234567890123456789012345678901234567890");
    let y = BigInt::new("9876543210987654321098765432109876543210");

    println!("--- 멀티스레딩 카라츠바 곱셈 시작 ---");
    let result = parallel_karatsuba(&x, &y);
    
    print!("결과: ");
    result.print();
}
