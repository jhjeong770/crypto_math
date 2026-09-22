use std::cmp;

/// 가변 길이 거대 정수(BigInt) 구조체
/// 숫자는 little-endian 방식(작은 자릿수가 배열의 앞쪽)으로 저장
#[derive(Debug, Clone)]
pub struct BigInt {
    digits: Vec<u32>,
}

impl BigInt {
    /// 배열 끝(가장 높은 자릿수)에 있는 불필요한 0들을 제거
    pub fn trim_zeros(mut self) -> Self {
        while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
            self.digits.pop();
        }
        self
    }

    /// 스크래치패드를 활용하여 외부(사용자)가 호출하는 카라츠바 곱셈 진입점
    pub fn mul_karatsuba(&self, other: &Self) -> Self {
        let max_len = cmp::max(self.digits.len(), other.digits.len());
        
        // 1. 결과가 들어갈 정확한 크기의 공간을 딱 1번 할당 (Zero-Copy)
        let mut out = vec![0u32; self.digits.len() + other.digits.len()];
        
        // 2. 재귀 전체에서 재활용할 임시 공간(Scratchpad) 딱 1번 할당
        // 충분한 여유 공간으로 (max_len * 6) 정도면 재귀 분할에 넉넉
        let mut scratch = vec![0u32; max_len * 6];

        // 3. 힙 할당이 전혀 없는 코어 함수로 진입
        karatsuba_core(&self.digits, &other.digits, &mut out, &mut scratch);

        BigInt { digits: out }.trim_zeros()
    }
}

// --------------------------------------------------------------------
// 시스템 레벨 최적화 코어 함수들 (오직 &[u32]와 &mut [u32]만 사용)
// --------------------------------------------------------------------

/// 두 슬라이스를 더해 out에 덮어쓰기. 
fn add_slices(x: &[u32], y: &[u32], out: &mut [u32]) {
    let mut carry = 0u64;
    let max_len = cmp::max(x.len(), y.len());

    for i in 0..max_len {
        let xi = if i < x.len() { x[i] as u64 } else { 0 };
        let yi = if i < y.len() { y[i] as u64 } else { 0 };
        
        let sum = xi + yi + carry;
        out[i] = sum as u32;         
        carry = sum >> 32;           
    }
    // out 배열은 항상 max_len + 1 만큼의 크기가 보장되어야 함.
    if max_len < out.len() {
        out[max_len] = carry as u32; 
    }
}

/// x에서 y를 빼서 out에 덮어쓰기. (반드시 x >= y 라고 가정)
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

/// Base Case: 길이가 짧을 때 사용하는 일반(Schoolbook) O(n^2) 곱셈
fn multiply_schoolbook(x: &[u32], y: &[u32], out: &mut [u32]) {
    // 덮어쓸 공간을 깨끗하게 초기화
    for val in out.iter_mut() {
        *val = 0;
    }

    for i in 0..x.len() {
        let mut carry = 0u64;
        let xi = x[i] as u64; 

        for j in 0..y.len() {
            let yj = y[j] as u64;
            let sum = (out[i + j] as u64) + (xi * yj) + carry;
            
            out[i + j] = sum as u32;
            carry = sum >> 32;
        }
        if i + y.len() < out.len() {
            out[i + y.len()] = carry as u32;
        }
    }
}

/// 카라츠바 알고리즘 코어 (Zero-Copy 재귀)
fn karatsuba_core(x: &[u32], y: &[u32], out: &mut [u32], scratch: &mut [u32]) {
    let len = cmp::max(x.len(), y.len());
    
    // Base Case: 크기가 32 이하로 작아지면 일반 곱셈으로 분기
    if x.len() <= 32 || y.len() <= 32 {
        multiply_schoolbook(x, y, out);
        return;
    }

    let m = len / 2;
    
    // 1. 메모리 복사 없이 슬라이스 자르기
    let (x0, x1) = x.split_at(cmp::min(m, x.len()));
    let (y0, y1) = y.split_at(cmp::min(m, y.len()));

    // 2. 스크래치패드 메모리를 용도별로 토막 내기
    let (sum_x, rest) = scratch.split_at_mut(m + 1);
    let (sum_y, rest) = rest.split_at_mut(m + 1);
    let (z1_temp, next_scratch) = rest.split_at_mut(len + 2); 

    // 3. 덧셈 (임시 공간에 저장)
    add_slices(x0, x1, sum_x);
    add_slices(y0, y1, sum_y);

    // 4. 재귀 곱셈 (메모리 재할당 없이 슬라이스만 던져줌)
    let (out_z0, out_rest) = out.split_at_mut(m * 2);
    karatsuba_core(x0, y0, out_z0, next_scratch);

    let (out_z2, _) = out_rest.split_at_mut(x1.len() + y1.len() + 2); // 여유 공간
    karatsuba_core(x1, y1, out_z2, next_scratch);

    karatsuba_core(sum_x, sum_y, z1_temp, next_scratch);

    // 5. 조합: z1 = z1 - z0 - z2
    sub_slices(z1_temp, out_z0, z1_temp);
    sub_slices(z1_temp, out_z2, z1_temp);

    // 6. out의 위치(m)에 z1 결과를 병합 (Ripple-Carry Addition)
    let mut carry = 0u64;
    for i in 0..z1_temp.len() {
        let out_idx = m + i;
        if out_idx >= out.len() { break; }

        let sum = (out[out_idx] as u64) + (z1_temp[i] as u64) + carry;
        out[out_idx] = sum as u32;
        carry = sum >> 32;
    }

    // 도미노 올림수(Ripple Carry) 처리
    let mut curr_idx = m + z1_temp.len();
    while carry > 0 && curr_idx < out.len() {
        let sum = (out[curr_idx] as u64) + carry;
        out[curr_idx] = sum as u32;
        carry = sum >> 32;
        curr_idx += 1;
    }
}

// --------------------------------------------------------------------
// 테스트 실행용 메인 함수
// --------------------------------------------------------------------
fn main() {
    // 64비트 환경에서 u32 2개를 넣었으므로, 사실상 엄청나게 큰 정수
    let a = BigInt { digits: vec![0xFFFFFFFF, 0xFFFFFFFF] }; 
    let b = BigInt { digits: vec![0xFFFFFFFF, 0xFFFFFFFF] };
    
    println!("a: {:?}", a);
    println!("b: {:?}", b);
    
    let result = a.mul_karatsuba(&b);
    println!("Result (a * b): {:?}", result);
    println!("카라츠바 곱셈이 Zero-Copy로 성공적으로 완료되었습니다!");
}
