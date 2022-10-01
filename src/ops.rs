use crate::*;
use crate::vecops::{add, iadd, sub, isub, mul, imul, div};

#[derive(Clone)]
struct Computation {
    value: Vec<DType>
}

#[derive(Clone)]
pub struct Variable(NodeIdx, Computation);

impl Computation {
    fn new(value: Vec<DType>) -> Self {
        Computation { value: value }
    }
}


impl Variable {
    pub fn new(value: Vec<DType>) -> ANode {
        let v = Variable(NodeIdx::new(), Computation::new(value));
        ANode::new(Arc::new(v))
    }
}

impl Node for Variable {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn is_leaf(&self) -> bool { true }

    fn value(&self) -> &[DType] {
        &self.1.value
    }

    fn get_children(&self) -> Option<&[ANode]> { None }

    fn requires_grad(&self) -> bool { true }

    fn compute_grad(&self, _grad: &[DType], _results: &mut [Vec<DType>]) {
        // Pass
    }
}

#[derive(Clone)]
pub struct Constant(NodeIdx, Computation);

impl Constant {
    pub fn new(value: Vec<DType>) -> ANode {
        let c = Constant(NodeIdx::new(), Computation::new(value));
        ANode::new(Arc::new(c))
    }
    pub fn scalar(value: DType) -> ANode {
        Constant::new(vec![value])
    }
}

impl Node for Constant {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { None }

    fn is_leaf(&self) -> bool { true }

    fn value(&self) -> &[DType] {
        &self.1.value
    }

    fn requires_grad(&self) -> bool { false }
}


pub(crate) struct AddN(NodeIdx, Vec<ANode>, Computation);

impl AddN {
    pub(crate) fn new(left: ANode, right: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = AddN::compute(&left, &right);
        let node = AddN(idx, vec![left, right], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode, right: &ANode) -> Vec<DType> {
        let lv = left.value();
        let rv = right.value();
        let mut out = vec![0.; lv.len()];
        add(&lv, &rv, &mut out);
        out
    }
}

impl Node for AddN {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // f(x,y) = x - y
        // df(x,y)/dx = 1
        // df(x,y)/dy = 1
        results[0].clone_from_slice(grad);
        results[1].clone_from_slice(grad);
    }

}

pub(crate) struct Subtract(NodeIdx, Vec<ANode>, Computation);

impl Subtract {
    pub(crate) fn new(left: ANode, right: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Subtract::compute(&left, &right);
        let node = Subtract(idx, vec![left, right], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode, right: &ANode) -> Vec<DType> {
        let lv = left.value();
        let rv = right.value();
        let mut out = vec![0.; lv.len()];
        sub(&lv, &rv, &mut out);
        out
    }
}

impl Node for Subtract {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // f(x,y) = x - y
        // df(x,y)/dx = 1
        // df(x,y)/dy = -1
        results[0].fill(1f32);
        imul(&mut results[0], grad);

        results[1].fill(-1f32);
        imul(&mut results[1], grad);
    }

}

pub(crate) struct Multiply(NodeIdx, Vec<ANode>, Computation);

impl Multiply {
    pub(crate) fn new(left: ANode, right: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Multiply::compute(&left, &right);
        let node = Multiply(idx, vec![left, right], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode, right: &ANode) -> Vec<DType> {
        let lv = left.value();
        let rv = right.value();
        let mut out = vec![0.; lv.len()];
        mul(lv, rv, &mut out);
        out
    }
}

impl Node for Multiply {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // f(x,y) = x * y
        // df(x,y)/dx = y
        // df(x,y)/dy = x
        let x = self.1[0].value();
        let y = self.1[1].value();
        results[0].clone_from_slice(y);
        imul(&mut results[0], &grad);

        results[1].clone_from_slice(x);
        imul(&mut results[1], &grad);
    }

}

pub(crate) struct Divide(NodeIdx, Vec<ANode>, Computation);

impl Divide {
    pub(crate) fn new(left: ANode, right: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Divide::compute(&left, &right);
        let node = Divide(idx, vec![left, right], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode, right: &ANode) -> Vec<DType> {
        let lv = left.value();
        let rv = right.value();
        let mut out = vec![0.; lv.len()];
        div(lv, rv, &mut out);
        out
    }
}

impl Node for Divide {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // f(x,y) = x / y
        let x = self.1[0].value();
        let y = self.1[1].value();
        // df(x,y)/dx = 1 / y
        let mut out = &mut results[0];
        out.iter_mut().zip(y.iter())
            .for_each(|(oi, yi)| *oi = 1f32 / *yi);

        imul(&mut out, grad);
        
        // df(x,y)/dy = x / y ^ 2
        let mut out = &mut results[1];
        out.iter_mut().zip(x.iter().zip(y.iter())).for_each(|(oi, (xi, yi))| {
            *oi = -*xi / yi.powf(2f32);
        });
        imul(&mut out, grad);
    }

}

pub(crate) struct Power(NodeIdx, Vec<ANode>, Computation);

impl Power {
    pub(crate) fn new(base: ANode, exp: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Power::compute(&base, &exp);
        let node = Power(idx, vec![base, exp], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode, right: &ANode) -> Vec<DType> {
        let lv = left.value();
        let rv = right.value();
        lv.iter().zip(rv.iter()).map(|(lvi, rvi)| lvi.powf(*rvi)).collect()
    }
}

impl Node for Power {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // f(x,y) = x ^ y
        // df(x,y)/dx = y * x ^ (y - 1)
        // df(x,y)/dy = ln(y) * x ^ y
        let x = self.1[0].value();
        let y = self.1[1].value();
        
        let mut out = &mut results[0];
        out.iter_mut().zip(x.iter().zip(y.iter()))
            .for_each(|(oi, (xi, yi))| *oi = *yi * xi.powf(*yi - 1f32));
        imul(&mut out, grad);
        
        let mut out = &mut results[1];

        out.iter_mut().zip(x.iter().zip(y.iter()))
            .for_each(|(oi, (xi, yi))| *oi = (yi).ln() * xi.powf(*yi));
        imul(&mut out, grad);
    }

}

pub(crate) struct SumVec(NodeIdx, Vec<ANode>, Computation);

impl SumVec {
    pub(crate) fn new(vec: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = SumVec::compute(&vec);
        let node = SumVec(idx, vec![vec], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode) -> Vec<DType> {
        let lv = left.value();
        vec![lv.iter().sum::<f32>()]
    }
}

impl Node for SumVec {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // f(x) = x.sum()
        // df(x)/dx_1 = 1;
        let out = &mut results[0];
        out.fill(grad[0]);
    }
}

pub(crate) struct Cos(NodeIdx, Vec<ANode>, Computation);

impl Cos {
    pub(crate) fn new(vec: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Cos::compute(&vec);
        let node = Cos(idx, vec![vec], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode) -> Vec<DType> {
        let lv = left.value();
        lv.iter().map(|lvi| lvi.cos()).collect()
    }
}

impl Node for Cos {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        let x = self.1[0].value();
        let out = &mut results[0];
        out.iter_mut().zip(grad.iter().zip(x.iter())).for_each(|(oi, (gi, xi))| {
            *oi = *gi * -xi.sin()
        });
    }
}

pub(crate) struct Sin(NodeIdx, Vec<ANode>, Computation);

impl Sin {
    pub(crate) fn new(vec: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Sin::compute(&vec);
        let node = Sin(idx, vec![vec], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode) -> Vec<DType> {
        let lv = left.value();
        lv.iter().map(|lvi| lvi.sin()).collect()
    }
}

impl Node for Sin {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        let x = self.1[0].value();
        let out = &mut results[0];
        out.iter_mut().zip(grad.iter().zip(x.iter())).for_each(|(oi, (gi, xi))| {
            *oi = *gi * xi.cos()
        });
    }
}

pub(crate) struct Ln(NodeIdx, Vec<ANode>, Computation);

impl Ln {
    pub(crate) fn new(vec: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Ln::compute(&vec);
        let node = Ln(idx, vec![vec], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode) -> Vec<DType> {
        let lv = left.value();
        lv.iter().map(|lvi| lvi.ln()).collect()
    }
}

impl Node for Ln {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        let x = self.1[0].value();
        let out = &mut results[0];
        out.iter_mut().zip(grad.iter().zip(x.iter())).for_each(|(oi, (gi, xi))| {
            *oi = *gi / *xi
        });
    }
}

pub(crate) struct Exp(NodeIdx, Vec<ANode>, Computation);

impl Exp {
    pub(crate) fn new(vec: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Exp::compute(&vec);
        let node = Exp(idx, vec![vec], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode) -> Vec<DType> {
        let lv = left.value();
        lv.iter().map(|lvi| lvi.exp()).collect()
    }
}

impl Node for Exp {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        let x = self.value();
        let mut out = &mut results[0];
        out.clone_from_slice(x);
        imul(&mut out, grad);
    }
}

pub(crate) struct Negate(NodeIdx, Vec<ANode>, Computation);

impl Negate {
    pub(crate) fn new(vec: ANode) -> ANode {
        let idx = NodeIdx::new();
        let value = Negate::compute(&vec);
        let node = Negate(idx, vec![vec], Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(left: &ANode) -> Vec<DType> {
        let lv = left.value();
        lv.iter().map(|lvi| -lvi).collect()
    }
}

impl Node for Negate {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        results[0].iter_mut().zip(grad.iter()).for_each(|(oi, gi)| {
            *oi = -*gi;
        });
    }
}

pub(crate) struct BulkSum(NodeIdx, Vec<ANode>, Computation);

impl BulkSum {
    pub(crate) fn new(vecs: impl Iterator<Item=ANode>) -> ANode {
        let idx = NodeIdx::new();
        let children: Vec<_> = vecs.collect();
        let value = BulkSum::compute(&children);
        let node  = BulkSum(idx, children, Computation::new(value));
        ANode::new(Arc::new(node))
    }

    fn compute(xs: &[ANode]) -> Vec<DType> {
        let mut agg = vec![0.; xs[0].value().len()];
        for x in xs {
            iadd(&mut agg, x.value());
        }
        agg
    }
}

impl Node for BulkSum {
    fn get_id(&self) -> NodeIdx { self.0.clone() }

    fn get_children(&self) -> Option<&[ANode]> { 
        Some(self.1.as_slice())
    }

    fn is_leaf(&self) -> bool { false }

    fn value(&self) -> &[DType] {
        &self.2.value
    }

    fn requires_grad(&self) -> bool { false }

    fn compute_grad(&self, grad: &[DType], results: &mut [Vec<DType>]) {
        // Just the gradient for each, easy peasy
        let x = self.value();
        for out in results.iter_mut() {
            out.clone_from_slice(grad);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_add() {
        let x = Variable::new(vec![0., 1.]);
        let y = Variable::new(vec![2., 3.]);
        let res = AddN::new(x, y);
        assert_eq!(res.value(), &[2., 4.]);
    }

    #[test]
    fn test_sub() {
        let x = Variable::new(vec![0., 1.]);
        let y = Variable::new(vec![2., 3.]);
        let res = Subtract::new(x, y);
        assert_eq!(res.value(), &[-2., -2.]);
    }

    #[test]
    fn test_mul() {
        let x = Variable::new(vec![0., 1.]);
        let y = Variable::new(vec![2., 3.]);
        let res = Multiply::new(x, y);
        assert_eq!(res.value(), &[0., 3.]);
    }

    #[test]
    fn test_div() {
        let x = Variable::new(vec![0., 1.]);
        let y = Variable::new(vec![2., 3.]);
        let res = Divide::new(x, y);
        assert_eq!(res.value(), &[0., 1./3.]);
    }

    #[test]
    fn test_pow() {
        let x = Variable::new(vec![0., 1., 2.]);
        let y = Variable::new(vec![2., 3., 3.]);
        let res = Power::new(x, y);
        assert_eq!(res.value(), &[0., 1., 8.]);
    }

    #[test]
    fn test_exp() {
        let x = Variable::new(vec![0., 1., 2.]);
        let out = (&x).exp();
        let mut graph = Graph::new();
        graph.backward(&out);
        let grad = graph.get_grad(&x);
        assert_eq!(out.value(), &[1., 1f32.exp(), 2f32.exp()]);
    }

    #[test]
    fn test_neg_exp() {
        let x = Variable::new(vec![0., 1., 2.]);
        let nx = -&x;
        let enx = nx.exp();
        let out = enx;
        let mut graph = Graph::new();
        graph.backward(&out);

        let grad = graph.get_grad(&x).unwrap();
        assert_eq!(grad, &[-1., -(-1f32).exp(), -(-2f32).exp()]);
    }

    #[test]
    fn test_backward_pass_simple1() {
        // 2x
        // df/dx = 2
        let x = Variable::new(vec![0f32]);
        let x2 = Multiply::new(x.clone(), Constant::scalar(2f32));

        let mut graph = Graph::new();
        graph.backward(&x2);
        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![2f32]), x_grad);
    }

    #[test]
    fn test_backward_pass_simple2() {
        // 2 + x
        // df/dx = 1
        let x = Variable::new(vec![0f32]);
        let x2 = AddN::new(x.clone(), Constant::scalar(2f32));

        let mut graph = Graph::new();
        graph.backward(&x2);
        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![1f32]), x_grad);
    }

    #[test]
    fn test_backward_pass_simple3() {
        // x - y
        // df/dx = 1
        let x = Variable::new(vec![1f32]);
        let y = Variable::new(vec![2f32]);
        let x2 = Subtract::new(x.clone(), y.clone());

        let mut graph = Graph::new();
        graph.backward(&x2);
        let x_grad = graph.get_grad(&x);
        let y_grad = graph.get_grad(&y);

        assert_eq!(Some(&vec![1f32]), x_grad);
        assert_eq!(Some(&vec![-1f32]), y_grad);
    }

    #[test]
    fn test_backward_pass_simple4() {
        // x ^ 2
        // df/dx = 2x
        let x = Variable::new(vec![1f32]);
        let x2 = Power::new(x.clone(), Constant::scalar(2f32));

        let mut graph = Graph::new();
        graph.backward(&x2);

        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![2f32]), x_grad);
    }

    #[test]
    fn test_backward_pass_simple5() {
        // x ^ 2 + 3x
        // df/dx = 2x + 3
        let x = Variable::new(vec![1f32]);
        let x2 = Power::new(x.clone(), Constant::scalar(2f32));
        let x3 = Multiply::new(x.clone(), Constant::scalar(3f32));
        let x4 = AddN::new(x2, x3);

        assert_eq!(x4.value(), vec![4f32]);

        let mut graph = Graph::new();
        graph.backward(&x4);

        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![5f32]), x_grad);
    }


    #[test]
    fn test_backward_pass_simple6() {
        // 2x + 3
        // df/dx = 2
        let x = Variable::new(vec![0f32]);
        let x2 = Multiply::new(x.clone(), Constant::scalar(2f32));
        let x2_3 = AddN::new(x2, Constant::scalar(3f32));

        let mut graph = Graph::new();
        graph.backward(&x2_3);
        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![2f32]), x_grad);
    }

    #[test]
    fn test_backward_pass_simple7() {
        // dot(x, y)
        let x = Variable::new(vec![1f32, 2f32, 3f32]);
        let y = Variable::new(vec![0f32, 2f32, 4f32]);
        let x2 = Multiply::new(x.clone(), y.clone());
        let ret = SumVec::new(x2);

        assert_eq!(ret.value(), vec![16f32]);
        let mut graph = Graph::new();
        graph.backward(&ret);
        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![0f32, 2f32, 4f32]), x_grad);
    }

    #[test]
    fn test_backward_pass_complicated() {
        // (x+2) ^ 2 
        // x^2 + 4x + 4
        // 2x + 4
        let x      = Variable::new(vec![0f32]);
        let x2     = AddN::new(x.clone(), Constant::scalar(2f32));
        let x2_2   = Power::new(x2.clone(), Constant::scalar(2f32));

        assert_eq!(x2_2.value(), vec![4f32]);

        let mut graph = Graph::new();
        graph.backward(&x2_2);

        let x2_grad = graph.get_grad(&x2);
        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![4f32]), x_grad);
    }

    #[test]
    fn test_composition() {
        // (x+2) ^ 2 
        let x      = Variable::new(vec![0f32]);
        let res = (&x + 2f32).pow(2f32);
        assert_eq!(res.value(), vec![4f32]);

        let mut graph = Graph::new();
        graph.backward(&res);

        let x_grad = graph.get_grad(&x);
        assert_eq!(Some(&vec![4f32]), x_grad);
    }

    #[test]
    fn test_sigmoid_denom() {
        // e ^ -x
        let x      = Variable::new(vec![1f32]);
        let res = &(-&x).exp();
        assert_eq!(res.value(), vec![(-1f32).exp()]);

        let mut graph = Graph::new();
        graph.backward(&res);

        let x_grad = graph.get_grad(&x);
        let x_0 = res.value()[0];
        let expected = -(-1f32).exp();
        assert_eq!(Some(&vec![expected]), x_grad);
    }

    #[test]
    fn test_logistic() {
        // 1 / (1 + e ^ -x)
        let x = Variable::new(vec![0f32]);
        let res = 1f32 / &(&(-&x).exp() + 1f32);
        assert_eq!(res.value(), vec![0.5]);

        let mut graph = Graph::new();
        graph.backward(&res);

        let x_grad = graph.get_grad(&x);
        let sigma_trick = res.value()[0] * (1f32 - res.value()[0]);
        assert_eq!(Some(&vec![sigma_trick]), x_grad);
    }

}
