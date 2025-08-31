pub fn new_view(path: String) -> View {
    return View{
        path: path
    };
}

pub struct View {
    pub(crate) path: String
}

impl View {
    pub fn render(&mut self, view: String) -> Vec<u8> {
        return vec![]
    }
}