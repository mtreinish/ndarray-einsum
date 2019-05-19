use ndarray::prelude::*;
use ndarray::LinalgScalar;

use super::{PairContractor, SingletonContractor, SingletonViewer};
use crate::{Contraction, SizedContraction};

#[derive(Clone, Debug)]
pub struct TensordotFixedPosition {
    len_contracted_lhs: usize,
    len_uncontracted_lhs: usize,
    len_contracted_rhs: usize,
    len_uncontracted_rhs: usize,
    output_shape: Vec<usize>,
}

impl TensordotFixedPosition {
    pub fn new(sc: &SizedContraction) -> Self {
        assert_eq!(sc.contraction.operand_indices.len(), 2);
        let lhs_indices = &sc.contraction.operand_indices[0];
        let rhs_indices = &sc.contraction.operand_indices[1];
        let output_indices = &sc.contraction.output_indices;
        // Returns an n-dimensional array where n = |D| + |E| - 2 * last_n.
        let twice_num_matched_axes = lhs_indices.len() + rhs_indices.len() - output_indices.len();
        assert_eq!(twice_num_matched_axes % 2, 0);
        let num_matched_axes = twice_num_matched_axes / 2;
        // TODO: Add an assert! that they have the same indices

        let mut len_uncontracted_lhs = 1;
        let mut len_uncontracted_rhs = 1;
        let mut len_contracted_lhs = 1;
        let mut len_contracted_rhs = 1;
        let mut output_shape = Vec::new();

        let num_axes_lhs = lhs_indices.len();
        for (axis, index) in lhs_indices.iter().enumerate() {
            let axis_length = sc.output_size[index];
            if axis < (num_axes_lhs - num_matched_axes) {
                len_uncontracted_lhs *= axis_length;
                output_shape.push(axis_length);
            } else {
                len_contracted_lhs *= axis_length;
            }
        }

        for (axis, index) in rhs_indices.iter().enumerate() {
            let axis_length = sc.output_size[index];
            if axis < num_matched_axes {
                len_contracted_rhs *= axis_length;
            } else {
                len_uncontracted_rhs *= axis_length;
                output_shape.push(axis_length);
            }
        }

        TensordotFixedPosition {
            len_contracted_lhs,
            len_uncontracted_lhs,
            len_contracted_rhs,
            len_uncontracted_rhs,
            output_shape,
        }
    }
}

impl<A> PairContractor<A> for TensordotFixedPosition {
    fn contract_pair<'a>(&self, lhs: &'a ArrayViewD<'a, A>, rhs: &'a ArrayViewD<'a, A>) -> ArrayD<A>
    where
        A: Clone + LinalgScalar,
    {
        let lhs_array;
        let lhs_view = if lhs.is_standard_layout() {
            lhs.view()
                .into_shape((self.len_uncontracted_lhs, self.len_contracted_lhs))
                .unwrap()
        } else {
            lhs_array = Array::from_shape_vec(
                [self.len_uncontracted_lhs, self.len_contracted_lhs],
                lhs.iter().cloned().collect(),
            )
            .unwrap();
            lhs_array.view()
        };

        let rhs_array;
        let rhs_view = if rhs.is_standard_layout() {
            rhs.view()
                .into_shape((self.len_uncontracted_rhs, self.len_contracted_rhs))
                .unwrap()
        } else {
            rhs_array = Array::from_shape_vec(
                [self.len_contracted_rhs, self.len_uncontracted_rhs],
                rhs.iter().cloned().collect(),
            )
            .unwrap();
            rhs_array.view()
        };

        lhs_view
            .dot(&rhs_view)
            .into_shape(IxDyn(&self.output_shape))
            .unwrap()
    }
}
