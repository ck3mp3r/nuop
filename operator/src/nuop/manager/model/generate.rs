use kube::CustomResourceExt;
use operator::nuop::manager::NuOperator;
pub fn main() {
    print!("{}", serde_yaml::to_string(&NuOperator::crd()).unwrap());
}
