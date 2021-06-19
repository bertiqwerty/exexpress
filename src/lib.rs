use simple_error::bail;
use num::Float;
use regex::Regex;
use std::{cmp, error::Error, num::ParseFloatError, str::FromStr};
use std::cmp::Ordering;

fn find_numbers(text: &str) -> Result<Vec<f32>, ParseFloatError> {
    let re = Regex::new("\\.?[0-9]+(\\.[0-9]+)?").unwrap();
    re.captures_iter(text)
        .map(|c| c[0].parse::<f32>())
        .collect()
}

#[derive(Copy, Clone)]
struct BinaryOperator<T: Copy> {
    f: fn(T, T) -> T,
    priority: i16,
}

type VecBinOps<'a, T> = Vec<(&'a str, BinaryOperator<T>)>;

fn make_binary_operators<'a, T: Float>() -> (VecBinOps<'a, T>, Regex) {
    (
        [
            ("*", BinaryOperator{f: |a, b| a * b, priority: 1}),
            ("/", BinaryOperator{f: |a, b| a / b, priority: 1}),
            ("+", BinaryOperator{f: |a, b| a + b, priority: 0}),
            ("-", BinaryOperator{f: |a, b| a - b, priority: 0}),
        ].iter().cloned().collect(),
        Regex::new("[*/+\\-]").unwrap(),
    )
}

fn find_op<'a, T: Float>(name: &str, ops: &VecBinOps<'a, T>) -> BinaryOperator<T> {
    ops.iter().find(|(op_name, _)| op_name == &name).unwrap().1
}

fn find_operators<T: Float>(text: &str) -> Vec<BinaryOperator<T>> {
    let (bin_ops, bin_re) = make_binary_operators::<T>();
    bin_re
        .captures_iter(text)
        .map(|c| find_op(&c[0], &bin_ops))
        .collect()
}

fn priorized_indices(bin_ops: &Vec<BinaryOperator<f32>>) -> Vec<usize> {
    let mut indices: Vec<_> = (0..bin_ops.len()).collect();
    indices.sort_by(|i1, i2| bin_ops[*i2].priority.partial_cmp(&bin_ops[*i1].priority).unwrap());
    indices
}

type BoxResult<T> = Result<T,Box<Error>>;

pub fn eval(text: &str) -> BoxResult<f32> {
    let bin_ops = find_operators::<f32>(text); 
    let mut numbers = find_numbers(text)?;
    if bin_ops.len() != numbers.len() - 1 {
        bail!("We expect exactly one number more than operator.");
    };
    let indices = priorized_indices(&bin_ops);
    for i in 0..indices.len() {
        let idx = indices[i];
        numbers[idx] = (bin_ops[idx].f)(numbers[idx], numbers[idx+1]);
        numbers.remove(idx+1);
    }
    Ok(numbers[0])    
}

#[cfg(test)]
mod tests {
    use num::Float;

    use crate::{eval, find_numbers, find_op, find_operators, make_binary_operators, priorized_indices};
    fn assert_float_eq(f1: f32, f2: f32) {
        if (f1-f2).abs() >= 1e-5 {
            panic!("Floats not almost equal.\nf1: {}\nf2: {}\n", f1, f2);
        }
    }
    #[test]
    fn test_find_numbers() {
        assert_eq!(find_numbers("3.4").unwrap(), vec![3.4]);
        assert_eq!(find_numbers("3.465  ").unwrap(), vec![3.465]);
        assert_eq!(find_numbers("0.524 +3.4").unwrap(), vec![0.524, 3.4]);
        assert_eq!(find_numbers("00.524+ 3.4").unwrap(), vec![0.524, 3.4]);
        assert_eq!(find_numbers("0 + 0").unwrap(), vec![0.0, 0.0]);
        assert_eq!(find_numbers("0.0.0+0.").unwrap(), vec![0.0, 0.0, 0.0]);
        assert_eq!(find_numbers(".5+.0").unwrap(), vec![0.5, 0.0]);
    }

    #[test]
    fn test_ops() {
        fn check_add(op: fn(f32, f32) -> f32) {
            assert_eq!(op(1.0, 1.0), 2.0);
        }
        fn check_sub(op: fn(f32, f32) -> f32) {
            assert_eq!(op(1.0, 1.0), 0.0);
        }
        fn check_mul(op: fn(f32, f32) -> f32) {
            assert_eq!(op(1.0, 1.0), 1.0);
            assert_float_eq(op(7.5, 3.0), 22.5);
        }
        fn check_div(op: fn(f32, f32) -> f32) {
            assert_eq!(op(1.0, 1.0), 1.0);
            assert_float_eq(op(1.0, 2.0), 0.5);
        }

        let (bin_ops, _) = make_binary_operators::<f32>();
        check_add(find_op(&"+", &bin_ops).f);
        check_sub(find_op(&"-", &bin_ops).f);
        check_mul(find_op(&"*", &bin_ops).f);
        check_div(find_op(&"/", &bin_ops).f);

        fn check_ops(text: &str) {
            let ops = find_operators::<f32>(text);
            check_add(ops[0].f);
            check_sub(ops[1].f);
            check_mul(ops[2].f);
            check_div(ops[3].f);
        }
        check_ops(&"4.4 + 5-6 *  7/8");
        check_ops(&"0+.7- 32.623*733.1/.8");
        check_ops(&".4+. .5-6..*7./8.");
    }

    #[test]
    fn test_prio(){
        assert_eq!(priorized_indices(&find_operators::<f32>("1+2*3")), vec![1, 0]);
        assert_eq!(priorized_indices(&find_operators::<f32>("1+2*3+4/6")), vec![1, 3, 0, 2]);
        assert_eq!(priorized_indices(&find_operators::<f32>("1*2*3+7-8")), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_eval() {
        assert_float_eq(eval(&"1.3+0.7").unwrap(), 2.0);
        assert_float_eq(eval(&"1.3+0.7*2").unwrap(), 2.7);
        assert_float_eq(eval(&"1.3+0.7*2-1").unwrap(), 1.7);
        assert_float_eq(eval(&"1.3+0.7*2-1/10").unwrap(), 2.6);
    }
}
