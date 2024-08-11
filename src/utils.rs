trait Coeff : Copy + std::cmp::PartialOrd + core::iter::Sum {}

impl Coeff for u32 {}
impl Coeff for f64 {}
impl Coeff for f32 {}

pub trait Matrix : Clone {
    type Item: Coeff;
    type Row;
    fn matrix_map(&self, f: impl Fn(Self::Item) -> Self::Item + Clone) -> Self;
    fn matrix_max(&self) -> Self::Item;
    fn sum_one_level(&self) -> Self::Row;
}
 

impl<T> Matrix for Vec<T> 
where T: Coeff {
    type Item = T;
    type Row = T;
    fn matrix_map(&self, f: impl Fn(T) -> T + Clone) -> Self {
        self.into_iter().map(|x| f(*x)).collect()
    }
    fn matrix_max(&self) -> T {
        *self.into_iter().max_by(|a, b| T::partial_cmp(a, b).unwrap()).unwrap()
    }
    fn sum_one_level(&self) -> T {
        self.into_iter().map(|x| *x).sum()
    }
}

impl<T> Matrix for Vec<Vec<T>> 
where T: Coeff {
    type Item = T;
    type Row = Vec<T>;
    fn matrix_map(&self, f: impl Fn(T) -> T + Clone) -> Self {
        self.into_iter().map(|x| x.matrix_map(f.clone())).collect()
    }
    fn matrix_max(&self) -> Self::Item {
        self.into_iter()
            .map(|x| x.matrix_max())
            .max_by(|a, b| Self::Item::partial_cmp(a, b).unwrap()).unwrap()
    }
    fn sum_one_level(&self) -> Self::Row {
        self.into_iter().map(|x| x.sum_one_level()).collect()
    }
}
