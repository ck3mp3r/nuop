use kube::Client;

#[derive(Clone)]
pub struct State {
    pub client: Client,
}

impl State {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
