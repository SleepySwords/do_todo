

pub struct Task {
    pub progress: bool,
    pub content: String
}

impl Task {
    pub fn new(content: String) -> Self {
        Task { progress: false, content }
    }

    pub fn update_progresss(&mut self, progress: bool) {
        self.progress = progress;
    }
}
