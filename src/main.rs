use std::cmp::Ordering;

// ====================================================================
// 1. 거대 정수 (BigInt) 자료구조 및 기본 메서드
// ====================================================================
#[derive(Debug, Clone)]
pub struct BigInt {
    pub digits: Vec<u32>,
}

impl BigInt {
    /// 불필요한 상위 0을 제거합니다.
    pub fn trim_zeros(mut self) -> Self {
        while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
            self.digits.pop();
        }
        self
    }

    /// 두 BigInt의 크기를 비교합니다.
    pub fn cmp(&self, other: &Self) -> Ordering {
        let a = self.clone().trim_zeros();
        let b = other.clone().trim_zeros();
        
        if a.digits.len() != b.digits.len() {
            return a.digits.len().cmp(&b.digits.len());
        }
        for i in (0..a.digits.len()).rev() {
            if a.digits[i] != b.digits[i] {
                return a.digits[i].cmp(&b.digits[i]);
            }
        }
        Ordering::Equal
    }

    /// 뺄셈 래퍼 함수 (반드시 self >= other 일 때만 사용)
    pub fn sub(&self, other: &Self) -> Self {
        let mut out = vec![0u32; self.digits.len()];
        sub_slices(&self.digits, &other.digits, &mut out);
        BigInt { digits: out }.trim_zeros()
    }

    /// Zero-Copy 카라츠바 곱셈 진입점
    pub fn mul_karatsuba(&self, other: &Self) -> Self {
        let max_len = std::cmp::max(self.digits.len(), other.digits.len());
        let mut out = vec![0u32; self.digits.len() + other.digits.len()];
        let mut scratch = vec![0u32; max_len * 6];

        karatsuba_core(&self.digits, &other.digits, &mut out, &mut scratch);
        BigInt { digits: out }.trim_zeros()
    }
}

// ====================================================================
// 2. 시스템 레벨 최적화 코어 (Zero-Copy 슬라이스 연산)
// ====================================================================
fn add_slices(x: &[u32], y: &[u32], out: &mut [u32]) {
    let mut carry = 0u64;
    let max_len = std::cmp::max(x.len(), y.len());
    for i in 0..max_len {
        let xi = if i < x.len() { x[i] as u64 } else { 0 };
        let yi = if i < y.len() { y[i] as u64 } else { 0 };
        let sum = xi + yi + carry;
        out[i] = sum as u32;         
        carry = sum >> 32;           
    }
    if max_len < out.len() { out[max_len] = carry as u32; }
}

fn sub_slices(x: &[u32], y: &[u32], out: &mut [u32]) {
    let mut borrow = 0i64;
    for i in 0..x.len() {
        let xi = x[i] as i64;
        let yi = if i < y.len() { y[i] as i64 } else { 0 };
        let diff = xi - yi - borrow;
        if diff < 0 {
            out[i] = (diff + (1i64 << 32)) as u32;
            borrow = 1;
        } else {
            out[i] = diff as u32;
            borrow = 0;
        }
    }
}

/// x에서 y를 제자리(in-place)에서 뺍니다. (x = x - y)
/// Borrow Checker 충돌을 피하기 위해 추가된 함수입니다.
fn sub_assign(x: &mut [u32], y: &[u32]) {
    let mut borrow = 0i64;
    for i in 0..x.len() {
        let xi = x[i] as i64;
        let yi = if i < y.len() { y[i] as i64 } else { 0 };
        let diff = xi - yi - borrow;
        if diff < 0 {
            x[i] = (diff + (1i64 << 32)) as u32;
            borrow = 1;
        } else {
            x[i] = diff as u32;
            borrow = 0;
        }
    }
}

fn multiply_schoolbook(x: &[u32], y: &[u32], out: &mut [u32]) {
    for val in out.iter_mut() { *val = 0; }
    for i in 0..x.len() {
        let mut carry = 0u64;
        let xi = x[i] as u64; 
        for j in 0..y.len() {
            let yj = y[j] as u64;
            let sum = (out[i + j] as u64) + (xi * yj) + carry;
            out[i + j] = sum as u32;
            carry = sum >> 32;
        }
        if i + y.len() < out.len() { out[i + y.len()] = carry as u32; }
    }
}

fn karatsuba_core(x: &[u32], y: &[u32], out: &mut [u32], scratch: &mut [u32]) {
    let len = std::cmp::max(x.len(), y.len());
    if x.len() <= 32 || y.len() <= 32 {
        multiply_schoolbook(x, y, out);
        return;
    }
    let m = len / 2;
    let (x0, x1) = x.split_at(std::cmp::min(m, x.len()));
    let (y0, y1) = y.split_at(std::cmp::min(m, y.len()));

    let (sum_x, rest) = scratch.split_at_mut(m + 1);
    let (sum_y, rest) = rest.split_at_mut(m + 1);
    let (z1_temp, next_scratch) = rest.split_at_mut(len + 2); 

    add_slices(x0, x1, sum_x);
    add_slices(y0, y1, sum_y);

    let (out_z0, out_rest) = out.split_at_mut(m * 2);
    karatsuba_core(x0, y0, out_z0, next_scratch);

    let (out_z2, _) = out_rest.split_at_mut(x1.len() + y1.len() + 2); 
    karatsuba_core(x1, y1, out_z2, next_scratch);

    karatsuba_core(sum_x, sum_y, z1_temp, next_scratch);

    // Borrow Checker 에러 해결: In-place 뺄셈 함수 사용
    sub_assign(z1_temp, out_z0);
    sub_assign(z1_temp, out_z2);

    let mut carry = 0u64;
    for i in 0..z1_temp.len() {
        let out_idx = m + i;
        if out_idx >= out.len() { break; }
        let sum = (out[out_idx] as u64) + (z1_temp[i] as u64) + carry;
        out[out_idx] = sum as u32;
        carry = sum >> 32;
    }

    let mut curr_idx = m + z1_temp.len();
    while carry > 0 && curr_idx < out.len() {
        let sum = (out[curr_idx] as u64) + carry;
        out[curr_idx] = sum as u32;
        carry = sum >> 32;
        curr_idx += 1;
    }
}

// ====================================================================
// 3. 몽고메리 감산 및 모듈로 거듭제곱 (Montgomery Reduction)
// ====================================================================
#[derive(Debug, Clone)]
pub struct MontgomeryContext {
    pub modulus: BigInt,
    pub r_bits: usize,
    pub n_prime: u32,
    pub r_squared: BigInt,
}

impl MontgomeryContext {
    pub fn new(n: BigInt) -> Self {
        assert!(n.digits[0] & 1 == 1, "Modulus N must be odd for Montgomery Reduction.");
        
        let r_bits = n.digits.len() * 32;
        let n_prime = Self::compute_n_prime(n.digits[0]);

        let mut r_sq = BigInt { digits: vec![1] };
        for _ in 0..(2 * r_bits) {
            let mut out = vec![0; r_sq.digits.len() + 1];
            add_slices(&r_sq.digits, &r_sq.digits, &mut out); 
            r_sq = BigInt { digits: out }.trim_zeros();
            
            // Ordering 비교 에러 해결
            if r_sq.cmp(&n) != Ordering::Less {
                r_sq = r_sq.sub(&n);
            }
        }

        MontgomeryContext {
            modulus: n,
            r_bits,
            n_prime,
            r_squared: r_sq,
        }
    }

    fn compute_n_prime(n0: u32) -> u32 {
        let mut x = n0;
        for _ in 0..4 {
            x = x.wrapping_mul(2u32.wrapping_sub(n0.wrapping_mul(x)));
        }
        x.wrapping_neg()
    }

    pub fn reduce(&self, mut t: BigInt) -> BigInt {
        let n_len = self.modulus.digits.len();
        if t.digits.len() < n_len * 2 {
            t.digits.resize(n_len * 2, 0);
        }

        for i in 0..n_len {
            let t_i = t.digits[i];
            let m = t_i.wrapping_mul(self.n_prime);
            
            let mut carry = 0u64;
            for j in 0..n_len {
                let n_j = self.modulus.digits[j] as u64;
                let sum = (t.digits[i + j] as u64) + ((m as u64) * n_j) + carry;
                t.digits[i + j] = sum as u32;
                carry = sum >> 32;
            }
            
            let mut k = i + n_len;
            while carry > 0 && k < t.digits.len() {
                let sum = (t.digits[k] as u64) + carry;
                t.digits[k] = sum as u32;
                carry = sum >> 32;
                k += 1;
            }
        }

        let result_digits = t.digits[n_len..].to_vec();
        let mut result = BigInt { digits: result_digits };
        
        // Ordering 비교 에러 해결
        if result.cmp(&self.modulus) != Ordering::Less {
            result = result.sub(&self.modulus); 
        }
        result.trim_zeros()
    }

    pub fn into_domain(&self, a: &BigInt) -> BigInt {
        self.reduce(a.mul_karatsuba(&self.r_squared))
    }

    pub fn out_of_domain(&self, a_bar: &BigInt) -> BigInt {
        self.reduce(a_bar.clone())
    }

    pub fn mul_in_domain(&self, a_bar: &BigInt, b_bar: &BigInt) -> BigInt {
        self.reduce(a_bar.mul_karatsuba(b_bar))
    }

    pub fn mod_pow(&self, a: &BigInt, b: &BigInt) -> BigInt {
        let mut base_bar = self.into_domain(a);
        let one = BigInt { digits: vec![1] };
        let mut res_bar = self.into_domain(&one);

        for (i, &digit) in b.digits.iter().enumerate() {
            let mut current_bits = digit;
            let bit_len = if i == b.digits.len() - 1 {
                32 - current_bits.leading_zeros()
            } else {
                32
            };

            for _ in 0..bit_len {
                if current_bits & 1 == 1 {
                    res_bar = self.mul_in_domain(&res_bar, &base_bar);
                }
                base_bar = self.mul_in_domain(&base_bar, &base_bar);
                current_bits >>= 1;
            }
        }
        self.out_of_domain(&res_bar)
    }
}

// ====================================================================
// 4. 테스트 실행 (Main)
// ====================================================================
fn main() {
    println!("--- 몽고메리 감산 기반 고성능 모듈로 거듭제곱 테스트 ---");
    
    let a = BigInt { digits: vec![123456789] };
    let b = BigInt { digits: vec![987654321] };
    let n = BigInt { digits: vec![1000000007] }; 
    
    let ctx = MontgomeryContext::new(n.clone());
    let result = ctx.mod_pow(&a, &b);
    
    println!("A = {:?}", a.digits);
    println!("B = {:?}", b.digits);
    println!("N = {:?}", n.digits);
    println!("Result (A^B mod N) = {:?}", result.digits);
    println!("성공적으로 나눗셈 없는 모듈로 거듭제곱 연산을 완료했습니다!");
}
