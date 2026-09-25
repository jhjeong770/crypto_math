use std::cmp::Ordering;

// ====================================================================
// [Chapter 1] 거대 정수 (BigInt) 자료구조 및 유틸리티
// ====================================================================
#[derive(Debug, Clone)]
pub struct BigInt {
    pub digits: Vec<u32>, // Little-endian 저장 방식
}

impl BigInt {
    /// 32비트 정수로부터 BigInt 생성
    pub fn from_u32(val: u32) -> Self {
        BigInt { digits: vec![val] }.trim_zeros()
    }

    /// 값이 1인지 확인
    pub fn is_one(&self) -> bool {
        self.digits.len() == 1 && self.digits[0] == 1
    }

    /// 짝수 판별 (하위 1비트 검사)
    pub fn is_even(&self) -> bool {
        self.digits[0] & 1 == 0
    }

    /// 불필요한 상위 0 제거
    pub fn trim_zeros(mut self) -> Self {
        while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
            self.digits.pop();
        }
        self
    }

    /// 두 BigInt 크기 비교
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

    pub fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }

    /// 뺄셈 (반드시 self >= other 가정)
    pub fn sub(&self, other: &Self) -> Self {
        let mut out = vec![0u32; self.digits.len()];
        sub_slices(&self.digits, &other.digits, &mut out);
        BigInt { digits: out }.trim_zeros()
    }

    /// 비트 우측 시프트 (n-1 = 2^s * d 분해 시 사용)
    pub fn shr(&self, shift: u32) -> Self {
        let mut out = self.digits.clone();
        let word_shift = (shift / 32) as usize;
        let bit_shift = shift % 32;
        
        if word_shift > 0 {
            if word_shift >= out.len() { return BigInt::from_u32(0); }
            out.drain(0..word_shift);
        }
        
        if bit_shift > 0 {
            let mut carry = 0u32;
            for i in (0..out.len()).rev() {
                let current = out[i];
                out[i] = (current >> bit_shift) | (carry << (32 - bit_shift));
                carry = current & ((1 << bit_shift) - 1);
            }
        }
        BigInt { digits: out }.trim_zeros()
    }
}

// ====================================================================
// [Chapter 2] Zero-Copy 카라츠바 곱셈 엔진 (O(n^1.585))
// ====================================================================
impl BigInt {
    pub fn mul_karatsuba(&self, other: &Self) -> Self {
        let max_len = std::cmp::max(self.digits.len(), other.digits.len());
        let mut out = vec![0u32; self.digits.len() + other.digits.len()];
        let mut scratch = vec![0u32; max_len * 6];

        karatsuba_core(&self.digits, &other.digits, &mut out, &mut scratch);
        BigInt { digits: out }.trim_zeros()
    }
}

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
// [Chapter 3] 몽고메리 감산 엔진 (나눗셈 없는 모듈로 연산)
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
        assert!(n.digits[0] & 1 == 1, "Modulus N must be odd");
        let r_bits = n.digits.len() * 32;
        let n_prime = Self::compute_n_prime(n.digits[0]);

        let mut r_sq = BigInt::from_u32(1);
        for _ in 0..(2 * r_bits) {
            let mut out = vec![0; r_sq.digits.len() + 1];
            add_slices(&r_sq.digits, &r_sq.digits, &mut out); 
            r_sq = BigInt { digits: out }.trim_zeros();
            
            if r_sq.cmp(&n) != Ordering::Less {
                r_sq = r_sq.sub(&n);
            }
        }
        MontgomeryContext { modulus: n, r_bits, n_prime, r_squared: r_sq }
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
        if t.digits.len() < n_len * 2 { t.digits.resize(n_len * 2, 0); }

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

        let mut result = BigInt { digits: t.digits[n_len..].to_vec() };
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
        let mut res_bar = self.into_domain(&BigInt::from_u32(1));

        for (i, &digit) in b.digits.iter().enumerate() {
            let mut current_bits = digit;
            let bit_len = if i == b.digits.len() - 1 {
                32 - current_bits.leading_zeros()
            } else { 32 };

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
// [Chapter 4] 밀러-라빈 소수 판별기 (Miller-Rabin Primality Test)
// ====================================================================
impl BigInt {
    pub fn is_probably_prime(&self) -> bool {
        if self.eq(&BigInt::from_u32(2)) || self.eq(&BigInt::from_u32(3)) { return true; }
        if self.cmp(&BigInt::from_u32(1)) != Ordering::Greater || self.is_even() { return false; }

        let n_minus_1 = self.sub(&BigInt::from_u32(1));
        
        let mut s = 0;
        for &digit in &n_minus_1.digits {
            if digit == 0 {
                s += 32;
            } else {
                s += digit.trailing_zeros();
                break;
            }
        }
        let d = n_minus_1.shr(s); 

        let ctx = MontgomeryContext::new(self.clone());
        let bases = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

        for &a in &bases {
            let base = BigInt::from_u32(a);
            if self.cmp(&base) != Ordering::Greater { break; }

            let x = ctx.mod_pow(&base, &d);
            if x.is_one() || x.eq(&n_minus_1) { continue; }

            let mut x_bar = ctx.into_domain(&x);
            let mut passed = false;

            for _ in 1..s {
                x_bar = ctx.mul_in_domain(&x_bar, &x_bar);
                let current_x = ctx.out_of_domain(&x_bar);
                
                if current_x.eq(&n_minus_1) {
                    passed = true;
                    break; 
                }
            }
            if !passed { return false; }
        }
        true
    }
}

// ====================================================================
// [Chapter 5] Main 실행부
// ====================================================================
fn main() {
    println!("=== 고성능 암호학 엔진 ===");
    
    // 테스트 1: 메르센 소수 (2^31 - 1)
    let p1 = BigInt::from_u32(2147483647);
    println!("숫자 2,147,483,647 판별 중...");
    if p1.is_probably_prime() {
        println!(" => 완벽한 소수입니다! (밀러-라빈 검증 통과)\n");
    } else {
        println!(" => 합성수입니다.\n");
    }

    // 테스트 2: 합성수 (2147483647 + 2 = 2147483649)
    // u32 범위를 활용하기 위해 배열 구조 직접 할당
    let p2 = BigInt { digits: vec![2147483649] };
    println!("숫자 2,147,483,649 판별 중...");
    if p2.is_probably_prime() {
        println!(" => 완벽한 소수입니다! (밀러-라빈 검증 통과)\n");
    } else {
        println!(" => 합성수입니다. (제곱근 충돌 감지)\n");
    }
}
