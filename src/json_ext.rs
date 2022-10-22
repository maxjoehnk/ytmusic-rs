use serde_json::value::Index;

pub(crate) trait JsonExt: Sized {
    fn nav(&self, path: &[&dyn Index]) -> Option<Self>;
}

impl JsonExt for serde_json::Value {
    fn nav(&self, path: &[&dyn Index]) -> Option<Self> {
        let mut root = self;
        for part in path {
            root = root.get(part)?;
        }

        Some(root.clone())
    }
}
