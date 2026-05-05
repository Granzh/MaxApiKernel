use super::MaxClient;

impl MaxClient {
    pub fn start_scheduled_tasks(&self) {
        self.run_scheduled_tasks();
    }
}
