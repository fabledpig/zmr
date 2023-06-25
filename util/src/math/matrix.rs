use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use super::vector::VectorType;
use crate::forward_ref_binop;
use crate::forward_ref_binop_assign;
use crate::forward_ref_unop;

#[derive(Clone, PartialEq, Debug)]
pub struct Matrix<T, const N: usize, const M: usize> {
    pub elements: [[T; N]; M],
}

impl<T: Copy + Default, const N: usize, const M: usize> Matrix<T, N, M> {
    pub fn new(elements: [[T; N]; M]) -> Self {
        Self { elements }
    }
}

impl<T: Copy + Default + From<u8>, const N: usize> Matrix<T, N, N> {
    pub fn identity() -> Self {
        let mut res = Self::default();
        res.elements.iter_mut().enumerate().for_each(|(i, row)| {
            row[i] = 1.into();
        });

        res
    }
}

impl<T: Copy + Default, const N: usize, const M: usize> Default for Matrix<T, N, M> {
    fn default() -> Self {
        Self {
            elements: [[T::default(); N]; M],
        }
    }
}

impl<T: VectorType + Default, const N: usize, const M: usize> Add<&Matrix<T, N, M>>
    for &Matrix<T, N, M>
{
    type Output = Matrix<T, N, M>;

    fn add(self, rhs: &Matrix<T, N, M>) -> Self::Output {
        let mut res = self.clone();
        for i in 0..N * M {
            let x = i % M;
            let y = i / M;

            let lhs = &mut res.elements[y][x];
            *lhs = *lhs + rhs.elements[y][x];
        }

        res
    }
}

forward_ref_binop!(impl [T: VectorType + Default, const N: usize, const M: usize] Add, add for Matrix<T, N, M>, Matrix<T, N, M>);
forward_ref_binop_assign!(impl [T: VectorType + Default, const N: usize, const M: usize] Add, add, AddAssign, add_assign for Matrix<T, N, M>, Matrix<T, N, M>);

impl<T: VectorType + Default, const N: usize, const M: usize> Sub<&Matrix<T, N, M>>
    for &Matrix<T, N, M>
{
    type Output = Matrix<T, N, M>;

    fn sub(self, rhs: &Matrix<T, N, M>) -> Self::Output {
        self + (-rhs)
    }
}

forward_ref_binop!(impl [T: VectorType + Default, const N: usize, const M: usize] Sub, sub for Matrix<T, N, M>, Matrix<T, N, M>);
forward_ref_binop_assign!(impl [T: VectorType + Default, const N: usize, const M: usize] Sub, sub, SubAssign, sub_assign for Matrix<T, N, M>, Matrix<T, N, M>);

impl<T: VectorType + Default, const N: usize, const M: usize> Neg for &Matrix<T, N, M> {
    type Output = Matrix<T, N, M>;

    fn neg(self) -> Self::Output {
        let mut res = self.clone();
        for i in 0..N * M {
            let x = i % M;
            let y = i / M;

            let lhs = &mut res.elements[y][x];
            *lhs = -*lhs;
        }

        res
    }
}

forward_ref_unop!(impl [T: VectorType + Default, const N: usize, const M: usize] Neg, neg for Matrix<T, N, M>);

impl<T: VectorType + Default, const N: usize, const M: usize, const J: usize> Mul<&Matrix<T, M, J>>
    for &Matrix<T, N, M>
{
    type Output = Matrix<T, N, J>;

    fn mul(self, rhs: &Matrix<T, M, J>) -> Self::Output {
        let mut res = Self::Output::default();
        for i in 0..N * J {
            let x = i % J;
            let y = i / J;

            res.elements[y][x] = self.elements[y]
                .iter()
                .zip(rhs.elements.iter().map(|row| row[x]))
                .fold(T::default(), |acc, (lhs, rhs)| acc + *lhs * rhs);
        }

        res
    }
}

forward_ref_binop!(impl [T: VectorType + Default, const N: usize, const M: usize, const J: usize] Mul, mul for Matrix<T, N, M>, Matrix<T, M, J>);
forward_ref_binop_assign!(impl [T: VectorType + Default, const N: usize] Mul, mul, MulAssign, mul_assign for Matrix<T, N, N>, Matrix<T, N, N>);

impl<T: VectorType + Default, const N: usize, const M: usize> Mul<&T> for &Matrix<T, N, M> {
    type Output = Matrix<T, N, M>;

    fn mul(self, rhs: &T) -> Self::Output {
        let mut res = self.clone();
        res.elements.iter_mut().for_each(|row| {
            row.iter_mut()
                .for_each(|element| *element = *element * *rhs)
        });

        res
    }
}

forward_ref_binop!(impl [T: VectorType + Default, const N: usize, const M: usize] Mul, mul for Matrix<T, N, M>, T);
forward_ref_binop_assign!(impl [T: VectorType + Default, const N: usize, const M: usize] Mul, mul, MulAssign, mul_assign for Matrix<T, N, M>, T);

impl<T: VectorType + Default, const N: usize, const M: usize> Div<&T> for &Matrix<T, N, M> {
    type Output = Matrix<T, N, M>;

    fn div(self, rhs: &T) -> Self::Output {
        let mut res = self.clone();
        res.elements.iter_mut().for_each(|row| {
            row.iter_mut()
                .for_each(|element| *element = *element / *rhs)
        });

        res
    }
}

forward_ref_binop!(impl [T: VectorType + Default, const N: usize, const M: usize] Div, div for Matrix<T, N, M>, T);
forward_ref_binop_assign!(impl [T: VectorType + Default, const N: usize, const M: usize] Div, div, DivAssign, div_assign for Matrix<T, N, M>, T);

pub type Matrix2x2i = Matrix<isize, 2, 2>;
pub type Matrix3x3i = Matrix<isize, 3, 3>;
pub type Matrix4x4i = Matrix<isize, 4, 4>;
pub type Matrix2x2f = Matrix<f64, 2, 2>;
pub type Matrix3x3f = Matrix<f64, 3, 3>;
pub type Matrix4x4f = Matrix<f64, 4, 4>;

#[cfg(test)]
mod tests {
    use super::Matrix2x2i;

    #[test]
    fn test_matrix_add() {
        let lhs = Matrix2x2i::new([[1, 2], [3, 4]]);
        let rhs = Matrix2x2i::new([[2, 1], [4, 3]]);
        assert_eq!(Matrix2x2i::new([[3, 3], [7, 7]]), lhs + rhs);
    }

    #[test]
    fn test_matrix_mul() {
        let lhs = Matrix2x2i::new([[1, 2], [3, 4]]);
        let rhs = Matrix2x2i::new([[2, 1], [4, 3]]);
        assert_eq!(Matrix2x2i::new([[10, 7], [22, 15]]), lhs * rhs);
    }
}
