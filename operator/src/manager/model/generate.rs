use kube::CustomResourceExt;
use operator::manager::model::nu_operator;
pub fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&nu_operator::NuOperator::crd()).unwrap()
    );
}
