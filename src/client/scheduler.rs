// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use super::MaxClient;

impl MaxClient {
    pub fn start_scheduled_tasks(&self) {
        self.run_scheduled_tasks();
    }
}
